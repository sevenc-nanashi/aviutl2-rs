use aviutl2::{
    AnyResult,
    module::{IntoScriptModuleReturnValue, ScriptModuleFunctions},
};

#[aviutl2::plugin(ScriptModule)]
struct UsernameModule;

impl aviutl2::module::ScriptModule for UsernameModule {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Ok(UsernameModule)
    }

    fn plugin_info(&self) -> aviutl2::module::ScriptModuleTable {
        aviutl2::module::ScriptModuleTable {
            information: format!(
                "User name query for AviUtl, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/username-module",
                version = env!("CARGO_PKG_VERSION")
            ),
            functions: Self::functions(),
        }
    }
}

#[aviutl2::module::functions]
impl UsernameModule {
    fn get_username(&self) -> String {
        whoami::username()
    }

    fn sum(a: i32, b: i32) -> i32 {
        a + b
    }

    fn return_overload(a: i32) -> impl aviutl2::module::IntoScriptModuleReturnValue {
        if a % 2 == 0 {
            return "Even".into_return_values().map_err(anyhow::Error::from);
        } else {
            return ("Odd", a).into_return_values();
        }
    }

    #[direct]
    fn direct_sum(params: &mut aviutl2::module::ScriptModuleCallHandle) {
        let a: i32 = params.get_param(0).unwrap_or(0);
        let b: i32 = params.get_param(1).unwrap_or(0);
        params.push_result(a + b);
    }
}

aviutl2::register_script_module!(UsernameModule);
