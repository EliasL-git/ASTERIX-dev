use std::sync::Arc;

use anyhow::Context;
use tokio::runtime::{Builder as RuntimeBuilder, Runtime};
use tokio::sync::{mpsc, oneshot};
use tokio::sync::oneshot::error::TryRecvError;
use tracing::{info, warn};
use url::Url;

pub use asterix_core::{BrowserError, PageRequest, PageResponse, TabId, TabSnapshot};
use asterix_core::BrowserCore;

enum RuntimeCommand {
    Navigate {
        request: PageRequest,
        respond_to: oneshot::Sender<Result<PageResponse, BrowserError>>,
    },
    Shutdown,
}

struct RuntimeInner {
    core: Arc<BrowserCore>,
    tx: mpsc::UnboundedSender<RuntimeCommand>,
}

/// Long-lived runtime responsible for executing asynchronous browser work.
pub struct BrowserRuntime {
    runtime: Runtime,
    inner: Arc<RuntimeInner>,
    supervisor: Option<tokio::task::JoinHandle<()>>,
}

impl BrowserRuntime {
    pub fn new(user_agent: Option<&str>) -> anyhow::Result<Self> {
        let core = Arc::new(BrowserCore::new(user_agent)?);
        let runtime = RuntimeBuilder::new_multi_thread()
            .enable_io()
            .enable_time()
            .worker_threads(4)
            .thread_name("asterix-worker")
            .build()
            .context("failed to construct tokio runtime")?;

        let (tx, mut rx) = mpsc::unbounded_channel();
        let core_for_task = Arc::clone(&core);
        let supervisor = runtime.spawn(async move {
            while let Some(command) = rx.recv().await {
                match command {
                    RuntimeCommand::Navigate { request, respond_to } => {
                        let result = core_for_task.fetch_page(request).await;
                        if respond_to.send(result).is_err() {
                            warn!("navigation consumer dropped before response arrived");
                        }
                    }
                    RuntimeCommand::Shutdown => {
                        info!("browser runtime shutting down");
                        break;
                    }
                }
            }
        });

        let inner = Arc::new(RuntimeInner { core, tx });

        Ok(Self {
            runtime,
            inner,
            supervisor: Some(supervisor),
        })
    }

    /// Returns a lightweight handle for interacting with the runtime from the UI thread.
    pub fn handle(&self) -> BrowserHandle {
        BrowserHandle {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Drop for BrowserRuntime {
    fn drop(&mut self) {
        if let Some(supervisor) = self.supervisor.take() {
            let _ = self.inner.tx.send(RuntimeCommand::Shutdown);
            let _ = self.runtime.block_on(supervisor);
        }
    }
}

/// Public handle exposed to the rest of the application for issuing browser commands.
#[derive(Clone)]
pub struct BrowserHandle {
    inner: Arc<RuntimeInner>,
}

impl BrowserHandle {
    pub fn create_tab(&self, title: impl Into<String>) -> TabSnapshot {
        self.inner.core.create_tab(title)
    }

    pub fn tabs(&self) -> Vec<TabSnapshot> {
        self.inner.core.snapshot_tabs()
    }

    pub fn request_navigation(&self, tab: TabId, url: Url) -> anyhow::Result<NavigationJob> {
        let (respond_to, receiver) = oneshot::channel();
        let request = PageRequest { tab, url };

        self.inner
            .tx
            .send(RuntimeCommand::Navigate { request, respond_to })
            .map_err(|_| anyhow::anyhow!("browser runtime is no longer running"))?;

        Ok(NavigationJob { receiver })
    }
}

/// Represents an in-flight navigation that the UI can poll for completion.
pub struct NavigationJob {
    receiver: oneshot::Receiver<Result<PageResponse, BrowserError>>,
}

impl NavigationJob {
    pub fn try_complete(&mut self) -> Option<Result<PageResponse, BrowserError>> {
        match self.receiver.try_recv() {
            Ok(value) => Some(value),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Closed) => Some(Err(BrowserError::Cancelled)),
        }
    }
}
