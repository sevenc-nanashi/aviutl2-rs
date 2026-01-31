use aviutl2::AnyResult;

mod gui;
mod metronome;
mod wav;

pub static EDIT_HANDLE: std::sync::OnceLock<aviutl2::generic::EditHandle> =
    std::sync::OnceLock::new();

#[aviutl2::plugin(GenericPlugin)]
pub struct MetronomePlugin {
    window: aviutl2_eframe::EframeWindow,
    metronome: aviutl2::generic::SubPlugin<crate::metronome::MetronomeFilter>,
}

impl aviutl2::generic::GenericPlugin for MetronomePlugin {
    fn new(info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Self::init_logging();
        log::info!("Initializing Rusty Metronome Plugin...");
        let window =
            aviutl2_eframe::EframeWindow::new("RustyMetronomePlugin", move |cc, handle| {
                Ok(Box::new(gui::MetronomeApp::new(cc, handle)))
            })?;

        Ok(Self {
            window,
            metronome: aviutl2::generic::SubPlugin::new_filter_plugin(&info)?,
        })
    }

    fn register(&mut self, registry: &mut aviutl2::generic::HostAppHandle) {
        registry.register_filter_plugin(&self.metronome);
        registry.set_plugin_information(&format!(
            "Metronome for AviUtl2, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/metronome-plugin",
            version = env!("CARGO_PKG_VERSION")
        ));
        registry
            .register_window_client("Rusty Metronome Plugin", &self.window)
            .unwrap();
        let edit_handle = registry.create_edit_handle();
        EDIT_HANDLE.set(edit_handle).unwrap();
    }

    fn on_clear_cache(&mut self, _edit_section: &aviutl2::generic::EditSection) {
        crate::wav::clear_sample_cache();
    }
}

impl MetronomePlugin {
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

aviutl2::register_generic_plugin!(MetronomePlugin);
