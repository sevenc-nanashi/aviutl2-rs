use crate::AviUtl2Info;
use pastey::paste;
use std::num::NonZeroIsize;

/// ホストアプリケーションのハンドル。
/// プラグインの初期化処理で使用します。
///
/// # Panics
///
/// この型がプラグインの初期化処理の外で使用された場合はPanicします。
pub struct HostAppHandle<'a> {
    internal: *mut aviutl2_sys::plugin2::HOST_APP_TABLE,
    global_leak_manager: &'a mut crate::common::LeakManager,
    kill_switch: std::sync::Arc<std::sync::atomic::AtomicBool>,
    plugin_registry: &'a mut crate::generic::PluginRegistry,
}

/// プラグインの初期化状態を管理するためのハンドル。
pub struct SubPlugin<T> {
    plugin: std::marker::PhantomData<T>,
    internal: std::sync::Arc<InternalReferenceHandle>,
}
struct InternalReferenceHandle {
    uninitialize_fn: fn(),
}
impl Drop for InternalReferenceHandle {
    fn drop(&mut self) {
        (self.uninitialize_fn)();
    }
}

impl<'a> HostAppHandle<'a> {
    pub(crate) unsafe fn new(
        internal: *mut aviutl2_sys::plugin2::HOST_APP_TABLE,
        global_leak_manager: &'a mut crate::common::LeakManager,
        kill_switch: std::sync::Arc<std::sync::atomic::AtomicBool>,
        plugin_registry: &'a mut crate::generic::PluginRegistry,
    ) -> Self {
        Self {
            internal,
            global_leak_manager,
            kill_switch,
            plugin_registry,
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
        let information = if cfg!(debug_assertions) {
            format!("{information} (Debug Build)")
        } else {
            information.to_string()
        };
        unsafe {
            ((*self.internal).set_plugin_information)(
                self.global_leak_manager.leak_as_wide_string(&information),
            )
        }
    }

    /// プロジェクトデータ編集用のハンドルを登録します。
    pub fn create_edit_handle(&mut self) -> crate::generic::EditHandle {
        self.assert_not_killed();
        let raw_handle = unsafe { ((*self.internal).create_edit_handle)() };
        unsafe { crate::generic::EditHandle::new(raw_handle) }
    }

    /// インポートメニューを登録します。
    ///
    /// # See Also
    ///
    /// - [`crate::generic::menus`]
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
    /// - [`crate::generic::menus`]
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

    /// レイヤーメニューを登録します。
    /// レイヤー編集でオブジェクト未選択時の右クリックメニューに追加されます。
    ///
    /// # See Also
    ///
    /// - [`crate::generic::menus`]
    pub fn register_layer_menu(
        &mut self,
        name: &str,
        callback: extern "C" fn(*mut aviutl2_sys::plugin2::EDIT_SECTION),
    ) {
        self.assert_not_killed();
        unsafe {
            ((*self.internal).register_layer_menu)(
                self.global_leak_manager.leak_as_wide_string(name),
                callback,
            )
        }
    }

    /// オブジェクトメニューを登録します。
    /// レイヤー編集でオブジェクト選択時の右クリックメニューに追加されます。
    ///
    /// # See Also
    ///
    /// - [`crate::generic::menus`]
    pub fn register_object_menu(
        &mut self,
        name: &str,
        callback: extern "C" fn(*mut aviutl2_sys::plugin2::EDIT_SECTION),
    ) {
        self.assert_not_killed();
        unsafe {
            ((*self.internal).register_object_menu)(
                self.global_leak_manager.leak_as_wide_string(name),
                callback,
            )
        }
    }

    /// 編集メニューを登録します。
    /// 名前に`\\`を含めるとサブメニューとして登録されます。
    ///
    /// # See Also
    ///
    /// - [`crate::generic::menus`]
    pub fn register_edit_menu(
        &mut self,
        name: &str,
        callback: extern "C" fn(*mut aviutl2_sys::plugin2::EDIT_SECTION),
    ) {
        self.assert_not_killed();
        unsafe {
            ((*self.internal).register_edit_menu)(
                self.global_leak_manager.leak_as_wide_string(name),
                callback,
            )
        }
    }

    /// 設定メニューを登録します。
    /// 設定メニューの登録後にウィンドウクライアントを登録するとシステムメニューに「設定」が追加されます。
    ///
    /// # See Also
    ///
    /// - [`crate::generic::menus`]
    pub fn register_config_menu(
        &mut self,
        name: &str,
        callback: extern "C" fn(aviutl2_sys::plugin2::HWND, aviutl2_sys::plugin2::HINSTANCE),
    ) {
        self.assert_not_killed();
        unsafe {
            ((*self.internal).register_config_menu)(
                self.global_leak_manager.leak_as_wide_string(name),
                callback,
            )
        };
    }

    /// ウィンドウクライアントを登録します。
    ///
    /// # Panics
    ///
    /// Win32のウィンドウハンドル以外が渡された場合はPanicします。
    pub fn register_window_client<T: raw_window_handle::HasWindowHandle>(
        &mut self,
        name: &str,
        instance: &T,
    ) -> Result<(), raw_window_handle::HandleError> {
        self.assert_not_killed();
        let raw_handle = instance.window_handle()?;
        let hwnd = match raw_handle.as_raw() {
            raw_window_handle::RawWindowHandle::Win32(handle) => handle.hwnd,
            _ => panic!("Only Win32WindowHandle is supported"),
        };
        unsafe {
            ((*self.internal).register_window_client)(
                self.global_leak_manager.leak_as_wide_string(name),
                hwnd.get() as *mut std::ffi::c_void,
            );
        }
        Ok(())
    }

    /// メニューを一括登録します。
    ///
    /// # See Also
    ///
    /// - [`crate::generic::menus`]
    pub fn register_menus<T: GenericPluginMenus>(&mut self) {
        self.assert_not_killed();
        T::register_menus(self);
    }

    /// プロジェクトファイルをロードした直後に呼ばれる関数を登録します。
    /// また、プロジェクトの初期化時にも呼ばれます。
    ///
    /// # Note
    ///
    /// [`crate::generic::GenericPlugin::on_project_load`] が自動的に登録されるため、
    /// 通常はこの関数を直接使用する必要はありません。
    pub fn register_project_load_handler(
        &mut self,
        callback: extern "C" fn(*mut aviutl2_sys::plugin2::PROJECT_FILE),
    ) {
        self.assert_not_killed();
        unsafe {
            ((*self.internal).register_project_load_handler)(callback);
        }
    }

    /// プロジェクトファイルを保存する直前に呼ばれる関数を登録します。
    ///
    /// # Note
    ///
    /// [`crate::generic::GenericPlugin::on_project_save`] が自動的に登録されるため、
    /// 通常はこの関数を直接使用する必要はありません。
    pub fn register_project_save_handler(
        &mut self,
        callback: extern "C" fn(*mut aviutl2_sys::plugin2::PROJECT_FILE),
    ) {
        self.assert_not_killed();
        unsafe {
            ((*self.internal).register_project_save_handler)(callback);
        }
    }

    /// 「キャッシュを破棄」が呼ばれたときに呼ばれる関数を登録します。
    ///
    /// # Note
    ///
    /// [`crate::generic::GenericPlugin::on_clear_cache`] が自動的に登録されるため、
    /// 通常はこの関数を直接使用する必要はありません。
    pub fn register_clear_cache_handler(
        &mut self,
        callback: extern "C" fn(*mut aviutl2_sys::plugin2::EDIT_SECTION),
    ) {
        self.assert_not_killed();
        unsafe {
            ((*self.internal).register_clear_cache_handler)(callback);
        }
    }

    /// シーンを変更した直後に呼ばれる関数を登録します。
    ///
    /// # Note
    ///
    /// [`crate::generic::GenericPlugin::on_change_scene`] が自動的に登録されるため、
    /// 通常はこの関数を直接使用する必要はありません。
    pub fn register_change_scene_handler(
        &mut self,
        callback: extern "C" fn(*mut aviutl2_sys::plugin2::EDIT_SECTION),
    ) {
        self.assert_not_killed();
        unsafe {
            ((*self.internal).register_change_scene_handler)(callback);
        }
    }
}

/// 汎用プラグインのメニュー登録用トレイト。
///
/// <div class="warning">
///
/// このトレイトは [`crate::generic::menus`] マクロで自動的に実装されます。
/// 通常は手動で実装する必要はありません。
///
/// </div>
pub trait GenericPluginMenus {
    fn register_menus(host: &mut HostAppHandle);
}

#[doc(inline)]
pub use aviutl2_macros::generic_menus as menus;

#[derive(Default)]
pub(crate) struct PluginRegistry {
    #[cfg(feature = "input")]
    input_plugins: Vec<std::sync::Arc<InternalReferenceHandle>>,
    #[cfg(feature = "output")]
    output_plugins: Vec<std::sync::Arc<InternalReferenceHandle>>,
    #[cfg(feature = "filter")]
    filter_plugins: Vec<std::sync::Arc<InternalReferenceHandle>>,
    #[cfg(feature = "module")]
    script_modules: Vec<std::sync::Arc<InternalReferenceHandle>>,
}
impl PluginRegistry {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

macro_rules! impl_plugin_registry {
    (
        $description:literal,
        $feature:literal,
        $module:ident,
        $name:ident,
        $register_method:ident,
        $PluginTrait:path,
        $SingletonTrait:path,
        $TableType:ty
    ) => {
        paste! {
            impl<T> SubPlugin<T> {
                #[cfg(feature = $feature)]
                #[doc = concat!($description, "の新しいインスタンスを作成します。")]
                pub fn [<new_ $name>](info: &AviUtl2Info) -> crate::AnyResult<Self>
                where
                    T: $PluginTrait + $SingletonTrait + 'static
                {
                    crate::$module::__bridge::initialize_plugin::<T>(info.version.into())?;
                    let internal = std::sync::Arc::new(InternalReferenceHandle {
                        uninitialize_fn: || {
                            unsafe {
                                crate::$module::__bridge::uninitialize_plugin::<T>();
                            }
                        },
                    });
                    Ok(Self {
                        plugin: std::marker::PhantomData,
                        internal,
                    })
                }
            }
            #[cfg(feature = $feature)]
            impl<'a> HostAppHandle<'a> {
                #[doc = concat!($description, "を登録します。")]
                pub fn [<register_ $name>]<T: $PluginTrait + $SingletonTrait + 'static>(
                    &mut self,
                    handle: &SubPlugin<T>,
                ) {
                    self.assert_not_killed();
                    unsafe { ((*self.internal).$register_method)(crate::$module::__bridge::create_table_unwind::<T>()) };
                    self.plugin_registry
                        .[<$name s>]
                        .push(std::sync::Arc::clone(&handle.internal));
                }
                #[doc = concat!("unwindなしで", $description, "を登録します。")]
                pub fn [<register_ $name _nounwind>]<T: $PluginTrait + $SingletonTrait + 'static>(
                    &mut self,
                    handle: &SubPlugin<T>,
                ) {
                    self.assert_not_killed();
                    unsafe { ((*self.internal).$register_method)(crate::$module::__bridge::create_table::<T>()) };
                    self.plugin_registry
                        .[<$name s>]
                        .push(std::sync::Arc::clone(&handle.internal));
                }
            }
        }
    };
}

impl_plugin_registry!(
    "入力プラグイン",
    "input",
    input,
    input_plugin,
    register_input_plugin,
    crate::input::InputPlugin,
    crate::input::__bridge::InputSingleton,
    aviutl2_sys::input2::INPUT_PLUGIN_TABLE
);
impl_plugin_registry!(
    "出力プラグイン",
    "output",
    output,
    output_plugin,
    register_output_plugin,
    crate::output::OutputPlugin,
    crate::output::__bridge::OutputSingleton,
    aviutl2_sys::output2::OUTPUT_PLUGIN_TABLE
);
impl_plugin_registry!(
    "フィルタープラグイン",
    "filter",
    filter,
    filter_plugin,
    register_filter_plugin,
    crate::filter::FilterPlugin,
    crate::filter::__bridge::FilterSingleton,
    aviutl2_sys::filter2::FILTER_PLUGIN_TABLE
);
impl_plugin_registry!(
    "スクリプトモジュール",
    "module",
    module,
    script_module,
    register_script_module,
    crate::module::ScriptModule,
    crate::module::__bridge::ScriptModuleSingleton,
    aviutl2_sys::module2::SCRIPT_MODULE_TABLE
);

#[doc(hidden)]
pub unsafe fn __internal_rwh_from_raw(
    hwnd: aviutl2_sys::plugin2::HWND,
    hinstance: aviutl2_sys::plugin2::HINSTANCE,
) -> raw_window_handle::Win32WindowHandle {
    let mut handle =
        raw_window_handle::Win32WindowHandle::new(NonZeroIsize::new(hwnd as isize).unwrap());
    handle.hinstance = Some(NonZeroIsize::new(hinstance as isize).unwrap());
    handle
}
