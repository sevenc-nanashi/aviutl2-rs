use aviutl2::generic::GenericPlugin;

use crate::{CURRENT_ALIAS, LocalAliasPlugin};

#[aviutl2::plugin(FilterPlugin)]
pub struct DummyObject {}

impl aviutl2::filter::FilterPlugin for DummyObject {
    fn new(_info: aviutl2::AviUtl2Info) -> aviutl2::AnyResult<Self> {
        Ok(DummyObject {})
    }

    fn plugin_info(&self) -> aviutl2::filter::FilterPluginTable {
        aviutl2::filter::FilterPluginTable {
            name: "Local Alias".to_string(),
            label: None,
            information: "A dummy filter plugin that does nothing.".to_string(),
            filter_type: aviutl2::filter::FilterType::Both,
            as_object: true,
            config_items: vec![],
        }
    }

    fn proc_video(
        &self,
        _config: &[aviutl2::filter::FilterConfigItem],
        video: &mut aviutl2::filter::FilterProcVideo,
    ) -> anyhow::Result<()> {
        LocalAliasPlugin::with_instance(|plugin| {
            let current_alias = CURRENT_ALIAS.lock().unwrap().clone();
            if let Some(alias) = current_alias {
                plugin
                    .edit_handle
                    .get()
                    .unwrap()
                    .call_edit_section(|section| {
                        for layer in section.layers() {
                            for (_, _obj) in layer.objects() {
                                todo!()
                            }
                        }

                        anyhow::Ok(())
                    })
                    .flatten()
            } else {
                Ok(())
            }
        })?;
        Ok(())
    }
}
