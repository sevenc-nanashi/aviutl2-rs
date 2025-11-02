use std::borrow::Cow;

use crate::{
    common::{AnyResult, AviUtl2Info, FileFilter, Rational32, Win32WindowHandle, Yc48, f16},
    generic::HostAppHandle,
};

use zerocopy::IntoBytes;

/// ホストアプリケーション構造体
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
}

/// 編集セクションのハンドル。
pub struct EditSectionHandle {
    pub(crate) internal: *mut aviutl2_sys::plugin2::EDIT_SECTION,
}

impl EditSectionHandle {
    /// 生ポインタからハンドルを生成します。
    ///
    /// # Safety
    ///
    /// 有効な `EDIT_SECTION` ポインタである必要があります。
    pub unsafe fn from_ptr(ptr: *mut aviutl2_sys::plugin2::EDIT_SECTION) -> Self {
        Self { internal: ptr }
    }
}
