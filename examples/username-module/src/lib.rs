use aviutl2::{AnyResult, module::ScriptModuleFunctions};

struct UsernameModule;

struct MyModule;
#[aviutl2::module::functions]
impl MyModule {
    fn sum(a: i32, b: i32) -> i32 {
        a + b
    }

    #[direct]
    fn direct_sum(params: &aviutl2::module::ScriptModuleCallHandle) {
        let a: i32 = params.get_param(0).unwrap_or(0);
        let b: i32 = params.get_param(1).unwrap_or(0);
        params.push_result(&(a + b));
    }
}

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
}

aviutl2::register_module_plugin!(UsernameModule);
