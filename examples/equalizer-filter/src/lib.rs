#[aviutl2::filter::filter_config]
struct FilterConfig {
    #[track(name = "サンプル整数", min = 0.0, max = 100.0, default = 50.0, step = 1.0)]
    sample_integer: f64,
}

struct EqualizerFilter {}

impl aviutl2::filter::FilterPlugin for EqualizerFilter {
    fn new() -> Self {
        Self {}
    }

    fn plugin_info(&self) -> aviutl2::filter::FilterPluginTable {
        aviutl2::filter::FilterPluginTable {
            name: "Equalizer Filter".to_string(),
            label: None,
            information: "An example equalizer filter plugin.".to_string(),
            input_type: aviutl2::filter::FilterType::Audio,
            wants_initial_input: false,
            config_items: vec![],
        }
    }
}
