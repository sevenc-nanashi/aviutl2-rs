use std::num::NonZeroIsize;
use std::sync::Mutex;

use crate::common::{ChildKillablePointer, KillablePointer};
use crate::filter::RgbaPixel;
use crate::generic::{EditSection, ReadSection};

/// 編集ハンドル。
///
/// # Panics
///
/// [`crate::generic::GenericPlugin::register`]が終了するまでは、以下のメソッド以外は呼び出せません。
/// - [`Self::get_host_app_window`]
/// - [`Self::get_host_app_window_raw`]
/// - [`Self::is_ready`]
///
#[derive(Debug)]
pub struct EditHandle {
    pub(crate) internal: *mut aviutl2_sys::plugin2::EDIT_HANDLE,
    pub(crate) is_ready: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

unsafe impl Send for EditHandle {}
unsafe impl Sync for EditHandle {}

/// [`EditHandle`] 関連のエラー。
#[derive(thiserror::Error, Debug)]
pub enum EditHandleError {
    #[error("api call failed")]
    ApiCallFailed,
    #[error("effect does not exist")]
    EffectNotFound,
    #[error("input utf-16 string contains null byte")]
    InputCwstrContainsNull(#[from] crate::common::NullByteError),
    #[error("unknown edit state: {0}")]
    UnknownEditState(i32),
    #[error("value is out of range")]
    ValueOutOfRange,
}

/// シーンの映像レンダリング結果。
#[derive(Debug, Clone, Copy)]
pub struct RenderingSceneVideo<'a> {
    /// レンダリング対象のフレーム。
    pub frame: u32,
    /// レンダリングした画像の幅。
    pub width: u32,
    /// レンダリングした画像の高さ。
    pub height: u32,
    /// 画像データの横1ラインのバイト数。
    pub pitch: u32,
    /// レンダリングした画像データ。
    ///
    /// データはコールバック中のみ有効です。
    pub buffer: &'a [u8],
}

impl RenderingSceneVideo<'_> {
    /// レンダリングした画像データを RGBA ピクセル列として取得する。
    ///
    /// `pitch == width * size_of::<RgbaPixel>()` の場合のみ `Some` を返します。
    pub fn as_rgba_pixels(&self) -> Option<&[RgbaPixel]> {
        let expected_pitch = self.width as usize * std::mem::size_of::<RgbaPixel>();
        if self.pitch as usize != expected_pitch {
            return None;
        }
        Some(unsafe {
            std::slice::from_raw_parts(
                self.buffer.as_ptr() as *const RgbaPixel,
                self.width as usize * self.height as usize,
            )
        })
    }
}

/// シーンの音声レンダリング結果。
#[derive(Debug, Clone, Copy)]
pub struct RenderingSceneAudio<'a> {
    /// レンダリング対象のフレーム。
    pub frame: u32,
    /// レンダリングした音声データ(左チャンネル)。
    ///
    /// データはコールバック中のみ有効です。
    pub buffer0: &'a [f32],
    /// レンダリングした音声データ(右チャンネル)。
    ///
    /// データはコールバック中のみ有効です。
    pub buffer1: &'a [f32],
}

impl EditHandle {
    pub(crate) unsafe fn new(
        internal: *mut aviutl2_sys::plugin2::EDIT_HANDLE,
        is_ready: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        Self { internal, is_ready }
    }

    /// 編集ハンドルの初期化が完了しているかどうかを確認します。
    pub fn is_ready(&self) -> bool {
        self.is_ready.load(std::sync::atomic::Ordering::Acquire)
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

    /// プロジェクトデータの参照を開始する。
    ///
    /// # Note
    ///
    /// 内部では call_read_section_param を使用しています。
    pub fn call_read_section<'a, T, F>(&self, callback: F) -> Result<T, EditHandleError>
    where
        T: Send + 'static,
        F: FnOnce(&ReadSection) -> T + Send + 'a,
    {
        assert!(
            self.is_ready(),
            "call_read_section cannot be called before register_plugin is done"
        );

        type CallbackParam<'a, F, T> = (ChildKillablePointer<Option<F>>, &'a mut Option<T>);

        let closure = Some(callback);
        let param = KillablePointer::new(closure);
        let child_param = param.create_child();

        extern "C" fn trampoline<F, T>(
            param: *mut std::ffi::c_void,
            read_section: *mut aviutl2_sys::plugin2::EDIT_SECTION,
        ) where
            T: Send + 'static,
            F: FnOnce(&ReadSection) -> T,
        {
            unsafe {
                let (child_param, result_ptr) = &mut *(param as *mut CallbackParam<F, T>);
                let callback = child_param.as_mut().take().expect("Callback already taken");
                let read_section = ReadSection::from_raw(read_section);
                let res = callback(&read_section);
                result_ptr.replace(res);
            }
        }

        let trampoline_static = trampoline::<F, T>
            as extern "C" fn(*mut std::ffi::c_void, *mut aviutl2_sys::plugin2::EDIT_SECTION);

        let mut result = None;
        let param = Box::<CallbackParam<F, T>>::new((child_param, &mut result));
        let param_ptr = Box::into_raw(param);

        let success = unsafe {
            ((*self.internal).call_read_section_param)(
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
    pub fn get_edit_info(&self) -> crate::generic::EditInfo {
        assert!(
            self.is_ready(),
            "get_edit_info cannot be called before register_plugin is done"
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
            "restart_host_app cannot be called before register_plugin is done"
        );
        unsafe {
            ((*self.internal).restart_host_app)();
        }
    }

    /// エフェクトの一覧をコールバック関数で取得する。
    ///
    /// # Note
    ///
    /// 不明なエフェクト種別があった場合はスキップされます。
    pub fn enumerate_effects<F>(&self, callback: F)
    where
        F: FnMut(Effect),
    {
        assert!(
            self.is_ready(),
            "enumerate_effects cannot be called before register_plugin is done"
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
            if let Ok(effect_type) = EffectType::try_from(r#type) {
                let effect = Effect {
                    name: name_str,
                    effect_type,
                    flag: EffectFlag::from_bits(flag),
                };
                callback(effect);
            } else {
                tracing::warn!("Unknown effect type: {}", r#type);
            }
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
            "get_effects cannot be called before register_plugin is done"
        );
        let mut effects = Vec::new();
        self.enumerate_effects(|effect| {
            effects.push(effect);
        });
        effects
    }

    /// エフェクトの設定項目一覧をコールバック関数で取得する。
    ///
    /// # Arguments
    ///
    /// - `effect`: 対象のエフェクト名。エイリアスファイルの `effect.name` を指定します。
    ///
    /// # Note
    ///
    /// 不明な設定項目種別があった場合はスキップされます。
    pub fn enumerate_effect_items<F>(
        &self,
        effect: &str,
        callback: F,
    ) -> Result<(), EditHandleError>
    where
        F: FnMut(EffectItemInfo),
    {
        assert!(
            self.is_ready(),
            "enumerate_effect_items cannot be called before register_plugin is done"
        );
        type CallbackParam<F> = ChildKillablePointer<F>;

        unsafe extern "C" fn trampoline<F>(
            param: *mut std::ffi::c_void,
            name: aviutl2_sys::common::LPCWSTR,
            r#type: i32,
        ) where
            F: FnMut(EffectItemInfo),
        {
            let callback = unsafe { &mut *(param as *mut CallbackParam<F>) };
            let callback = unsafe { callback.as_mut() };
            let name = unsafe { crate::common::load_wide_string(name) };
            if let Some(info) = effect_item_info_from_raw(name, r#type) {
                callback(info);
            }
        }

        let effect = crate::common::CWString::new(effect)?;
        let trampoline_static = trampoline::<F>
            as unsafe extern "C" fn(*mut std::ffi::c_void, aviutl2_sys::common::LPCWSTR, i32);
        let callback_guard = KillablePointer::new(callback);
        let child_param = callback_guard.create_child();
        let param = Box::new(child_param);
        let param_ptr = Box::into_raw(param);
        let success = unsafe {
            ((*self.internal).enum_effect_item)(
                effect.as_ptr(),
                param_ptr as *mut std::ffi::c_void,
                trampoline_static,
            )
        };
        drop(unsafe { Box::from_raw(param_ptr) });

        if success {
            Ok(())
        } else {
            Err(EditHandleError::EffectNotFound)
        }
    }

    /// エフェクトの設定項目一覧を取得する。
    ///
    /// # Arguments
    ///
    /// - `effect`: 対象のエフェクト名。エイリアスファイルの `effect.name` を指定します。
    pub fn get_effect_items(&self, effect: &str) -> Result<Vec<EffectItemInfo>, EditHandleError> {
        assert!(
            self.is_ready(),
            "get_effect_items cannot be called before register_plugin is done"
        );
        let mut items = Vec::new();
        self.enumerate_effect_items(effect, |item| {
            items.push(item);
        })?;
        Ok(items)
    }

    /// モジュールの一覧をコールバック関数で取得する。
    pub fn enumerate_modules<F>(&self, callback: F)
    where
        F: FnMut(ModuleInfo),
    {
        assert!(
            self.is_ready(),
            "enumerate_modules cannot be called before register_plugin is done"
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
            if let Some(module_info) = module_info_from_raw(module) {
                callback(module_info);
            }
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
            "get_modules cannot be called before register_plugin is done"
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

    /// 編集状態を取得する。
    pub fn get_edit_state(&self) -> Result<EditState, EditHandleError> {
        assert!(
            self.is_ready(),
            "get_edit_state cannot be called before register_plugin is done"
        );
        let state = unsafe { ((*self.internal).get_edit_state)() };
        EditState::try_from(state).map_err(|_| EditHandleError::UnknownEditState(state))
    }

    /// 現在のシーンの映像レンダリングを要求する。
    ///
    /// この関数はレンダリングタスクの追加のみを行います。レンダリング完了時に
    /// コールバックがレンダリング用スレッドから呼ばれます。
    pub fn rendering_scene_video<F>(&self, frame: u32, callback: F) -> Result<(), EditHandleError>
    where
        F: FnMut(RenderingSceneVideo<'_>) + Send + 'static,
    {
        assert!(
            self.is_ready(),
            "rendering_scene_video cannot be called before register_plugin is done"
        );

        type CallbackParam<F> = Mutex<Option<F>>;

        unsafe extern "C" fn trampoline<F>(
            param: *mut std::ffi::c_void,
            frame: i32,
            buffer: *const std::ffi::c_void,
            width: i32,
            height: i32,
            pitch: i32,
        ) where
            F: FnMut(RenderingSceneVideo<'_>),
        {
            let callback = unsafe { Box::from_raw(param as *mut CallbackParam<F>) };
            let len = usize::try_from(pitch)
                .ok()
                .and_then(|pitch| {
                    usize::try_from(height)
                        .ok()
                        .and_then(|height| pitch.checked_mul(height))
                })
                .unwrap_or(0);
            let buffer = if buffer.is_null() || len == 0 {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(buffer as *const u8, len) }
            };
            let video = RenderingSceneVideo {
                frame: frame as u32,
                width: width as u32,
                height: height as u32,
                pitch: pitch as u32,
                buffer,
            };
            let mut callback = callback
                .lock()
                .unwrap()
                .take()
                .expect("Callback already taken");
            callback(video);
        }

        let frame = i32::try_from(frame).map_err(|_| EditHandleError::ValueOutOfRange)?;
        let param = Box::into_raw(Box::new(Mutex::new(Some(callback))));
        let success = unsafe {
            ((*self.internal).rendering_scene_video)(
                frame,
                param as *mut std::ffi::c_void,
                trampoline::<F>,
            )
        };
        if success {
            Ok(())
        } else {
            unsafe {
                drop(Box::from_raw(param));
            }
            Err(EditHandleError::ApiCallFailed)
        }
    }

    /// 現在のシーンの音声レンダリングを要求する。
    ///
    /// この関数はレンダリングタスクの追加のみを行います。レンダリング完了時に
    /// コールバックがレンダリング用スレッドから呼ばれます。
    pub fn rendering_scene_audio<F>(&self, frame: u32, callback: F) -> Result<(), EditHandleError>
    where
        F: FnMut(RenderingSceneAudio<'_>) + Send + 'static,
    {
        assert!(
            self.is_ready(),
            "rendering_scene_audio cannot be called before register_plugin is done"
        );

        type CallbackParam<F> = Mutex<Option<F>>;

        unsafe extern "C" fn trampoline<F>(
            param: *mut std::ffi::c_void,
            frame: i32,
            buffer0: *const f32,
            buffer1: *const f32,
            sample_num: i32,
        ) where
            F: FnMut(RenderingSceneAudio<'_>),
        {
            let callback = unsafe { Box::from_raw(param as *mut CallbackParam<F>) };
            let len = usize::try_from(sample_num).unwrap_or(0);
            let buffer0 = if buffer0.is_null() || len == 0 {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(buffer0, len) }
            };
            let buffer1 = if buffer1.is_null() || len == 0 {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(buffer1, len) }
            };
            let audio = RenderingSceneAudio {
                frame: frame as u32,
                buffer0,
                buffer1,
            };
            let mut callback = callback
                .lock()
                .unwrap()
                .take()
                .expect("Callback already taken");
            callback(audio);
        }

        let frame = i32::try_from(frame).map_err(|_| EditHandleError::ValueOutOfRange)?;
        let param = Box::into_raw(Box::new(Mutex::new(Some(callback))));
        let success = unsafe {
            ((*self.internal).rendering_scene_audio)(
                frame,
                param as *mut std::ffi::c_void,
                trampoline::<F>,
            )
        };
        if success {
            Ok(())
        } else {
            unsafe {
                drop(Box::from_raw(param));
            }
            Err(EditHandleError::ApiCallFailed)
        }
    }

    /// レンダリング中のタスクが全て完了するまで待機する。
    ///
    /// # Note
    ///
    /// <div class="warning">
    ///
    /// この関数を[`Self::call_read_section`]や[`Self::call_edit_section`]のコールバック内で呼び出すとデッドロックする可能性があります。
    ///
    /// </div>
    pub fn wait_rendering_task(&self) {
        assert!(
            self.is_ready(),
            "wait_rendering_task cannot be called before register_plugin is done"
        );
        unsafe {
            ((*self.internal).wait_rendering_task)();
        }
    }

    /// フォント名の一覧をコールバック関数で取得する。
    pub fn enumerate_font_names<F>(&self, callback: F)
    where
        F: FnMut(String),
    {
        assert!(
            self.is_ready(),
            "enumerate_font_names cannot be called before register_plugin is done"
        );
        type CallbackParam<F> = ChildKillablePointer<F>;

        unsafe extern "C" fn trampoline<F>(
            param: *mut std::ffi::c_void,
            name: aviutl2_sys::common::LPCWSTR,
        ) where
            F: FnMut(String),
        {
            let callback = unsafe { &mut *(param as *mut CallbackParam<F>) };
            let callback = unsafe { callback.as_mut() };
            let name_str = unsafe { crate::common::load_wide_string(name) };
            callback(name_str);
        }

        let trampoline_static = trampoline::<F>
            as unsafe extern "C" fn(*mut std::ffi::c_void, aviutl2_sys::common::LPCWSTR);
        let callback_guard = KillablePointer::new(callback);
        let child_param = callback_guard.create_child();
        let param = Box::new(child_param);
        let param_ptr = Box::into_raw(param);
        unsafe {
            ((*self.internal).enum_font_name)(
                param_ptr as *mut std::ffi::c_void,
                trampoline_static,
            );
        }
        drop(unsafe { Box::from_raw(param_ptr) });
    }

    /// フォント名の一覧を取得する。
    pub fn get_font_names(&self) -> Vec<String> {
        assert!(
            self.is_ready(),
            "get_font_names cannot be called before register_plugin is done"
        );
        let mut font_names = Vec::new();
        self.enumerate_font_names(|name| {
            font_names.push(name);
        });
        font_names
    }

    /// パレット名の一覧をコールバック関数で取得する。
    pub fn enumerate_palette_names<F>(&self, callback: F)
    where
        F: FnMut(String),
    {
        assert!(
            self.is_ready(),
            "enumerate_palette_names cannot be called before register_plugin is done"
        );
        type CallbackParam<F> = ChildKillablePointer<F>;

        unsafe extern "C" fn trampoline<F>(
            param: *mut std::ffi::c_void,
            name: aviutl2_sys::common::LPCWSTR,
        ) where
            F: FnMut(String),
        {
            let callback = unsafe { &mut *(param as *mut CallbackParam<F>) };
            let callback = unsafe { callback.as_mut() };
            let name_str = unsafe { crate::common::load_wide_string(name) };
            callback(name_str);
        }

        let trampoline_static = trampoline::<F>
            as unsafe extern "C" fn(*mut std::ffi::c_void, aviutl2_sys::common::LPCWSTR);
        let callback_guard = KillablePointer::new(callback);
        let child_param = callback_guard.create_child();
        let param = Box::new(child_param);
        let param_ptr = Box::into_raw(param);
        unsafe {
            ((*self.internal).enum_palette_name)(
                param_ptr as *mut std::ffi::c_void,
                trampoline_static,
            );
        }
        drop(unsafe { Box::from_raw(param_ptr) });
    }

    /// パレット名の一覧を取得する。
    pub fn get_palette_names(&self) -> Vec<String> {
        assert!(
            self.is_ready(),
            "get_palette_names cannot be called before register_plugin is done"
        );
        let mut palette_names = Vec::new();
        self.enumerate_palette_names(|name| {
            palette_names.push(name);
        });
        palette_names
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

/// エフェクトの設定項目情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectItemInfo {
    /// 設定項目名。
    pub name: String,
    /// 設定項目種別。
    pub item_type: EffectItemType,
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
    /// オブジェクト制御。
    Control,
    /// メディア出力。
    Output,
}

/// 設定項目種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectItemType {
    /// 整数。
    Integer,
    /// 数値。
    Number,
    /// チェックボックス。
    Check,
    /// テキスト。
    Text,
    /// 文字列。
    String,
    /// ファイル。
    File,
    /// 色。
    Color,
    /// リスト選択。
    Select,
    /// シーン。
    Scene,
    /// レイヤー範囲。
    Range,
    /// リストと文字の複合。
    Combo,
    /// マスク。
    Mask,
    /// フォント。
    Font,
    /// 図形。
    Figure,
    /// データ。
    Data,
    /// フォルダ。
    Folder,
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

        /// カメラ効果をサポートするかどうか。
        camera: aviutl2_sys::plugin2::EDIT_HANDLE::EFFECT_FLAG_CAMERA,
    }
}

impl TryFrom<i32> for EffectType {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(EffectType::Filter),
            2 => Ok(EffectType::Input),
            3 => Ok(EffectType::SceneChange),
            4 => Ok(EffectType::Control),
            5 => Ok(EffectType::Output),
            _ => Err(()),
        }
    }
}
impl From<EffectType> for i32 {
    fn from(value: EffectType) -> Self {
        match value {
            EffectType::Filter => 1,
            EffectType::Input => 2,
            EffectType::SceneChange => 3,
            EffectType::Control => 4,
            EffectType::Output => 5,
        }
    }
}

impl TryFrom<i32> for EffectItemType {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(EffectItemType::Integer),
            2 => Ok(EffectItemType::Number),
            3 => Ok(EffectItemType::Check),
            4 => Ok(EffectItemType::Text),
            5 => Ok(EffectItemType::String),
            6 => Ok(EffectItemType::File),
            7 => Ok(EffectItemType::Color),
            8 => Ok(EffectItemType::Select),
            9 => Ok(EffectItemType::Scene),
            10 => Ok(EffectItemType::Range),
            11 => Ok(EffectItemType::Combo),
            12 => Ok(EffectItemType::Mask),
            13 => Ok(EffectItemType::Font),
            14 => Ok(EffectItemType::Figure),
            15 => Ok(EffectItemType::Data),
            16 => Ok(EffectItemType::Folder),
            _ => Err(()),
        }
    }
}
impl From<EffectItemType> for i32 {
    fn from(value: EffectItemType) -> Self {
        match value {
            EffectItemType::Integer => 1,
            EffectItemType::Number => 2,
            EffectItemType::Check => 3,
            EffectItemType::Text => 4,
            EffectItemType::String => 5,
            EffectItemType::File => 6,
            EffectItemType::Color => 7,
            EffectItemType::Select => 8,
            EffectItemType::Scene => 9,
            EffectItemType::Range => 10,
            EffectItemType::Combo => 11,
            EffectItemType::Mask => 12,
            EffectItemType::Font => 13,
            EffectItemType::Figure => 14,
            EffectItemType::Data => 15,
            EffectItemType::Folder => 16,
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
}

impl TryFrom<i32> for ModuleType {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ModuleType::ScriptFilter),
            2 => Ok(ModuleType::ScriptObject),
            3 => Ok(ModuleType::ScriptCamera),
            4 => Ok(ModuleType::ScriptTrack),
            5 => Ok(ModuleType::ScriptModule),
            6 => Ok(ModuleType::PluginInput),
            7 => Ok(ModuleType::PluginOutput),
            8 => Ok(ModuleType::PluginFilter),
            9 => Ok(ModuleType::PluginGeneric),
            _ => Err(()),
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
        }
    }
}

/// 編集状態。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditState {
    /// 編集中
    Edit,
    /// プレビュー再生中
    Preview,
    /// ファイル出力中
    Save,
}

impl TryFrom<i32> for EditState {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EditState::Edit),
            1 => Ok(EditState::Preview),
            2 => Ok(EditState::Save),
            _ => Err(()),
        }
    }
}
impl From<EditState> for i32 {
    fn from(value: EditState) -> Self {
        match value {
            EditState::Edit => 0,
            EditState::Preview => 1,
            EditState::Save => 2,
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
            .map_err(|_| tracing::warn!("GlobalEditHandle was already initialized"));
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

fn effect_item_info_from_raw(name: String, item_type: i32) -> Option<EffectItemInfo> {
    if let Ok(item_type) = EffectItemType::try_from(item_type) {
        Some(EffectItemInfo { name, item_type })
    } else {
        tracing::warn!("Unknown effect item type: {}", item_type);
        None
    }
}

fn module_info_from_raw(raw: *mut aviutl2_sys::plugin2::MODULE_INFO) -> Option<ModuleInfo> {
    let module_type = unsafe { (*raw).r#type };
    if let Ok(module_type) = ModuleType::try_from(module_type) {
        Some(ModuleInfo {
            module_type,
            name: unsafe { crate::common::load_wide_string((*raw).name) },
            information: unsafe { crate::common::load_wide_string((*raw).information) },
        })
    } else {
        tracing::warn!("Unknown module type: {}", module_type);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_item_type_try_from_known_values() {
        assert_eq!(EffectItemType::try_from(1), Ok(EffectItemType::Integer));
        assert_eq!(EffectItemType::try_from(8), Ok(EffectItemType::Select));
        assert_eq!(EffectItemType::try_from(16), Ok(EffectItemType::Folder));
    }

    #[test]
    fn effect_item_type_try_from_unknown_value_fails() {
        assert_eq!(EffectItemType::try_from(999), Err(()));
    }

    #[test]
    fn effect_item_type_into_i32() {
        assert_eq!(i32::from(EffectItemType::Integer), 1);
        assert_eq!(i32::from(EffectItemType::Combo), 11);
        assert_eq!(i32::from(EffectItemType::Folder), 16);
    }

    #[test]
    fn effect_item_info_from_raw_returns_none_for_unknown_type() {
        assert_eq!(effect_item_info_from_raw("test".to_string(), 999), None);
    }

    #[test]
    fn effect_item_info_from_raw_builds_info_for_known_type() {
        assert_eq!(
            effect_item_info_from_raw("test".to_string(), 4),
            Some(EffectItemInfo {
                name: "test".to_string(),
                item_type: EffectItemType::Text,
            })
        );
    }

    #[test]
    fn module_type_try_from_unknown_value_fails() {
        assert_eq!(ModuleType::try_from(999), Err(()));
    }

    #[test]
    fn edit_state_try_from_unknown_value_fails() {
        assert_eq!(EditState::try_from(999), Err(()));
    }
}
