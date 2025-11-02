use crate::common::{AnyResult, AviUtl2Info};

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
    fn register(&self, registry: &mut self::host_app::HostAppHandle);

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

pub mod edit_section;
pub use edit_section::*;
pub mod host_app;
pub use host_app::*;
