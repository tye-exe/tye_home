use std::sync::mpsc;

use circular_queue::CircularQueue;

use crate::{js_imports, LogType};

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

#[derive(serde::Deserialize, serde::Serialize, kinded::Kinded, Debug)]
#[kinded(kind = Layout)]
/// The different layouts that the app could have.
pub enum LayoutData {
    Desktop {},
    Mobile { tabs_open: bool },
}

impl Default for LayoutData {
    fn default() -> Self {
        Self::Desktop {}
    }
}

// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
/// Contains the current in-memory data for my app.
pub struct MyApp {
    /// The data for the currently rendered page.
    page_data: PageData,

    /// Whether the debug window is open.
    debug_window: bool,

    /// Which layout to render.
    layout: LayoutData,

    #[serde(skip)]
    /// A buffer of the 'x' most recent logs.
    logs: CircularQueue<String>,
    #[serde(skip)]
    /// Receives log messages to display.
    log_receiver: Option<mpsc::Receiver<LogType>>,
}

impl MyApp {
    /// Gets the [`Page`] that the current [`PageData`] represents.
    pub fn page(&self) -> Page {
        self.page_data.kind()
    }

    /// Gets the [`Layout`] that the current [`LayoutData`] represents.
    pub fn layout(&self) -> Layout {
        self.layout.kind()
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
            page_data: PageData::Home,
            logs: CircularQueue::with_capacity(16),
            log_receiver: None,
            debug_window: false,
            layout: LayoutData::Desktop {},
        }
    }
}

impl MyApp {
    /// Called once before the first frame.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        log_receiver: Option<mpsc::Receiver<LogType>>,
    ) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Lower scale is too small on mobile.
        match js_imports::is_mobile() {
            true => cc.egui_ctx.set_pixels_per_point(1.2),
            false => cc.egui_ctx.set_pixels_per_point(1.2),
        }

        // Load previous app state (if any).
        let mut app: MyApp = cc
            .storage
            .and_then(|storage| eframe::get_value::<_>(storage, STORAGE_KEY))
            .unwrap_or_default();

        app.log_receiver = log_receiver;

        app
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

        // Differences between mobile & desktop site in one place.
        // let theme_buttons: Box<dyn Fn(&mut egui::Ui)>;
        // let pages: Box<dyn Fn(&mut egui::Ui, &egui::Context, &mut eframe::Frame, &mut MyApp)>;

        // match self.layout() {
        //     Layout::Desktop => {
        //         theme_buttons = Box::new(|ui| {
        //             egui::widgets::global_dark_light_mode_buttons(ui);
        //         });
        //         pages = Box::new(|ui, _, frame, app| {
        //             let home_button =
        //                 ui.add(egui::Button::new("Home").selected(app.page() == Page::Home));
        //             let example_button =
        //                 ui.add(egui::Button::new("Example").selected(app.page() == Page::Example));
        //             let debug_menu =
        //                 ui.add(egui::Button::new("Debug Menu").selected(app.debug_window));

        //             if home_button.clicked() {
        //                 app.switch_page(Page::Home, frame);
        //             }
        //             if example_button.clicked() {
        //                 app.switch_page(Page::Example, frame);
        //             }
        //             if debug_menu.clicked() {
        //                 app.debug_window = !app.debug_window;
        //             }
        //         })
        //     }
        //     Layout::Mobile => {
        //         theme_buttons = Box::new(|ui| {
        //             egui::widgets::global_dark_light_mode_switch(ui);
        //         });
        //         pages = Box::new(|ui, ctx, frame, app| {
        //             get_mobile!(app, {
        //                 let page_button = ui.add(egui::Button::new("Pages").selected(tabs_open));
        //                 if page_button.clicked() {
        //                     tabs_open = !tabs_open;
        //                     egui::Window::new("Pages").show(ctx, |ui| {
        //                         //
        //                     });
        //                 }
        //             });
        //         })
        //     }
        // }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                match self.layout() {
                    Layout::Desktop => egui::widgets::global_dark_light_mode_buttons(ui),
                    Layout::Mobile => egui::widgets::global_dark_light_mode_switch(ui),
                }

                ui.add(egui::Separator::default().vertical());

                match self.layout {
                    LayoutData::Desktop {} => {
                        let home_button =
                            ui.add(egui::Button::new("Home").selected(self.page() == Page::Home));
                        let example_button = ui.add(
                            egui::Button::new("Example").selected(self.page() == Page::Example),
                        );

                        ui.separator();

                        let debug_menu =
                            ui.add(egui::Button::new("Debug Menu").selected(self.debug_window));

                        if home_button.clicked() {
                            self.switch_page(Page::Home, frame);
                        }
                        if example_button.clicked() {
                            self.switch_page(Page::Example, frame);
                        }
                        if debug_menu.clicked() {
                            self.debug_window = !self.debug_window;
                        }
                    }
                    LayoutData::Mobile { ref mut tabs_open } => {
                        let page_button = ui.add(egui::Button::new("Pages").selected(*tabs_open));
                        if page_button.clicked() {
                            *tabs_open = !*tabs_open;
                        }

                        if *tabs_open {
                            egui::Window::new("Pages").show(ctx, |ui| {
                                ui.vertical(|ui| {
                                    let home_button = ui.add(
                                        egui::Button::new("Home")
                                            .selected(self.page() == Page::Home),
                                    );
                                    let example_button = ui.add(
                                        egui::Button::new("Example")
                                            .selected(self.page() == Page::Example),
                                    );

                                    ui.separator();

                                    let debug_menu = ui.add(
                                        egui::Button::new("Debug Menu").selected(self.debug_window),
                                    );

                                    if home_button.clicked() {
                                        self.switch_page(Page::Home, frame);
                                    }
                                    if example_button.clicked() {
                                        self.switch_page(Page::Example, frame);
                                    }
                                    if debug_menu.clicked() {
                                        self.debug_window = !self.debug_window;
                                    }
                                });
                            });
                        }
                    }
                }
            });
        });

        if self.debug_window {
            egui::Window::new("Debug window").show(ctx, |ui| {
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

                ui.separator();
                ui.label("Layout Options:");

                let is_mobile = ui.add(egui::Button::new("Is Mobile?"));
                let toggle_layout = ui.add(egui::Button::new("Toggle Layout"));
                let reset_layout = ui.add(egui::Button::new("Default Layout"));

                if is_mobile.clicked() {
                    log::info!("Mobile: {}", self.layout() == Layout::Mobile);
                }
                if toggle_layout.clicked() {
                    self.layout = match self.layout() == Layout::Mobile {
                        true => LayoutData::Desktop {},
                        false => LayoutData::Mobile { tabs_open: false },
                    };
                    log::info!("New Layout: {}", self.layout());
                }
                if reset_layout.clicked() {
                    let is_mobile = js_imports::is_mobile();

                    self.layout = match is_mobile {
                        false => LayoutData::Desktop {},
                        true => LayoutData::Mobile { tabs_open: false },
                    };

                    log::info!("Default Layout: {}", self.layout());
                }

                ui.separator();
                ui.label("Log Output:");
                // Concats log messages
                let mut collect = self.logs.iter().fold("".to_owned(), |acc, log| acc + log);
                ui.add(egui::TextEdit::multiline(&mut collect));
            });
        }

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
                PageData::Home => {
                    ui.heading("Welcome!");
                    ui.separator();
                    ui.label("Hello, i'm tye! I'm non-binary & go by they/them, thank you for being respectfull.");
                    ui.label("");

                    // ui.with_layout(, )
                    ui.horizontal_wrapped(|ui| {
                        let vec2 = ui.style().spacing.item_spacing.clone();
                        ui.style_mut().spacing.item_spacing = egui::Vec2::new(0.0,0.0);
                        ui.label("My favorite pastime is fighting with computers, which ");
                        ui.label(egui::RichText::new("sometimes").italics());
                        ui.label(" goes smoothly.");
                        // ui.label("test o.0");

                        ui.style_mut().spacing.item_spacing = vec2;
                    });
                }
            }
        });

        // Updates the log buffer
        let log = match &self.log_receiver {
            Some(receiver) => match receiver.try_recv() {
                Ok(log) => Some(log),
                Err(_) => None,
            },
            None => None,
        };

        if let Some((level, text)) = log {
            self.logs.push(format!("{}: {}\n", level, text));
        }
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
