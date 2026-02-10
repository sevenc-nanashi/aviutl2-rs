use aviutl2::{AnyResult, module::ScriptModuleFunctions};

#[aviutl2::plugin(ScriptModule)]
struct UsernameModule;

impl aviutl2::module::ScriptModule for UsernameModule {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Ok(UsernameModule)
    }

    fn plugin_info(&self) -> aviutl2::module::ScriptModuleTable {
        aviutl2::module::ScriptModuleTable {
            information: format!(
                "User name query for AviUtl2, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/username-module",
                version = env!("CARGO_PKG_VERSION")
            ),
            functions: Self::functions(),
        }
    }
}

#[aviutl2::module::functions]
impl UsernameModule {
    fn get_username(&self) -> aviutl2::AnyResult<String> {
        Ok(whoami::username()?)
    }
}

aviutl2::register_script_module!(UsernameModule);
