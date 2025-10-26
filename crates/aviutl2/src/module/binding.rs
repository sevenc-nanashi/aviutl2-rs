use crate::common::{AnyResult, AviUtl2Info};

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
    pub func: extern "C" fn(*mut crate::sys::module2::SCRIPT_MODULE_PARAM),
}

pub use aviutl2_macros::module_functions as functions;

/// スクリプトモジュールの関数一覧を返すトレイト。
///
/// <div class="warning">
///
/// このトレイトは[`functions`]マクロで実装してください。
/// 手動で実装しないでください。
///
/// </div>
pub trait ScriptModuleFunctions: Sized + Send + Sync + 'static {
    /// プラグインが提供する関数の一覧を返す。
    fn functions() -> Vec<ModuleFunction>;

    #[doc(hidden)]
    fn __internal_setup_plugin_handle(
        handle: std::sync::Arc<
            std::sync::RwLock<Option<crate::module::__bridge::InternalScriptModuleState<Self>>>,
        >,
    ) where
        Self: ScriptModule;

    #[doc(hidden)]
    fn __internal_get_plugin_handle() -> std::sync::Arc<
        std::sync::RwLock<Option<crate::module::__bridge::InternalScriptModuleState<Self>>>,
    >
    where
        Self: ScriptModule;
}

/// スクリプトモジュールプラグインのトレイト。
/// このトレイトを実装し、[`crate::register_script_module!`] マクロを使用してプラグインを登録します。
pub trait ScriptModule: Sized + Send + Sync + 'static + ScriptModuleFunctions {
    /// プラグインを初期化する。
    fn new(info: AviUtl2Info) -> AnyResult<Self>;

    /// プラグインの情報を返す。
    fn plugin_info(&self) -> ScriptModuleTable;
}
