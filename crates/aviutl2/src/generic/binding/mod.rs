use crate::common::{AnyResult, AviUtl2Info};

/// 汎用プラグインのトレイト。
/// このトレイトを実装し、[`crate::register_generic_plugin!`] マクロを使用してプラグインを登録します。
pub trait GenericPlugin: Send + Sync + Sized {
    /// プラグインを初期化する。
    fn new(info: AviUtl2Info) -> AnyResult<Self>;

    /// プラグインをホストに登録する。
    fn register(&mut self, registry: &mut self::host_app::HostAppHandle);

    /// プロジェクトファイルのロードを処理する。
    ///
    /// プロジェクトの初期化時にも呼ばれます。
    fn on_project_load(&mut self, _project: &mut crate::generic::ProjectFile) {}

    /// プロジェクトファイルをセーブする直前に呼ばれる。
    fn on_project_save(&mut self, _project: &mut crate::generic::ProjectFile) {}

    /// 「キャッシュを破棄」が呼ばれたときに呼ばれる。
    fn on_clear_cache(&mut self, _edit_section: &crate::generic::EditSection) {}

    /// シーンを変更した直後に呼ばれる。
    fn on_change_scene(&mut self, _edit_section: &crate::generic::EditSection) {}

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

mod project;
pub use project::*;
mod edit_section;
pub use edit_section::*;
mod host_app;
pub use host_app::*;
