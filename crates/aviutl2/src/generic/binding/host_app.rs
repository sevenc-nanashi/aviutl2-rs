use crate::{AviUtl2Info, generic::EditSection};
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

/// 編集ハンドル。
#[derive(Debug)]
pub struct EditHandle {
    pub(crate) internal: *mut aviutl2_sys::plugin2::EDIT_HANDLE,
    edit_info_worker: std::sync::OnceLock<EditInfoWorker>,
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
        Self {
            internal,
            edit_info_worker: std::sync::OnceLock::new(),
        }
    }

    fn edit_info_worker(&self) -> &EditInfoWorker {
        self.edit_info_worker
            .get_or_init(|| EditInfoWorker::new(self.internal))
    }

    /// プロジェクトデータの編集を開始します。
    ///
    /// # Note
    ///
    /// 内部では call_edit_section_param を使用しています。
    pub fn call_edit_section<'a, T, F>(&self, callback: F) -> Result<T, EditHandleError>
    where
        T: Send + 'static,
        F: FnOnce(&mut EditSection) -> T + Send + 'a,
    {
        type CallbackParam<'a, F, T> = (ChildKillablePointer<Option<F>>, &'a mut Option<T>);

        let closure = Some(callback);
        let param = KillablePointer::new(closure);
        let child_param = param.create_child();

        extern "C" fn trampoline<F, T>(
            param: *mut std::ffi::c_void,
            edit_section: *mut aviutl2_sys::plugin2::EDIT_SECTION,
        ) where
            T: Send + 'static,
            F: FnOnce(&mut EditSection) -> T,
        {
            unsafe {
                let (child_param, result_ptr) = &mut *(param as *mut CallbackParam<F, T>);
                let callback = child_param.as_mut().take().expect("Callback already taken");
                let mut edit_section = EditSection::from_raw(edit_section);
                let res = callback(&mut edit_section);
                result_ptr.replace(res);
            }
        }

        let trampoline_static = trampoline::<F, T>
            as extern "C" fn(*mut std::ffi::c_void, *mut aviutl2_sys::plugin2::EDIT_SECTION);

        let mut result = None;
        let param = Box::<CallbackParam<F, T>>::new((child_param, &mut result));
        let param_ptr = Box::into_raw(param);

        let success = unsafe {
            ((*self.internal).call_edit_section_param)(
                param_ptr as *mut std::ffi::c_void,
                trampoline_static,
            )
        };

        drop(unsafe { Box::from_raw(param_ptr) });

        if success {
            Ok(result.expect("Callback did not set result"))
        } else {
            Err(EditHandleError::ApiCallFailed)
        }
    }

    /// 編集情報を取得します。
    ///
    /// # Note
    ///
    /// 既に編集処理中（`call_edit_section` 内）である場合、デッドロックします。
    pub fn get_edit_info(&self) -> crate::generic::EditInfo {
        let mut raw_info = std::mem::MaybeUninit::<aviutl2_sys::plugin2::EDIT_INFO>::uninit();
        unsafe {
            ((*self.internal).get_edit_info)(
                raw_info.as_mut_ptr(),
                std::mem::size_of::<aviutl2_sys::plugin2::EDIT_INFO>() as _,
            );
            let edit_info = raw_info.assume_init();
            crate::generic::EditInfo::from_raw(&edit_info)
        }
    }

    /// 編集情報を取得します。
    ///
    /// [`get_edit_info`] と異なり、タイムアウトを指定できます。
    ///
    /// # Note
    ///
    /// 現在、なぜか別スレッドでのcall_edit_section中にこの関数を呼び出すとデッドロックするため、
    /// タイムアウトを指定できるようにしています。
    pub fn try_get_edit_info(
        &self,
        timeout: std::time::Duration,
    ) -> Result<crate::generic::EditInfo, EditHandleError> {
        let (tx, rx) = std::sync::mpsc::channel();
        if self
            .edit_info_worker()
            .sender
            .send(EditInfoRequest { responder: tx })
            .is_err()
        {
            return Err(EditHandleError::ApiCallFailed);
        }
        match rx.recv_timeout(timeout) {
            Ok(info) => Ok(info),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                log::warn!("try_get_edit_info timed out");
                Err(EditHandleError::ApiCallFailed)
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                Err(EditHandleError::ApiCallFailed)
            }
        }
    }

    /// ホストアプリケーションを再起動します。
    pub fn restart_host_app(&self) {
        unsafe {
            ((*self.internal).restart_host_app)();
        }
    }

    /// エフェクトの一覧をコールバック関数で取得します。
    pub fn enumerate_effects<F>(&self, mut callback: F)
    where
        F: FnMut(Effect),
    {
        extern "C" fn trampoline<F>(
            param: *mut std::ffi::c_void,
            name: aviutl2_sys::common::LPCWSTR,
            r#type: i32,
            flag: i32,
        ) where
            F: FnMut(Effect),
        {
            let callback = unsafe { &mut *(param as *mut F) };
            let name_str = unsafe { crate::common::load_wide_string(name) };
            let effect = Effect {
                name: name_str,
                effect_type: EffectType::from(r#type),
                flag: EffectFlag::from_bits(flag),
            };
            callback(effect);
        }

        let trampoline_static = trampoline::<F>
            as extern "C" fn(*mut std::ffi::c_void, aviutl2_sys::common::LPCWSTR, i32, i32);
        let user_data = &mut callback as *mut F as *mut std::ffi::c_void;
        unsafe {
            ((*self.internal).enum_effect_name)(user_data, trampoline_static);
        }
    }

    /// エフェクトの一覧を取得します。
    pub fn get_effects(&self) -> Vec<Effect> {
        let mut effects = Vec::new();
        self.enumerate_effects(|effect| {
            effects.push(effect);
        });
        effects
    }

    /// モジュールの一覧をコールバック関数で取得します。
    pub fn enumerate_modules<F>(&self, mut callback: F)
    where
        F: FnMut(ModuleInfo),
    {
        extern "C" fn trampoline<F>(
            param: *mut std::ffi::c_void,
            module: *mut aviutl2_sys::plugin2::MODULE_INFO,
        ) where
            F: FnMut(ModuleInfo),
        {
            let callback = unsafe { &mut *(param as *mut F) };
            let module_info = ModuleInfo {
                module_type: ModuleType::from(unsafe { (*module).r#type }),
                name: unsafe { crate::common::load_wide_string((*module).name) },
                information: unsafe { crate::common::load_wide_string((*module).information) },
            };
            callback(module_info);
        }
        let trampoline_static = trampoline::<F>
            as unsafe extern "C" fn(*mut std::ffi::c_void, *mut aviutl2_sys::plugin2::MODULE_INFO);
        let user_data = &mut callback as *mut F as *mut std::ffi::c_void;
        unsafe {
            ((*self.internal).enum_module_info)(user_data, trampoline_static);
        }
    }

    /// モジュールの一覧を取得します。
    pub fn get_modules(&self) -> Vec<ModuleInfo> {
        let mut modules = Vec::new();
        self.enumerate_modules(|module| {
            modules.push(module);
        });
        modules
    }
}

struct InternalSendableEditHandle(*mut aviutl2_sys::plugin2::EDIT_HANDLE);
unsafe impl Send for InternalSendableEditHandle {}
impl InternalSendableEditHandle {
    fn get(&self) -> *mut aviutl2_sys::plugin2::EDIT_HANDLE {
        self.0
    }
}

struct EditInfoRequest {
    responder: std::sync::mpsc::Sender<crate::generic::EditInfo>,
}

#[derive(Debug)]
struct EditInfoWorker {
    sender: std::sync::mpsc::Sender<EditInfoRequest>,
}

impl EditInfoWorker {
    fn new(internal: *mut aviutl2_sys::plugin2::EDIT_HANDLE) -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<EditInfoRequest>();
        let internal = InternalSendableEditHandle(internal);
        std::thread::Builder::new()
            .name("aviutl2-edit-info-worker".to_string())
            .spawn(move || {
                while let Ok(request) = rx.recv() {
                    let mut raw_info =
                        std::mem::MaybeUninit::<aviutl2_sys::plugin2::EDIT_INFO>::uninit();
                    unsafe {
                        ((*internal.get()).get_edit_info)(
                            raw_info.as_mut_ptr(),
                            std::mem::size_of::<aviutl2_sys::plugin2::EDIT_INFO>() as _,
                        );
                        let edit_info = raw_info.assume_init();
                        let info = crate::generic::EditInfo::from_raw(&edit_info);
                        let _ = request.responder.send(info);
                    }
                }
            })
            .expect("Failed to spawn edit info worker thread");
        Self { sender: tx }
    }
}

/// エフェクト情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Effect {
    /// エフェクト名。
    pub name: String,
    /// エフェクト種別。
    pub effect_type: EffectType,
    /// フラグ。
    pub flag: EffectFlag,
}

/// エフェクト種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectType {
    /// フィルタ効果。
    Filter,
    /// メディア入力。
    Input,
    /// シーンチェンジ。
    SceneChange,
    /// その他。
    Other(i32),
}

define_bitflag! {
    /// エフェクトのフラグ。
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[non_exhaustive]
    pub struct EffectFlag: i32 {
        /// 画像フィルタをサポートするかどうか。
        video: aviutl2_sys::plugin2::EDIT_HANDLE::EFFECT_FLAG_VIDEO,

        /// 音声フィルタをサポートするかどうか。
        audio: aviutl2_sys::plugin2::EDIT_HANDLE::EFFECT_FLAG_AUDIO,

        /// フィルタオブジェクトをサポートするかどうか。
        as_filter: aviutl2_sys::plugin2::EDIT_HANDLE::EFFECT_FLAG_FILTER,
    }
}

impl From<i32> for EffectType {
    fn from(value: i32) -> Self {
        match value {
            1 => EffectType::Filter,
            2 => EffectType::Input,
            3 => EffectType::SceneChange,
            other => EffectType::Other(other),
        }
    }
}
impl From<EffectType> for i32 {
    fn from(value: EffectType) -> Self {
        match value {
            EffectType::Filter => 1,
            EffectType::Input => 2,
            EffectType::SceneChange => 3,
            EffectType::Other(other) => other,
        }
    }
}

/// モジュール情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleInfo {
    /// モジュール種別。
    pub module_type: ModuleType,
    /// 名前。
    pub name: String,
    /// 情報。
    pub information: String,
}

/// モジュール種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleType {
    /// フィルタスクリプト。
    ScriptFilter,
    /// オブジェクトスクリプト。
    ScriptObject,
    /// カメラスクリプト。
    ScriptCamera,
    /// トラックバースクリプト。
    ScriptTrack,
    /// スクリプトモジュール。
    ScriptModule,
    /// 入力プラグイン。
    PluginInput,
    /// 出力プラグイン。
    PluginOutput,
    /// フィルタプラグイン。
    PluginFilter,
    /// 汎用プラグイン。
    PluginGeneric,

    /// その他。
    Other(i32),
}

impl From<i32> for ModuleType {
    fn from(value: i32) -> Self {
        match value {
            1 => ModuleType::ScriptFilter,
            2 => ModuleType::ScriptObject,
            3 => ModuleType::ScriptCamera,
            4 => ModuleType::ScriptTrack,
            5 => ModuleType::ScriptModule,
            6 => ModuleType::PluginInput,
            7 => ModuleType::PluginOutput,
            8 => ModuleType::PluginFilter,
            9 => ModuleType::PluginGeneric,
            other => ModuleType::Other(other),
        }
    }
}
impl From<ModuleType> for i32 {
    fn from(value: ModuleType) -> Self {
        match value {
            ModuleType::ScriptFilter => 1,
            ModuleType::ScriptObject => 2,
            ModuleType::ScriptCamera => 3,
            ModuleType::ScriptTrack => 4,
            ModuleType::ScriptModule => 5,
            ModuleType::PluginInput => 6,
            ModuleType::PluginOutput => 7,
            ModuleType::PluginFilter => 8,
            ModuleType::PluginGeneric => 9,
            ModuleType::Other(other) => other,
        }
    }
}

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
            panic!("parent KillablePointer has been dropped");
        }
        unsafe { &mut *self.inner }
    }
}
