use std::borrow::Cow;

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
    pub(crate) unsafe fn new(
        internal: *mut aviutl2_sys::plugin2::EDIT_HANDLE,
    ) -> Self {
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

/// 編集セクションのハンドル。
#[derive(Debug)]
pub struct EditSection {
    pub(crate) internal: *mut aviutl2_sys::plugin2::EDIT_SECTION,
}

impl EditSection {
    /// 生ポインタから `EditSection` を作成します。
    ///
    /// # Safety
    ///
    /// 有効な `EDIT_SECTION` ポインタである必要があります。
    pub unsafe fn from_ptr(ptr: *mut aviutl2_sys::plugin2::EDIT_SECTION) -> Self {
        Self { internal: ptr }
    }
}
