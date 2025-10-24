use crate::common::{AnyResult, AviUtl2Info, FileFilter, Rational32, Win32WindowHandle, Yc48, f16};
use zerocopy::IntoBytes;

/// スクリプトモジュールプラグインの情報を表す構造体。
#[derive(Debug, Clone)]
pub struct ScriptModuleTable {
    /// プラグインの情報。
    /// 「プラグイン情報」ダイアログで表示されます。
    pub information: String,

    /// プラグインが提供する関数。
    pub functions: Vec<ModuleFunction>,
}

/// スクリプトモジュールプラグインの関数を表す構造体。
#[derive(Debug, Clone)]
pub struct ModuleFunction {
    /// 関数名。
    pub name: String,
    /// 関数の実装。
    pub func: fn(&crate::module::ScriptModuleParam),
}

/// 関数のパラメータを表す構造体。
#[derive(Debug, Clone)]
pub struct ScriptModuleParam {
    pub(crate) internal: *mut aviutl2_sys::module2::SCRIPT_MODULE_PARAM,
}

/// スクリプトモジュールの関数一覧を返すトレイト。
///
/// <div class="warning">
///
/// このトレイトは[`aviutl2::module::functions`]マクロで実装してください。
/// 手動で実装しないでください。
///
/// </div>
pub trait ScriptModuleFunctions: Sized + Send + Sync + 'static {
    /// プラグインが提供する関数の一覧を返す。
    fn functions() -> Vec<ModuleFunction>;

    #[doc(hidden)]
    fn __internal_setup_pluin_handle(
        handle: std::sync::Arc<
            std::sync::RwLock<Option<crate::module::__bridge::InternalScriptModuleState<Self>>>,
        >,
    ) where
        Self: ScriptModule;

    #[doc(hidden)]
    fn __internal_get_plugin_handle(
        handle: &std::sync::Arc<
            std::sync::RwLock<Option<crate::module::__bridge::InternalScriptModuleState<Self>>>,
        >,
    ) -> std::sync::Arc<
        std::sync::RwLock<Option<crate::module::__bridge::InternalScriptModuleState<Self>>>,
    >
    where
        Self: ScriptModule;
}

/// スクリプトモジュールプラグインのトレイト。
/// このトレイトを実装し、[`crate::register_module_plugin!`] マクロを使用してプラグインを登録します。
pub trait ScriptModule: Sized + Send + Sync + 'static + ScriptModuleFunctions {
    /// プラグインを初期化する。
    fn new(info: AviUtl2Info) -> AnyResult<Self>;

    /// プラグインの情報を返す。
    fn plugin_info(&self) -> ScriptModuleTable;
}
