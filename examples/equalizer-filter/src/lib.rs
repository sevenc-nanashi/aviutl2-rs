use aviutl2::filter::FilterConfigItems;

#[derive(Debug, aviutl2::filter::FilterConfigItems)]
struct FilterConfig {
    #[track(name = "サンプル整数", min = 0, max = 100, default = 50, step = 1.0)]
    sample_integer: i32,
    #[track(name = "サンプル小数", min = -1.0, max = 1.0, default = 0.0, step = 0.01)]
    sample_float: f64,
    #[check(name = "サンプルチェックボックス", default = true)]
    sample_checkbox: bool,
    #[select(
        name = "サンプルセレクトボックス",
        items = ["オプション1", "オプション2", "オプション3"],
        default = 0
    )]
    sample_select: usize,
    #[color(name = "サンプルカラー", default = 0x48b0d5)]
    sample_color: aviutl2::filter::FilterConfigColorValue,
    #[file(name = "サンプルファイル", filters = {
        "テキストファイル" => ["txt"],
        "すべてのファイル" => ["*"]
    })]
    sample_file: Option<std::path::PathBuf>,
}

struct EqualizerFilter {}

impl aviutl2::filter::FilterPlugin for EqualizerFilter {
    fn new(info: aviutl2::AviUtl2Info) -> aviutl2::AnyResult<Self> {
        aviutl2::odbg!(info);
        Ok(Self {})
    }

    fn plugin_info(&self) -> aviutl2::filter::FilterPluginTable {
        aviutl2::filter::FilterPluginTable {
            name: "Equalizer Filter".to_string(),
            label: None,
            information: "An example equalizer filter plugin.".to_string(),
            filter_type: aviutl2::filter::FilterType::Both,
            wants_initial_input: false,
            config_items: FilterConfig::to_config_items(),
        }
    }
}

aviutl2::register_filter_plugin!(EqualizerFilter);
