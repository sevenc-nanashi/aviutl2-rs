use crate::{AviUtl2Info, generic::EditSection};
use pastey::paste;

/// ホストアプリケーションのハンドル。
/// プラグインの初期化処理で使用します。
///
/// # Panics
///
/// この方がプラグインの初期化処理の外で使用された場合はPanicします。
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
}

/// 汎用プラグインのメニュー登録用トレイト。
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
                pub fn [<new_ $name>](info: AviUtl2Info) -> crate::AnyResult<Self>
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

/// 編集ハンドル。
#[derive(Debug)]
pub struct EditHandle {
    pub(crate) internal: *mut aviutl2_sys::plugin2::EDIT_HANDLE,
}

unsafe impl Send for EditHandle {}
unsafe impl Sync for EditHandle {}

/// [`EditHandle`] 関連のエラー。
#[derive(thiserror::Error, Debug)]
pub enum EditHandleError {
    #[error("api call failed")]
    ApiCallFailed,
}

impl EditHandle {
    pub(crate) unsafe fn new(internal: *mut aviutl2_sys::plugin2::EDIT_HANDLE) -> Self {
        Self { internal }
    }

    /// プロジェクトデータの編集を開始します。
    pub fn call_edit_section<'a, T, F>(&self, callback: F) -> Result<T, EditHandleError>
    where
        T: Send + 'static,
        F: FnOnce(&mut EditSection) -> T + Send + 'a,
    {
        type TrampolineCallback =
            dyn FnOnce(&mut EditSection) -> Box<dyn std::any::Any + Send> + Send;
        static NEXT_CALLBACK: std::sync::Mutex<Option<Box<TrampolineCallback>>> =
            std::sync::Mutex::new(None);

        static CALLBACK_RETURN_VALUE: std::sync::Mutex<Option<Box<dyn std::any::Any + Send>>> =
            std::sync::Mutex::new(None);

        let callback = KillablePointer::new(Some(callback))
            .cast::<Option<Box<dyn FnOnce(&mut EditSection) -> T + Send>>>();

        {
            let mut guard = NEXT_CALLBACK.lock().unwrap();
            *guard = Some(Box::new({
                let mut callback_ref = callback.create_child();
                move |section: &mut EditSection| {
                    if callback_ref.is_killed() {
                        panic!(
                            "EditHandle has been dropped while EditSection callback is pending."
                        );
                    }
                    let callback = unsafe { callback_ref.as_mut() }
                        .take()
                        .expect("EditSection callback has already been called.");
                    let result = callback(section);
                    Box::new(result) as Box<dyn std::any::Any + Send>
                }
            }));
        }
        let call_result = unsafe { ((*self.internal).call_edit_section)(trampoline) };
        if call_result {
            let mut return_guard = CALLBACK_RETURN_VALUE.lock().unwrap();
            if let Some(return_value) = return_guard.take() {
                // 型安全にダウンキャストできるはず
                let boxed: Box<T> = return_value
                    .downcast::<T>()
                    .expect("Type mismatch in EditSection callback return value");
                return Ok(*boxed);
            } else {
                unreachable!("No return value from EditSection callback")
            }
        } else {
            return Err(EditHandleError::ApiCallFailed);
        }

        extern "C" fn trampoline(edit_section: *mut aviutl2_sys::plugin2::EDIT_SECTION) {
            let mut guard = NEXT_CALLBACK.lock().unwrap();
            if let Some(callback) = guard.take() {
                let mut section = unsafe { EditSection::from_ptr(edit_section) };
                let return_value = callback(&mut section);
                let mut return_guard = CALLBACK_RETURN_VALUE.lock().unwrap();
                *return_guard = Some(return_value);
            }
        }
    }
}

struct KillablePointer<T> {
    kill_switch: std::sync::Arc<std::sync::atomic::AtomicBool>,
    inner: *mut T,
}
unsafe impl<T> Send for KillablePointer<T> {}
unsafe impl<T> Sync for KillablePointer<T> {}
impl<T> Drop for KillablePointer<T> {
    fn drop(&mut self) {
        self.kill_switch
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}
impl<T> KillablePointer<T> {
    pub fn new(inner: T) -> Self {
        Self {
            kill_switch: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            inner: Box::into_raw(Box::new(inner)),
        }
    }

    pub fn create_child(&self) -> ChildKillablePointer<T> {
        ChildKillablePointer {
            kill_switch: std::sync::Arc::clone(&self.kill_switch),
            inner: self.inner,
        }
    }

    pub fn cast<U>(self) -> KillablePointer<U> {
        KillablePointer::new(unsafe { std::ptr::read(self.inner as *mut U) })
    }
}

struct ChildKillablePointer<T> {
    kill_switch: std::sync::Arc<std::sync::atomic::AtomicBool>,
    inner: *mut T,
}
unsafe impl<T> Send for ChildKillablePointer<T> {}
unsafe impl<T> Sync for ChildKillablePointer<T> {}
impl<T> ChildKillablePointer<T> {
    pub fn is_killed(&self) -> bool {
        self.kill_switch.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub unsafe fn as_mut(&mut self) -> &mut T {
        if self.is_killed() {
            panic!("Parent KillablePointer has been dropped.");
        }
        unsafe { &mut *self.inner }
    }
}
