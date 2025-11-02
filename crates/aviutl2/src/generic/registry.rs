mod to_plugin_table {
    pub trait ToPluginTable<T> {
        fn initialize_plugin(version: u32) -> bool;
        fn to_plugin_table(&self) -> *mut T;
        fn uninitialize_plugin();
    }
}
use to_plugin_table::ToPluginTable;

pub(crate) struct PluginRegistry {
    version: u32,

    #[cfg(feature = "input")]
    input_plugins: Vec<(
        Box<dyn std::any::Any + Send + Sync>,
        *mut aviutl2_sys::input2::INPUT_PLUGIN_TABLE,
    )>,
    #[cfg(feature = "output")]
    output_plugins: Vec<(
        Box<dyn std::any::Any + Send + Sync>,
        *mut aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE,
    )>,
    #[cfg(feature = "filter")]
    filter_plugins: Vec<(
        Box<dyn std::any::Any + Send + Sync>,
        *mut aviutl2_sys::filter2::FILTER_PLUGIN_TABLE,
    )>,
    #[cfg(feature = "module")]
    script_modules: Vec<(
        Box<dyn std::any::Any + Send + Sync>,
        *mut aviutl2_sys::module2::SCRIPT_MODULE_TABLE,
    )>,
}

macro_rules! impl_plugin_registry {
    (
        $feature:literal,
        $module:ident,
        $getter_field:ident,
        $register_method:ident,
        $getter_method:ident,
        $PluginTrait:path,
        $SingletonTrait:path,
        $table_type:ty
    ) => {
        #[cfg(feature = $feature)]
        impl<T: $PluginTrait + $SingletonTrait + 'static> ToPluginTable<$table_type> for T {
            fn initialize_plugin(version: u32) -> bool {
                crate::$module::__bridge::initialize_plugin::<T>(version)
            }
            fn to_plugin_table(&self) -> *mut $table_type {
                crate::$module::__bridge::create_table::<T>()
            }
            fn uninitialize_plugin() {
                crate::$module::__bridge::uninitialize_plugin::<T>()
            }
        }

        #[cfg(feature = $feature)]
        impl PluginRegistry {
            pub fn $register_method<T: $PluginTrait + $SingletonTrait + 'static>(
                &mut self,
                plugin: T,
            ) {
                T::initialize_plugin(self.version);
                let table = plugin.to_plugin_table();
                self.$getter_field.push((Box::new(plugin), table));
            }
            pub fn $getter_method(&self) -> Vec<*mut $table_type> {
                self.$getter_field
                    .iter()
                    .map(|(_, table)| *table)
                    .collect()
            }
        }
    };
}
impl_plugin_registry!(
    "input",
    input,
    input_plugins,
    register_input_plugin,
    get_input_plugins,
    crate::input::InputPlugin,
    crate::input::__bridge::InputSingleton,
    aviutl2_sys::input2::INPUT_PLUGIN_TABLE
);
impl_plugin_registry!(
    "output",
    output,
    output_plugins,
    register_output_plugin,
    get_output_plugins,
    crate::output::OutputPlugin,
    crate::output::__bridge::OutputSingleton,
    aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE
);
impl_plugin_registry!(
    "filter",
    filter,
    filter_plugins,
    register_filter_plugin,
    get_filter_plugins,
    crate::filter::FilterPlugin,
    crate::filter::__bridge::FilterSingleton,
    aviutl2_sys::filter2::FILTER_PLUGIN_TABLE
);
impl_plugin_registry!(
    "module",
    module,
    script_modules,
    register_script_module,
    get_script_modules,
    crate::module::ScriptModule,
    crate::module::__bridge::ScriptModuleSingleton,
    aviutl2_sys::module2::SCRIPT_MODULE_TABLE
);
