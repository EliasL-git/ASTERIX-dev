use std::sync::Arc;

use anyhow::Context;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use reqwest::redirect::Policy;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::instrument;
use url::Url;

/// Identifier for a logical browser tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(u64);

impl TabId {
    pub fn next(counter: &mut u64) -> Self {
        let id = *counter;
        *counter += 1;
        TabId(id)
    }
}

/// Represents a navigation request initiated by the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRequest {
    pub tab: TabId,
    pub url: Url,
}

/// Minimal representation of a fetched document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageResponse {
    pub url: Url,
    pub status: u16,
    pub mime_type: Option<String>,
    pub title: Option<String>,
    pub body: String,
    pub received_at: DateTime<Utc>,
}

/// Snapshot of the current tab state used by higher layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabSnapshot {
    pub id: TabId,
    pub title: String,
    pub url: Option<Url>,
    pub last_loaded: Option<DateTime<Utc>>,
}

#[derive(Default)]
struct BrowserState {
    next_tab_id: u64,
    tabs: Vec<TabSnapshot>,
}

/// Errors surfaced by the browser core when satisfying network requests.
#[derive(Debug, Error)]
pub enum BrowserError {
    #[error("network request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("invalid UTF-8 body")]
    InvalidBody,
    #[error("navigation was cancelled before completion")]
    Cancelled,
}

/// Core runtime responsible for performing network requests and tracking tab metadata.
pub struct BrowserCore {
    client: reqwest::Client,
    state: Arc<RwLock<BrowserState>>,
}

impl BrowserCore {
    pub fn new(user_agent: Option<&str>) -> anyhow::Result<Self> {
        let mut client_builder = reqwest::Client::builder()
            .redirect(Policy::limited(10))
            .cookie_store(true);

        if let Some(ua) = user_agent {
            client_builder = client_builder.user_agent(ua);
        }

        let client = client_builder
            .build()
            .context("failed to initialise HTTP client")?;

        Ok(Self {
            client,
            state: Arc::default(),
        })
    }

    /// Creates a new logical tab and returns its identifier along with a snapshot.
    pub fn create_tab(&self, title: impl Into<String>) -> TabSnapshot {
        let mut guard = self.state.write();
        let id = TabId::next(&mut guard.next_tab_id);
        let snapshot = TabSnapshot {
            id,
            title: title.into(),
            url: None,
            last_loaded: None,
        };
        guard.tabs.push(snapshot.clone());
        snapshot
    }

    /// Returns a lightweight snapshot of all tabs for UI consumption.
    pub fn snapshot_tabs(&self) -> Vec<TabSnapshot> {
        self.state.read().tabs.clone()
    }

    /// Fetches the provided page request and returns the resulting document.
    #[instrument(skip(self))]
    pub async fn fetch_page(&self, request: PageRequest) -> Result<PageResponse, BrowserError> {
        let response = self
            .client
            .get(request.url.clone())
            .send()
            .await?;

        let status = response.status().as_u16();
        let mime_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);

        let bytes = response.bytes().await?;
        let body = String::from_utf8(bytes.to_vec()).map_err(|_| BrowserError::InvalidBody)?;

        let page = PageResponse {
            url: request.url.clone(),
            status,
            mime_type,
            title: None,
            body,
            received_at: Utc::now(),
        };

        self.update_tab_after_fetch(request.tab, &page);

        Ok(page)
    }

    fn update_tab_after_fetch(&self, tab: TabId, page: &PageResponse) {
        let mut guard = self.state.write();
        if let Some(existing) = guard.tabs.iter_mut().find(|snapshot| snapshot.id == tab) {
            existing.url = Some(page.url.clone());
            existing.last_loaded = Some(page.received_at);
            existing.title = derive_title(page).unwrap_or_else(|| existing.title.clone());
        }
    }
}

fn derive_title(page: &PageResponse) -> Option<String> {
    if let Some(mime) = &page.mime_type {
        if !mime.starts_with("text/html") {
            return Some(page.url.to_string());
        }
    }

    let document = scraper::Html::parse_document(&page.body);
    let selector = scraper::Selector::parse("title").ok()?;
    document
        .select(&selector)
        .next()
        .and_then(|element| element.text().next())
        .map(|title| title.trim().to_owned())
        .filter(|title| !title.is_empty())
}
