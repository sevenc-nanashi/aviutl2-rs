use aviutl2::AnyResult;

mod gui;

#[aviutl2::plugin(GenericPlugin)]
pub struct ScriptsSearchPlugin {
    window: aviutl2_eframe::EframeWindow,
}
#[derive(Debug, Clone, PartialEq)]
pub struct EffectData {
    effect: aviutl2::generic::Effect,
    search_name: nucleo_matcher::Utf32String,
    search_label: nucleo_matcher::Utf32String,
    name: String,
    label: String,
}
pub struct EffectDb {
    effects: Vec<EffectData>,
}
pub static EFFECTS: std::sync::OnceLock<EffectDb> = std::sync::OnceLock::new();

pub static EDIT_HANDLE: std::sync::OnceLock<aviutl2::generic::EditHandle> =
    std::sync::OnceLock::new();

impl aviutl2::generic::GenericPlugin for ScriptsSearchPlugin {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Self::init_logging();
        log::info!("Initializing Rusty Scripts Search Plugin...");
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
        let maybe_config = ScriptsSearchPlugin::load_aviutl2_ini();
        let config = match maybe_config {
            Ok(cfg) => cfg,
            Err(e) => {
                log::error!("Failed to load aviutl2.ini: {}", e);
                return;
            }
        };
        let Some(effects_table) = config.get_table("Effect") else {
            log::error!("Effect section not found in aviutl2.ini");
            return;
        };
        EFFECTS.get_or_init(|| {
            let effects = EDIT_HANDLE.get().unwrap().get_effects();
            let mut has_missing_label = false;
            let effects = effects
                .into_iter()
                .map(|effect| {
                    let section_name =
                        aviutl2::config::get_language_text(&effect.name, &effect.name)
                            .expect("effect name contains null byte");
                    let effects_name = aviutl2::config::get_language_text("Effect", &effect.name)
                        .expect("Effect.name contains null byte");
                    let name = if effect.name != section_name {
                        section_name
                    } else if effect.name != effects_name {
                        effects_name
                    } else {
                        effect.name.clone()
                    };
                    let label = effects_table
                        .get_table(&effect.name)
                        .and_then(|t| t.get_value("label"));
                    let label = match label {
                        Some(l) => aviutl2::config::get_language_text("Effect", l)
                            .expect("effect label contains null byte"),
                        None => {
                            has_missing_label = true;
                            "？？？".to_string()
                        }
                    };
                    EffectData {
                        effect,
                        search_name: nucleo_matcher::Utf32String::from(
                            normalize_kana_for_search(&name).as_str(),
                        ),
                        search_label: nucleo_matcher::Utf32String::from(
                            normalize_kana_for_search(&label).as_str(),
                        ),
                        name,
                        label,
                    }
                })
                .collect();
            EffectDb { effects }
        });
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

    fn load_aviutl2_ini() -> AnyResult<aviutl2::alias::Table> {
        let main_config_path = aviutl2::config::app_data_path().join("aviutl2.ini");
        let content = std::fs::read_to_string(&main_config_path)?;
        Ok(content.parse()?)
    }
}

pub fn normalize_kana_for_search(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }
    input
        .chars()
        .map(|c| {
            if ('\u{3041}'..='\u{3096}').contains(&c) {
                let code = u32::from(c) + 0x60;
                char::from_u32(code).unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}

aviutl2::register_generic_plugin!(ScriptsSearchPlugin);
