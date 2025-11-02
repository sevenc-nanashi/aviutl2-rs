use std::borrow::Cow;

use crate::common::{AnyResult, AviUtl2Info, FileFilter, Rational32, Win32WindowHandle, Yc48, f16};
use zerocopy::IntoBytes;

/// ホストアプリケーション構造体
pub struct HostAppTable {
    pub(crate) internal: *mut aviutl2_sys::plugin2::HOST_APP_TABLE,
}

/// 初期化に使うハンドル
pub struct RegisterHandle<'a> {
    pub(crate) registry: &'a mut crate::generic::registry::PluginRegistry,
    pub(crate) internal: *mut aviutl2_sys::plugin2::HOST_APP_TABLE,
}

/// 汎用プラグインのトレイト。
/// このトレイトを実装し、[`crate::register_host_app_plugin!`] マクロを使用してプラグインを登録します。
pub trait GenericPlugin: Send + Sync + Sized {
    /// プラグインを初期化する。
    fn new(info: AviUtl2Info) -> AnyResult<Self>;

    /// プラグインをホストに登録する。
    fn register(&self, handle: &mut RegisterHandle) -> AnyResult<()>;
}
