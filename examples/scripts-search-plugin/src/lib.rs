use aviutl2::AnyResult;

mod gui;

#[aviutl2::plugin(GenericPlugin)]
pub struct ScriptsSearchPlugin {
    window: aviutl2_eframe::EframeWindow,
}
pub static EFFECTS: std::sync::OnceLock<Vec<aviutl2::generic::Effect>> = std::sync::OnceLock::new();

static EDIT_HANDLE: std::sync::OnceLock<aviutl2::generic::EditHandle> = std::sync::OnceLock::new();

impl aviutl2::generic::GenericPlugin for ScriptsSearchPlugin {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Self::init_logging();
        log::info!("Initializing Rusty Local Alias Plugin...");
        let window =
            aviutl2_eframe::EframeWindow::new("RustyScriptsSearchPlugin", move |cc, handle| {
                Ok(Box::new(gui::ScriptsSearchApp::new(cc, handle)))
            })?;

        Ok(ScriptsSearchPlugin { window })
    }

    fn register(&mut self, registry: &mut aviutl2::generic::HostAppHandle) {
        registry.set_plugin_information(&format!(
            "Scripts Search for AviUtl2, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/scripts-search-plugin",
            version = env!("CARGO_PKG_VERSION")
        ));
        registry
            .register_window_client("Rusty Scripts Search Plugin", &self.window)
            .unwrap();
        let edit_handle = registry.create_edit_handle();
        EDIT_HANDLE.set(edit_handle).unwrap();
    }

    fn on_project_load(&mut self, _project: &mut aviutl2::generic::ProjectFile) {
        EFFECTS.get_or_init(|| EDIT_HANDLE.get().unwrap().get_effects());
    }
}

impl ScriptsSearchPlugin {
    fn init_logging() {
        aviutl2::logger::LogBuilder::new()
            .filter_level(if cfg!(debug_assertions) {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            })
            .init();
    }
}

aviutl2::register_generic_plugin!(ScriptsSearchPlugin);
