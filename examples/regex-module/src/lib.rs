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

// TODO: あまりにも冗長すぎるので、こういうのを簡単に書けるようにするマクロを作る

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
    ) -> aviutl2::AnyResult<Option<aviutl2::module::ScriptModuleUserData<RegexCallbackUserData>>>
    {
        match index.as_str() {
            "is_match" => Ok(Some(
                RegexCallbackUserData {
                    regex: self.regex.clone(),
                }
                .into(),
            )),
            _ => Ok(None),
        }
    }
}
#[derive(Debug, Clone)]
struct RegexCallbackUserData {
    regex: std::sync::Arc<regex::Regex>,
}
#[aviutl2::module::metatable]
impl RegexCallbackUserData {
    fn call(&self, _this: (), text: String) -> aviutl2::AnyResult<bool> {
        Ok(self.regex.is_match(&text))
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
