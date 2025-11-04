use anyhow::Context;
use asterix_browser::BrowserRuntime;
use tracing::Level;
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    setup_tracing()?;

    let runtime = BrowserRuntime::new(Some(DEFAULT_USER_AGENT))
        .context("failed to start browser runtime")?;
    let handle = runtime.handle();

    asterix_ui::launch_shell(handle)?;

    Ok(())
}

fn setup_tracing() -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::default().add_directive(Level::INFO.into()));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .try_init()
        .ok();

    Ok(())
}

const DEFAULT_USER_AGENT: &str = concat!(
    "Mozilla/5.0 (X11; Linux x86_64) ",
    "AppleWebKit/537.36 (KHTML, like Gecko) ",
    "ASTERIX/0.1 Safari/537.36"
);
