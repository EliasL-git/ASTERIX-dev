use std::time::Instant;

use asterix_browser::{BrowserHandle, NavigationJob, TabSnapshot};
use eframe::egui;
use egui::{CentralPanel, Context as EguiContext, Layout, RichText, TopBottomPanel};
use tracing::info;
use url::Url;

/// Launches the native ASTERIX shell on the current thread.
pub fn launch_shell(handle: BrowserHandle) -> anyhow::Result<()> {
    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        follow_system_theme: true,
        viewport: egui::ViewportBuilder::default()
            .with_title("ASTERIX Browser Preview")
            .with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "ASTERIX",
        native_options,
        Box::new(move |_cc| {
            Box::new(ShellApp::new(handle).expect("failed to initialise UI")) as Box<dyn eframe::App>
        }),
    )
    .map_err(|err| anyhow::anyhow!("failed to launch shell: {err}"))
}

struct ShellApp {
    handle: BrowserHandle,
    tabs: Vec<TabSnapshot>,
    active_tab: Option<TabSnapshot>,
    url_input: String,
    nav_jobs: Vec<NavigationJob>,
    status_line: String,
    last_update: Instant,
    page_preview: Option<String>,
}

impl ShellApp {
    fn new(handle: BrowserHandle) -> anyhow::Result<Self> {
        let mut app = Self {
            handle: handle.clone(),
            tabs: Vec::new(),
            active_tab: None,
            url_input: String::new(),
            nav_jobs: Vec::new(),
            status_line: "Ready".to_owned(),
            last_update: Instant::now(),
            page_preview: None,
        };
        let initial_tab = app
            .handle
            .create_tab("New Tab");
        app.active_tab = Some(initial_tab);
        app.refresh_tabs();
        Ok(app)
    }

    fn refresh_tabs(&mut self) {
        self.tabs = self.handle.tabs();
        if let Some(active) = &self.active_tab {
            if let Some(updated) = self.tabs.iter().find(|tab| tab.id == active.id) {
                self.active_tab = Some(updated.clone());
            }
        }
    }

    fn poll_navigation_jobs(&mut self) {
        let mut pending = Vec::with_capacity(self.nav_jobs.len());
        let mut needs_refresh = false;
        for mut job in self.nav_jobs.drain(..) {
            match job.try_complete() {
                Some(Ok(page)) => {
                    info!(target = "ui", "loaded {} ({})", page.url, page.status);
                    self.status_line = format!("Loaded {}", page.url);
                    self.page_preview = Some(generate_preview(&page.body));
                    needs_refresh = true;
                }
                Some(Err(err)) => {
                    self.status_line = format!("Failed: {err}");
                }
                None => pending.push(job),
            }
        }
        self.nav_jobs = pending;
        if needs_refresh {
            self.refresh_tabs();
        }
    }

    fn initiate_navigation(&mut self) {
        if let Some(active) = &self.active_tab {
            if let Ok(url) = parse_user_url(&self.url_input) {
                match self.handle.request_navigation(active.id, url.clone()) {
                    Ok(job) => {
                        self.nav_jobs.push(job);
                        self.status_line = format!("Loading {url}");
                    }
                    Err(err) => {
                        self.status_line = format!("Navigation error: {err}");
                    }
                }
            } else {
                self.status_line = "Enter a valid URL".to_owned();
            }
        }
    }

    fn render_toolbar(&mut self, ctx: &EguiContext) {
        TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui| {
                let tabs_label = if let Some(active) = &self.active_tab {
                    format!("{}", active.title)
                } else {
                    "No Tab".to_owned()
                };
                ui.label(RichText::new(tabs_label).strong());
                ui.separator();

                let url_edit = ui.text_edit_singleline(&mut self.url_input);
                if url_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.initiate_navigation();
                }

                if ui.button("Go").clicked() {
                    self.initiate_navigation();
                }

                if ui.button("New Tab").clicked() {
                    let tab = self.handle.create_tab("New Tab");
                    self.active_tab = Some(tab);
                    self.refresh_tabs();
                }

                ui.separator();
                ui.label(self.status_line.clone());
            });
        });
    }

    fn render_content(&mut self, ctx: &EguiContext) {
        CentralPanel::default().show(ctx, |ui| {
            if let Some(preview) = &self.page_preview {
                ui.heading("Page Preview");
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.code(preview);
                });
            } else {
                ui.heading("Welcome to ASTERIX");
                ui.label("Enter a URL above to load a page. Rendering is limited to a textual preview while the engine evolves.");
            }
        });
    }
}

impl eframe::App for ShellApp {
    fn update(&mut self, ctx: &EguiContext, _frame: &mut eframe::Frame) {
        self.poll_navigation_jobs();
        if self.last_update.elapsed().as_secs() >= 1 {
            self.refresh_tabs();
            self.last_update = Instant::now();
        }
        self.render_toolbar(ctx);
        self.render_content(ctx);
    }
}

fn parse_user_url(input: &str) -> anyhow::Result<Url> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        anyhow::bail!("empty url");
    }

    let parsed = Url::parse(trimmed).or_else(|_| {
        let with_scheme = format!("https://{trimmed}");
        Url::parse(&with_scheme)
    })?;

    Ok(parsed)
}

fn generate_preview(body: &str) -> String {
    const MAX_PREVIEW: usize = 2048;
    let snippet = body.chars().take(MAX_PREVIEW).collect::<String>();
    if body.len() > MAX_PREVIEW {
        format!("{snippet}…")
    } else {
        snippet
    }
}
