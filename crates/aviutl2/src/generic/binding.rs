use std::{borrow::Cow, num::NonZero};

use crate::{
    common::{AnyResult, AviUtl2Info, FileFilter, Rational32, Win32WindowHandle, Yc48, f16},
    generic::HostAppHandle,
};

use zerocopy::IntoBytes;

/// ホストアプリケーション構造体
#[derive(Debug)]
pub struct HostAppTable {
    pub(crate) internal: *mut aviutl2_sys::plugin2::HOST_APP_TABLE,
}

/// 汎用プラグインの情報を表す構造体。
#[derive(Debug, Clone)]
pub struct FilterPluginTable {
    /// プラグインの情報。
    /// 「プラグイン情報」ダイアログで表示されます。
    pub information: String,
}

/// 汎用プラグインのトレイト。
/// このトレイトを実装し、[`crate::register_generic_plugin!`] マクロを使用してプラグインを登録します。
pub trait GenericPlugin: Send + Sync + Sized {
    /// プラグインを初期化する。
    fn new(info: AviUtl2Info) -> AnyResult<Self>;

    /// プラグインをホストに登録する。
    fn register(&self, registry: &mut HostAppHandle);

    /// シングルトンインスタンスを参照するためのヘルパーメソッド。
    ///
    /// # Panics
    ///
    /// プラグインが初期化されていない場合や、二重に呼び出された場合にパニックします。
    fn with_instance<R>(f: impl FnOnce(&Self) -> R) -> R
    where
        Self: crate::generic::__bridge::GenericSingleton,
    {
        <Self as crate::generic::__bridge::GenericSingleton>::with_instance(f)
    }

    /// シングルトンインスタンスを可変参照するためのヘルパーメソッド。
    ///
    /// # Panics
    ///
    /// プラグインが初期化されていない場合や、二重に呼び出された場合にパニックします。
    fn with_instance_mut<R>(f: impl FnOnce(&mut Self) -> R) -> R
    where
        Self: crate::generic::__bridge::GenericSingleton,
    {
        <Self as crate::generic::__bridge::GenericSingleton>::with_instance_mut(f)
    }
}

/// 編集ハンドル。
#[derive(Debug)]
pub struct EditHandle {
    pub(crate) internal: *mut aviutl2_sys::plugin2::EDIT_HANDLE,
}

unsafe impl Send for EditHandle {}
unsafe impl Sync for EditHandle {}

impl EditHandle {
    pub(crate) unsafe fn new(internal: *mut aviutl2_sys::plugin2::EDIT_HANDLE) -> Self {
        Self { internal }
    }

    /// プロジェクトデータの編集を開始します。
    pub fn call_edit_section<T, F>(&self, callback: F) -> AnyResult<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut EditSection) -> T + Send + 'static,
    {
        type TrampolineCallback =
            dyn FnOnce(&mut EditSection) -> Box<dyn std::any::Any + Send> + Send;
        static NEXT_CALLBACK: std::sync::Mutex<Option<Box<TrampolineCallback>>> =
            std::sync::Mutex::new(None);

        static CALLBACK_RETURN_VALUE: std::sync::Mutex<Option<Box<dyn std::any::Any + Send>>> =
            std::sync::Mutex::new(None);
        {
            let mut guard = NEXT_CALLBACK.lock().unwrap();
            *guard = Some(Box::new(move |section: &mut EditSection| {
                let result = callback(section);
                Box::new(result) as Box<dyn std::any::Any + Send>
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
            anyhow::bail!("call_edit_section failed")
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

/// オブジェクトへのハンドル。
#[derive(Debug, Clone, Copy)]
pub struct ObjectHandle {
    pub(crate) internal: aviutl2_sys::plugin2::OBJECT_HANDLE,
}
impl From<aviutl2_sys::plugin2::OBJECT_HANDLE> for ObjectHandle {
    fn from(value: aviutl2_sys::plugin2::OBJECT_HANDLE) -> Self {
        Self { internal: value }
    }
}
impl From<ObjectHandle> for aviutl2_sys::plugin2::OBJECT_HANDLE {
    fn from(value: ObjectHandle) -> Self {
        value.internal
    }
}
unsafe impl Send for ObjectHandle {}
unsafe impl Sync for ObjectHandle {}

/// 編集情報構造体。
///
/// # Note
///
/// UI表示と異なり、フレーム番号・レイヤー番号は0始まりです。
#[derive(Debug, Clone, Copy)]
pub struct EditInfo {
    /// シーンの幅（ピクセル単位）。
    pub width: usize,
    /// シーンの高さ（ピクセル単位）。
    pub height: usize,
    /// シーンのフレームレート。
    pub fps: Rational32,
    /// シーンのサンプリングレート。
    pub sample_rate: usize,
    /// 現在のカーソルのフレーム番号。
    pub frame: usize,
    /// 現在のカーソルのレイヤー番号。
    pub layer: usize,
    /// オブジェクトが存在する最大フレーム番号。
    pub frame_max: usize,
    /// オブジェクトが存在する最大レイヤー番号。
    pub layer_max: usize,
}

impl EditInfo {
    /// 生ポインタから `EditInfo` を作成します。
    ///
    /// # Safety
    ///
    /// 有効な `EDIT_INFO` ポインタである必要があります。
    pub unsafe fn from_ptr(ptr: *const aviutl2_sys::plugin2::EDIT_INFO) -> Self {
        let raw = unsafe { &*ptr };
        Self {
            width: raw.width as usize,
            height: raw.height as usize,
            fps: Rational32::new(raw.rate, raw.scale),
            sample_rate: raw.sample_rate as usize,
            frame: raw.frame as usize,
            layer: raw.layer as usize,
            frame_max: raw.frame_max as usize,
            layer_max: raw.layer_max as usize,
        }
    }
}

/// オブジェクトのレイヤーとフレーム情報。
#[derive(Debug, Clone, Copy)]
pub struct ObjectLayerFrame {
    /// オブジェクトのレイヤー番号。
    pub layer: usize,
    /// オブジェクトの開始フレーム番号。
    pub start: usize,
    /// オブジェクトの終了フレーム番号。
    pub end: usize,
}

/// 編集セクションのハンドル。
#[derive(Debug)]
pub struct EditSection {
    pub info: EditInfo,

    pub(crate) internal: *mut aviutl2_sys::plugin2::EDIT_SECTION,
}

impl EditSection {
    /// 生ポインタから `EditSection` を作成します。
    ///
    /// # Safety
    ///
    /// 有効な `EDIT_SECTION` ポインタである必要があります。
    pub unsafe fn from_ptr(ptr: *mut aviutl2_sys::plugin2::EDIT_SECTION) -> Self {
        Self {
            internal: ptr,
            info: unsafe { EditInfo::from_ptr((*ptr).info) },
        }
    }

    /// オブジェクトエイリアスから指定の位置にオブジェクトを作成します。
    ///
    /// # Arguments
    ///
    /// - `alias`：オブジェクトエイリアスのデータ。オブジェクトエイリアスと同じフォーマットで指定します。
    /// - `layer`：作成するオブジェクトのレイヤー番号（0始まり）。
    /// - `frame`：作成するオブジェクトのフレーム番号（0始まり）。
    /// - `length`：作成するオブジェクトの長さ（フレーム数）。
    ///
    /// # Errors
    ///
    /// エイリアスの変換に失敗した場合、またはオブジェクトが既存のオブジェクトに重なる場合にエラーを返します。
    ///
    pub fn create_object_from_alias(
        &self,
        alias: &str,
        layer: usize,
        frame: usize,
        length: usize,
    ) -> AnyResult<ObjectHandle> {
        let c_alias = std::ffi::CString::new(alias)?;
        let object_handle = unsafe {
            ((*self.internal).create_object_from_alias)(
                c_alias.as_ptr(),
                layer.try_into()?,
                frame.try_into()?,
                length.try_into()?,
            )
        };
        if object_handle.is_null() {
            anyhow::bail!("Failed to create object from alias");
        }
        Ok(ObjectHandle {
            internal: object_handle,
        })
    }

    /// 指定のフレーム番号以降にあるオブジェクトを検索します。
    ///
    /// # Arguments
    ///
    /// - `layer`：検索するレイヤー番号（0始まり）。
    /// - `frame`：検索を開始するフレーム番号（0始まり）。
    pub fn find_object_after(&self, layer: usize, frame: usize) -> AnyResult<Option<ObjectHandle>> {
        let object_handle =
            unsafe { ((*self.internal).find_object)(layer.try_into()?, frame.try_into()?) };
        if object_handle.is_null() {
            Ok(None)
        } else {
            Ok(Some(ObjectHandle {
                internal: object_handle,
            }))
        }
    }

    /// 指定のオブジェクトのレイヤーとフレーム情報を取得します。
    pub fn get_object_layer_frame(&self, object: &ObjectHandle) -> AnyResult<ObjectLayerFrame> {
        self.ensure_object_exists(object)?;
        let object = unsafe { ((*self.internal).get_object_layer_frame)(object.internal) };
        Ok(ObjectLayerFrame {
            layer: object.layer.try_into()?,
            start: object.start.try_into()?,
            end: object.end.try_into()?,
        })
    }

    /// オブジェクトの情報をエイリアスデータとして取得します。
    pub fn get_object_alias(&self, object: &ObjectHandle) -> AnyResult<String> {
        self.ensure_object_exists(object)?;
        let alias_ptr = unsafe { ((*self.internal).get_object_alias)(object.internal) };
        if alias_ptr.is_null() {
            anyhow::bail!("Failed to get object alias");
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(alias_ptr) };
        let alias = c_str.to_str()?.to_owned();
        Ok(alias)
    }

    /// オブジェクトの設定項目の値を文字列で取得します。
    ///
    /// # Arguments
    ///
    /// - `object`：対象のオブジェクトハンドル。
    /// - `effect_name`：設定項目の名前。
    /// - `effect_index`：同じ名前の設定項目が複数ある場合のインデックス（0始まり）。
    /// - `item`：設定項目の名前。（エイリアスファイルのキーの名前）
    pub fn get_object_effect_item(
        &self,
        object: &ObjectHandle,
        effect_name: &str,
        effect_index: usize,
        item: &str,
    ) -> AnyResult<String> {
        self.ensure_object_exists(object)?;
        let c_effect_name = crate::common::CWString::new(&format!("{effect_name}:{effect_index}"))?;
        let c_item = crate::common::CWString::new(item)?;
        let value_ptr = unsafe {
            ((*self.internal).get_object_item_value)(
                object.internal,
                c_effect_name.as_ptr(),
                c_item.as_ptr(),
            )
        };
        if value_ptr.is_null() {
            anyhow::bail!("Failed to get object effect item");
        }
        let c_str = unsafe { std::ffi::CStr::from_ptr(value_ptr) };
        let value = c_str.to_str()?.to_owned();
        Ok(value)
    }

    /// オブジェクトの設定項目の値を文字列で設定します。
    ///
    /// # Arguments
    ///
    /// - `object`：対象のオブジェクトハンドル。
    /// - `effect_name`：設定項目の名前。
    /// - `effect_index`：同じ名前の設定項目が複数ある場合のインデックス（0始まり）。
    /// - `item`：設定項目の名前。（エイリアスファイルのキーの名前）
    /// - `value`：設定する値。
    pub fn set_object_effect_item(
        &self,
        object: &ObjectHandle,
        effect_name: &str,
        effect_index: usize,
        item: &str,
        value: &str,
    ) -> AnyResult<()> {
        self.ensure_object_exists(object)?;
        let c_effect_name = crate::common::CWString::new(&format!("{effect_name}:{effect_index}"))?;
        let c_item = crate::common::CWString::new(item)?;
        let c_value = std::ffi::CString::new(value)?;
        unsafe {
            ((*self.internal).set_object_item_value)(
                object.internal,
                c_effect_name.as_ptr(),
                c_item.as_ptr(),
                c_value.as_ptr(),
            )
        };
        Ok(())
    }

    /// オブジェクトを移動します。
    pub fn move_object(
        &self,
        object: &ObjectHandle,
        new_layer: usize,
        new_start_frame: usize,
    ) -> AnyResult<()> {
        self.ensure_object_exists(object)?;
        let success = unsafe {
            ((*self.internal).move_object)(
                object.internal,
                new_layer.try_into()?,
                new_start_frame.try_into()?,
            )
        };
        if !success {
            anyhow::bail!("Failed to move object");
        }
        Ok(())
    }

    /// オブジェクトを削除します。
    pub fn delete_object(&self, object: &ObjectHandle) -> AnyResult<()> {
        self.ensure_object_exists(object)?;
        unsafe { ((*self.internal).delete_object)(object.internal) };
        Ok(())
    }

    /// 現在、オブジェクト設定ウィンドウで選択されているオブジェクトを取得します。
    pub fn get_focused_object(&self) -> AnyResult<Option<ObjectHandle>> {
        let object_handle = unsafe { ((*self.internal).get_focus_object)() };
        if object_handle.is_null() {
            Ok(None)
        } else {
            Ok(Some(ObjectHandle {
                internal: object_handle,
            }))
        }
    }

    /// オブジェクト設定ウィンドウで指定のオブジェクトを選択状態にします。
    ///
    /// # Note
    ///
    /// コールバック処理の終了時に設定されます。
    pub fn focus_object(&self, object: &ObjectHandle) -> AnyResult<()> {
        self.ensure_object_exists(object)?;
        unsafe { ((*self.internal).set_focus_object)(object.internal) };
        Ok(())
    }

    /// ログにメッセージを出力します。
    pub fn output_log(&self, message: &str) -> AnyResult<()> {
        let wide = crate::common::CWString::new(message)?;
        unsafe { ((*self.internal).output_log)(wide.as_ptr()) };
        Ok(())
    }

    /// オブジェクトが存在するかどうか調べます。
    ///
    /// # Note
    ///
    /// 内部的には、get_object_layer_frame を呼び出してレイヤー番号が -1 でないかを確認しています。
    pub fn object_exists(&self, object: &ObjectHandle) -> bool {
        let object = unsafe { ((*self.internal).get_object_layer_frame)(object.internal) };
        object.layer != -1
    }

    fn ensure_object_exists(&self, object: &ObjectHandle) -> AnyResult<()> {
        if !self.object_exists(object) {
            anyhow::bail!("Object does not exist in the current edit section");
        }
        Ok(())
    }

    #[doc(hidden)]
    #[expect(private_bounds)]
    pub fn __output_log_if_error<T: MenuCallbackReturn>(&self, result: T) {
        if let Some(err_msg) = result.into_optional_error() {
            let _ = self.output_log(&err_msg);
        }
    }
}

trait MenuCallbackReturn {
    fn into_optional_error(self) -> Option<String>;
}
impl<E> MenuCallbackReturn for Result<(), E>
where
    Box<dyn std::error::Error>: From<E>,
{
    fn into_optional_error(self) -> Option<String> {
        match self {
            Ok(_) => None,
            Err(e) => {
                let boxed: Box<dyn std::error::Error> = e.into();
                Some(format!("{}", boxed))
            }
        }
    }
}
impl MenuCallbackReturn for () {
    fn into_optional_error(self) -> Option<String> {
        None
    }
}
