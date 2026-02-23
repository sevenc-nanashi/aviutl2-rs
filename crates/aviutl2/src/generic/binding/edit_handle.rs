use std::num::NonZeroIsize;

use crate::common::{ChildKillablePointer, KillablePointer};
use crate::generic::EditSection;

/// 編集ハンドル。
///
/// # Panics
///
/// [`crate::generic::GenericPlugin::register`]が終了するまでは、以下のメソッド以外は呼び出せません。
/// - [`Self::get_host_app_window`]
/// - [`Self::get_host_app_window_raw`]
/// - [`Self::is_ready`]
#[derive(Debug)]
pub struct EditHandle {
    pub(crate) internal: *mut aviutl2_sys::plugin2::EDIT_HANDLE,
    pub(crate) is_registerplugin_done: std::sync::Arc<std::sync::atomic::AtomicBool>,
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
    pub(crate) unsafe fn new(
        internal: *mut aviutl2_sys::plugin2::EDIT_HANDLE,
        is_registerplugin_done: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        Self {
            internal,
            is_registerplugin_done,
        }
    }

    /// 編集ハンドルが使用可能かどうかを確認します。
    pub fn is_ready(&self) -> bool {
        self.is_registerplugin_done
            .load(std::sync::atomic::Ordering::Acquire)
    }

    /// プロジェクトデータの編集を開始する。
    ///
    /// # Note
    ///
    /// 内部では call_edit_section_param を使用しています。
    pub fn call_edit_section<'a, T, F>(&self, callback: F) -> Result<T, EditHandleError>
    where
        T: Send + 'static,
        F: FnOnce(&mut EditSection) -> T + Send + 'a,
    {
        assert!(
            self.is_ready(),
            "call_edit_section cannot be called before register_plugin is done"
        );

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

    /// 編集情報を取得する。
    ///
    /// # Note
    ///
    /// 既に編集処理中（`call_edit_section` 内）である場合、デッドロックします。
    pub fn get_edit_info(&self) -> crate::generic::EditInfo {
        assert!(
            self.is_ready(),
            "call_edit_section cannot be called before register_plugin is done"
        );
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

    /// ホストアプリケーションを再起動する。
    pub fn restart_host_app(&self) {
        assert!(
            self.is_ready(),
            "call_edit_section cannot be called before register_plugin is done"
        );
        unsafe {
            ((*self.internal).restart_host_app)();
        }
    }

    /// エフェクトの一覧をコールバック関数で取得する。
    pub fn enumerate_effects<F>(&self, callback: F)
    where
        F: FnMut(Effect),
    {
        assert!(
            self.is_ready(),
            "call_edit_section cannot be called before register_plugin is done"
        );
        type CallbackParam<F> = ChildKillablePointer<F>;

        extern "C" fn trampoline<F>(
            param: *mut std::ffi::c_void,
            name: aviutl2_sys::common::LPCWSTR,
            r#type: i32,
            flag: i32,
        ) where
            F: FnMut(Effect),
        {
            let callback = unsafe { &mut *(param as *mut CallbackParam<F>) };
            let callback = unsafe { callback.as_mut() };
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
        let callback_guard = KillablePointer::new(callback);
        let child_param = callback_guard.create_child();
        let param = Box::new(child_param);
        let param_ptr = Box::into_raw(param);
        unsafe {
            ((*self.internal).enum_effect_name)(
                param_ptr as *mut std::ffi::c_void,
                trampoline_static,
            );
        }
        drop(unsafe { Box::from_raw(param_ptr) });
    }

    /// エフェクトの一覧を取得する。
    pub fn get_effects(&self) -> Vec<Effect> {
        assert!(
            self.is_ready(),
            "call_edit_section cannot be called before register_plugin is done"
        );
        let mut effects = Vec::new();
        self.enumerate_effects(|effect| {
            effects.push(effect);
        });
        effects
    }

    /// モジュールの一覧をコールバック関数で取得する。
    pub fn enumerate_modules<F>(&self, callback: F)
    where
        F: FnMut(ModuleInfo),
    {
        assert!(
            self.is_ready(),
            "call_edit_section cannot be called before register_plugin is done"
        );
        type CallbackParam<F> = ChildKillablePointer<F>;

        extern "C" fn trampoline<F>(
            param: *mut std::ffi::c_void,
            module: *mut aviutl2_sys::plugin2::MODULE_INFO,
        ) where
            F: FnMut(ModuleInfo),
        {
            let callback = unsafe { &mut *(param as *mut CallbackParam<F>) };
            let callback = unsafe { callback.as_mut() };
            let module_info = ModuleInfo {
                module_type: ModuleType::from(unsafe { (*module).r#type }),
                name: unsafe { crate::common::load_wide_string((*module).name) },
                information: unsafe { crate::common::load_wide_string((*module).information) },
            };
            callback(module_info);
        }
        let trampoline_static = trampoline::<F>
            as unsafe extern "C" fn(*mut std::ffi::c_void, *mut aviutl2_sys::plugin2::MODULE_INFO);
        let callback_guard = KillablePointer::new(callback);
        let child_param = callback_guard.create_child();
        let param = Box::new(child_param);
        let param_ptr = Box::into_raw(param);
        unsafe {
            ((*self.internal).enum_module_info)(
                param_ptr as *mut std::ffi::c_void,
                trampoline_static,
            );
        }
        drop(unsafe { Box::from_raw(param_ptr) });
    }

    /// モジュールの一覧を取得する。
    pub fn get_modules(&self) -> Vec<ModuleInfo> {
        assert!(
            self.is_ready(),
            "call_edit_section cannot be called before register_plugin is done"
        );
        let mut modules = Vec::new();
        self.enumerate_modules(|module| {
            modules.push(module);
        });
        modules
    }

    /// ホストアプリケーションのメインウィンドウのハンドルを[`raw_window_handle::Win32WindowHandle`]として取得する。
    pub fn get_host_app_window_raw(&self) -> Option<raw_window_handle::Win32WindowHandle> {
        let hwnd = unsafe { ((*self.internal).get_host_app_window)() };
        NonZeroIsize::new(hwnd as isize).map(raw_window_handle::Win32WindowHandle::new)
    }

    /// ホストアプリケーションのメインウィンドウのハンドルを[`raw_window_handle::WindowHandle`]として取得する。
    ///
    /// # Safety
    ///
    /// [`raw_window_handle::WindowHandle::borrow_raw`] を参照してください。
    pub unsafe fn get_host_app_window(&'_ self) -> Option<raw_window_handle::WindowHandle<'_>> {
        self.get_host_app_window_raw().map(|handle| unsafe {
            raw_window_handle::WindowHandle::borrow_raw(raw_window_handle::RawWindowHandle::Win32(
                handle,
            ))
        })
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

/// グローバルに [EditHandle] を保持するための構造体。
///
/// `OnceLock` と違い、もし初期化していない状態でアクセスした場合にパニックします。
#[derive(Debug)]
pub struct GlobalEditHandle {
    edit_handle: std::sync::OnceLock<crate::generic::EditHandle>,
}

impl GlobalEditHandle {
    /// 新しいインスタンスを作成する。
    pub const fn new() -> Self {
        Self {
            edit_handle: std::sync::OnceLock::new(),
        }
    }

    /// 初期化する。すでに初期化されている場合は警告をログに出力します。
    pub fn init(&self, edit_handle: crate::generic::EditHandle) {
        let _ = self
            .edit_handle
            .set(edit_handle)
            .map_err(|_| log::warn!("GlobalEditHandle was already initialized"));
    }

    /// 初期化されているかどうかを確認します。
    pub fn is_ready(&self) -> bool {
        self.edit_handle
            .get()
            .is_some_and(|handle| handle.is_ready())
    }
}

impl Default for GlobalEditHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for GlobalEditHandle {
    type Target = crate::generic::EditHandle;

    fn deref(&self) -> &Self::Target {
        self.edit_handle
            .get()
            .expect("GlobalEditHandle is not initialized")
    }
}
