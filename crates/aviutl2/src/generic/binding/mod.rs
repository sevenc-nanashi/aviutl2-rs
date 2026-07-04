/// 汎用プラグインの情報を表す構造体。
#[derive(Debug, Clone)]
pub struct GenericPluginTable {
    /// プラグインの名前。
    pub name: String,
    /// プラグインの情報。
    /// 「プラグイン情報」ダイアログで表示されます。
    pub information: String,
}

/// 汎用プラグインのトレイト。
/// このトレイトを実装し、[`crate::register_generic_plugin!`] マクロを使用してプラグインを登録します。
pub trait GenericPlugin: Send + Sync + Sized {
    /// プラグインを初期化する。
    fn new(info: crate::common::AviUtl2Info) -> crate::common::AnyResult<Self>;

    /// プラグインの情報を返す。
    fn plugin_info(&self) -> crate::generic::GenericPluginTable;

    /// プラグインをホストに登録する。
    fn register(&mut self, registry: &mut crate::generic::HostAppHandle);

    /// プロジェクトファイルのロードを処理する。
    ///
    /// プロジェクトの初期化時にも呼ばれます。
    fn on_project_load(&mut self, project: &mut crate::generic::ProjectFile) {
        let _ = project;
    }

    /// プロジェクトファイルをセーブする直前に呼ばれる。
    fn on_project_save(&mut self, project: &mut crate::generic::ProjectFile) {
        let _ = project;
    }

    /// 「キャッシュを破棄」が呼ばれたときに呼ばれる。
    fn on_clear_cache(&mut self, edit_section: &crate::generic::EditSection) {
        let _ = edit_section;
    }

    // NOTE:
    // on_change_sceneはAviUtl2内でシーンを編集したときに呼ばれるが、これは同期的に呼ばれてしまう。
    // それにより、GenericPluginを同期ロックしている状態でシーン変更系のイベントを発生すると、デッドロックが発生してしまう。
    // Rustではトレイトに関数を実装したかどうかを判定する手段がないため、on_change_sceneを本当に呼ぶべきかどうかを判定することができない。
    // そのため、一旦GenericPlugin::on_change_sceneを無効化する。
    //
    // /// シーンを変更した直後に呼ばれる。
    // fn on_change_scene(&mut self, edit_section: &crate::generic::EditSection) {
    //     let _ = edit_section;
    // }

    /// オブジェクト情報が更新されたときに呼ばれる。
    ///
    /// # Note
    ///
    /// イベント用スレッドから呼び出されます。[`crate::generic::EditHandle::call_edit_section`]は利用できません。
    fn event_update_object_info(&mut self) {}

    /// フレームを移動した直後に呼ばれる。
    ///
    /// # Note
    ///
    /// イベント用スレッドから呼び出されます。[`crate::generic::EditHandle::call_edit_section`]は利用できません。
    fn event_change_edit_frame(&mut self) {}

    /// シーンを移動、またはシーン情報を変更したときに呼ばれる。
    ///
    /// # Note
    ///
    /// イベント用スレッドから呼び出されます。[`crate::generic::EditHandle::call_edit_section`]は利用できません。
    fn event_change_scene_info(&mut self) {}

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
mod edit_handle;
pub use edit_handle::*;
