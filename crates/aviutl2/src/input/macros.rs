#[macro_export]
macro_rules! register_input_plugin {
    ($struct:ident) => {
        mod __au2_register_plugin {
            static PLUGIN: std::sync::LazyLock<$struct> = std::sync::LazyLock::new(|| {
                let mut plugin = $struct::default();
                plugin.flag = $struct::FLAG_VIDEO | $struct::FLAG_AUDIO;
                plugin.name = "Input Plugin".into();
                plugin.filefilter = "Input Files (*.inp)\0*.inp\0All Files (*.*)\0*.*\0".into();
                plugin.information = "Input Plugin Information".into();
                plugin.func_input = Some($struct::func_input);
                plugin.func_config = Some($struct::func_config);
                plugin.func_get_config_text = Some($struct::func_get_config_text);
                plugin
            });
            extern "C" fn GetInputPluginTable() -> *mut aviutl2::sys::INPUT_PLUGIN_TABLE {
                &PLUGIN as *const _ as *mut aviutl2::sys::INPUT_PLUGIN_TABLE
            }
        }
    };
}
