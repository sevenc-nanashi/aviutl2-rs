use aviutl2::{filter::FilterConfigItems, generic::GenericPlugin};

use crate::LocalAliasPlugin;

#[aviutl2::plugin(FilterPlugin)]
pub struct DummyObject {}

#[derive(aviutl2::filter::FilterConfigItems)]
struct DummyConfig {
    #[track(
        name = "Marker",
        default = 0,
        range = 0..=1,
        step = 1.0,
    )]
    _marker: u32,
}

impl aviutl2::filter::FilterPlugin for DummyObject {
    fn new(_info: aviutl2::AviUtl2Info) -> aviutl2::AnyResult<Self> {
        Ok(DummyObject {})
    }

    fn plugin_info(&self) -> aviutl2::filter::FilterPluginTable {
        aviutl2::filter::FilterPluginTable {
            name: "Rusty Local Alias".to_string(),
            label: None,
            information: "A dummy filter plugin that does nothing.".to_string(),
            filter_type: aviutl2::filter::FilterType::Video,
            as_object: true,
            config_items: DummyConfig::to_config_items(),
        }
    }

    fn proc_video(
        &self,
        _config: &[aviutl2::filter::FilterConfigItem],
        _video: &mut aviutl2::filter::FilterProcVideo,
    ) -> anyhow::Result<()> {
        LocalAliasPlugin::with_instance(|plugin| {
            let _ = plugin.replace_flag.send(());
        });
        Ok(())
    }
}
