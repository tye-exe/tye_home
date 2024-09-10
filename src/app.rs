/// Default storage key for my app.
pub const STORAGE_KEY: &'static str = "tye_home";

/// Creates the storage key for the given page.
/// This is a macro due to ownership limitations.
macro_rules! page_storage_key {
    ($string:expr) => {
        format! {"{STORAGE_KEY}-{}", $string}.as_str()
    };
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
/// Contains the data for the example page.
pub struct Example {
    // Example stuff:
    pub label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    pub value: f32,
}

impl Default for Example {
    fn default() -> Self {
        Example {
            label: "Hello world!".to_owned(),
            value: 3.1415926,
        }
    }
}

// Kinded generates a "kind" enum equivalent to this enum; similar to `ErrorKind`
#[derive(serde::Deserialize, serde::Serialize, kinded::Kinded, Debug)]
#[kinded(derive(serde::Deserialize, serde::Serialize), kind = Page)]
/// The possible pages that can be displayed
pub enum PageData {
    Home,
    Example(Example),
}

impl Default for PageData {
    fn default() -> Self {
        Self::Home
    }
}

impl PageData {
    /// Saves the data from this page to storage.
    pub fn save(&self, frame: &mut eframe::Frame) {
        let page = self.kind();
        log::debug!("Saving path: {}", page_storage_key!(page));

        match frame.storage_mut() {
            Some(storage) => {
                log::debug!("Saving data: {:?}", self);
                eframe::set_value(storage, page_storage_key!(page), self);
            }
            None => log::error!("Failed to save path: {}", page_storage_key!(page)),
        }
    }
}

impl Into<PageData> for Page {
    /// Converts a [`Page`] into its respective default [`PageData`].
    fn into(self) -> PageData {
        match self {
            Page::Home => PageData::Home,
            Page::Example => PageData::Example(Default::default()),
        }
    }
}

impl Page {
    /// Creates a [`PageData`] instance from the stored values for this page.
    ///
    /// If no data exists then the default data is used instead.
    pub fn load(self, frame: &mut eframe::Frame) -> PageData {
        log::debug!("Loading path: {}", page_storage_key!(self));

        match frame.storage() {
            Some(storage) => {
                let page_data =
                    eframe::get_value(storage, page_storage_key!(self)).unwrap_or_default();
                log::debug!("Loading data: {:?}", page_data);
                page_data
            }
            None => self.into(),
        }
    }
}

// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
/// Contains the current in-memory data for my app.
pub struct MyApp {
    page_data: PageData,
}

impl MyApp {
    /// Gets the [`Page`] that the current [`PageData`] represents.
    pub fn page(&self) -> Page {
        self.page_data.kind()
    }

    /// Saves the current [`PageData`] & loads the [`PageData`] for the given [`Page`].
    pub fn switch_page(&mut self, page: Page, frame: &mut eframe::Frame) {
        self.page_data.save(frame);
        self.page_data = page.load(frame);
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            page_data: PageData::Example(Example::default()),
        }
    }
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, STORAGE_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for MyApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, STORAGE_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                if !(cfg!(target_arch = "wasm32")) {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);

                ui.add(egui::Separator::default().vertical());

                let home_button =
                    ui.add(egui::Button::new("Home").selected(self.page() == Page::Home));
                let example_button =
                    ui.add(egui::Button::new("Example").selected(self.page() == Page::Example));

                if home_button.clicked() {
                    self.switch_page(Page::Home, frame);
                }
                if example_button.clicked() {
                    self.switch_page(Page::Example, frame);
                }

                let debug_page = ui.add(egui::Button::new("Debug Page"));
                if debug_page.clicked() {
                    log::info!("Page: {}\nPageData: {:?}", self.page(), self.page_data);
                }

                let reset_storage = ui.add(egui::Button::new("Reset Storage"));
                if reset_storage.clicked() {
                    // Overwrites the saved data with default values.
                    for page in Page::all().to_owned() {
                        let page_data: PageData = page.into();
                        page_data.save(frame);
                    }

                    // Sets the current page to its default.
                    self.page_data = self.page().load(frame);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match &mut self.page_data {
                PageData::Example(Example { label, value }) => {
                    // The central panel the region left after adding TopPanel's and SidePanel's
                    ui.heading("eframe template");

                    ui.horizontal(|ui| {
                        ui.label("Write something: ");
                        ui.text_edit_singleline(label);
                    });

                    ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
                    if ui.button("Increment").clicked() {
                        *value += 1.0;
                    }

                    ui.separator();

                    ui.add(egui::github_link_file!(
                        "https://github.com/emilk/eframe_template/blob/main/",
                        "Source code."
                    ));

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        powered_by_egui_and_eframe(ui);
                        egui::warn_if_debug_build(ui);
                    });
                }
                PageData::Home => {}
            }
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
