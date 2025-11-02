mod to_plugin_table {
    pub trait ToPluginTable<T> {
        fn initialize_plugin(version: u32) -> bool;
        fn to_plugin_table(&self) -> *mut T;
        fn uninitialize_plugin();
    }
}

use to_plugin_table::ToPluginTable;

/// ホストアプリケーションのハンドル。
/// プラグインの初期化処理で使用します。
///
/// # Panics
///
/// この方がプラグインの初期化処理の外で使用された場合はPanicします。
pub struct HostAppHandle<'a> {
    version: u32,
    internal: *mut aviutl2_sys::plugin2::HOST_APP_TABLE,
    global_leak_manager: &'a mut crate::common::LeakManager,
    kill_switch: std::sync::Arc<std::sync::atomic::AtomicBool>,

    #[cfg(feature = "input")]
    input_plugins: Vec<Box<dyn std::any::Any + Send + Sync>>,
    #[cfg(feature = "output")]
    output_plugins: Vec<Box<dyn std::any::Any + Send + Sync>>,
    #[cfg(feature = "filter")]
    filter_plugins: Vec<Box<dyn std::any::Any + Send + Sync>>,
    #[cfg(feature = "module")]
    script_modules: Vec<Box<dyn std::any::Any + Send + Sync>>,
}

impl<'a> HostAppHandle<'a> {
    pub(crate) unsafe fn new(
        version: u32,
        internal: *mut aviutl2_sys::plugin2::HOST_APP_TABLE,
        global_leak_manager: &'a mut crate::common::LeakManager,
        kill_switch: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        Self {
            version,
            internal,
            global_leak_manager,
            kill_switch,
            #[cfg(feature = "input")]
            input_plugins: Vec::new(),
            #[cfg(feature = "output")]
            output_plugins: Vec::new(),
            #[cfg(feature = "filter")]
            filter_plugins: Vec::new(),
            #[cfg(feature = "module")]
            script_modules: Vec::new(),
        }
    }

    fn assert_not_killed(&self) {
        if self.kill_switch.load(std::sync::atomic::Ordering::SeqCst) {
            panic!("This HostAppHandle is no longer valid.");
        }
    }

    /// プラグインの情報を設定します。
    /// 「プラグイン情報」ダイアログで表示されます。
    pub fn set_plugin_information(&mut self, information: &str) {
        self.assert_not_killed();
        unsafe {
            ((*self.internal).set_plugin_information)(
                self.global_leak_manager.leak_as_wide_string(information),
            )
        }
    }

    /// インポートメニューを登録します。
    ///
    /// # See Also
    ///
    /// - [`register_menus`](Self::register_menus)
    /// - [`aviutl2::generic::menus`]
    pub fn register_import_menu(
        &mut self,
        name: &str,
        callback: extern "C" fn(*mut aviutl2_sys::plugin2::EDIT_SECTION),
    ) {
        self.assert_not_killed();
        unsafe {
            ((*self.internal).register_import_menu)(
                self.global_leak_manager.leak_as_wide_string(name),
                callback,
            )
        }
    }

    /// エクスポートメニューを登録します。
    ///
    /// # See Also
    ///
    /// - [`register_menus`](Self::register_menus)
    /// - [`aviutl2::generic::menus`]
    pub fn register_export_menu(
        &mut self,
        name: &str,
        callback: extern "C" fn(*mut aviutl2_sys::plugin2::EDIT_SECTION),
    ) {
        self.assert_not_killed();
        unsafe {
            ((*self.internal).register_export_menu)(
                self.global_leak_manager.leak_as_wide_string(name),
                callback,
            )
        }
    }

    /// メニューを一括登録します。
    ///
    /// # See Also
    ///
    /// - [`aviutl2::generic::menus`]
    pub fn register_menus<T: GenericPluginMenus>(&mut self) {
        self.assert_not_killed();
        T::register_menus(self);
    }
}

/// 汎用プラグインのメニュー登録用トレイト。
/// [`aviutl2::generic::menus`]マクロで実装できます。
pub trait GenericPluginMenus {
    /// メニューをホストに登録します。
    fn register_menus(host: &mut HostAppHandle);
}

// #[aviutl2::generic::menus] で使用するための再エクスポート
pub use aviutl2_macros::generic_menus as menus;

macro_rules! impl_plugin_registry {
    (
        $description:literal,
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
                unsafe { crate::$module::__bridge::initialize_plugin::<T>(version) }
            }
            fn to_plugin_table(&self) -> *mut $table_type {
                unsafe { crate::$module::__bridge::create_table::<T>() }
            }
            fn uninitialize_plugin() {
                unsafe { crate::$module::__bridge::uninitialize_plugin::<T>() }
            }
        }

        #[cfg(feature = $feature)]
        impl<'a> HostAppHandle<'a> {
            #[doc = concat!($description, "を登録します。")]
            pub fn $register_method<T: $PluginTrait + $SingletonTrait + 'static>(
                &mut self,
                plugin: T,
            ) {
                self.assert_not_killed();
                T::initialize_plugin(self.version);
                let table = plugin.to_plugin_table();
                unsafe {
                    ((*self.internal).$register_method)(table);
                }
                self.$getter_field.push(Box::new(plugin));
            }
        }
    };
}

impl_plugin_registry!(
    "入力プラグイン",
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
    "出力プラグイン",
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
    "フィルタープラグイン",
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
    "スクリプトモジュール",
    "module",
    module,
    script_modules,
    register_script_module,
    get_script_modules,
    crate::module::ScriptModule,
    crate::module::__bridge::ScriptModuleSingleton,
    aviutl2_sys::module2::SCRIPT_MODULE_TABLE
);
