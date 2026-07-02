use aviutl2::{AnyResult, module::ScriptModuleFunctions};

#[aviutl2::plugin(ScriptModule)]
struct RegexModule;

impl aviutl2::module::ScriptModule for RegexModule {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Ok(RegexModule)
    }

    fn plugin_info(&self) -> aviutl2::module::ScriptModuleTable {
        aviutl2::module::ScriptModuleTable {
            information: format!(
                "Regex for AviUtl2, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/regex-module",
                version = env!("CARGO_PKG_VERSION")
            ),
            functions: Self::functions(),
        }
    }
}

#[derive(Debug, Clone)]
struct RegexUserData {
    regex: std::sync::Arc<regex::Regex>,
}
#[aviutl2::module::metatable]
impl RegexUserData {
    fn index(
        &self,
        _this: (),
        index: String,
    ) -> aviutl2::AnyResult<Option<aviutl2::module::ScriptModuleFunctionCallback>> {
        aviutl2::lprintln!("__index called with index: {}", index);
        match index.as_str() {
            "is_match" => Ok(Some({
                // NOTE: コールバックのgcがされないのでメモリリークしない？
                let regex = self.regex.clone();
                aviutl2::module::script_module_callback!(move |text: String| {
                    regex.is_match(&text)
                })
            })),
            _ => Ok(None),
        }
    }
}

#[aviutl2::module::functions]
impl RegexModule {
    fn regex(
        &self,
        pattern: String,
    ) -> aviutl2::AnyResult<aviutl2::module::ScriptModuleUserData<RegexUserData>> {
        let regex = regex::Regex::new(&pattern)
            .map_err(|e| aviutl2::anyhow::anyhow!("Invalid regex pattern: {}", e))?;
        Ok(aviutl2::module::ScriptModuleUserData::new(RegexUserData {
            regex: std::sync::Arc::new(regex),
        }))
    }
}

aviutl2::register_script_module!(RegexModule);
