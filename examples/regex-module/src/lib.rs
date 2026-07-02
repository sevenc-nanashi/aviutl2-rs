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
    regex: regex::Regex,
}
#[aviutl2::module::metatable]
impl RegexUserData {
    fn __index(
        &self,
        _this: (),
        index: String,
    ) -> aviutl2::AnyResult<Option<aviutl2::module::ScriptModuleFunctionCallback>> {
        aviutl2::lprintln!("__index called with index: {}", index);
        match index.as_str() {
            "is_match" => Ok(Some(aviutl2::module::script_module_callback!(
                |text: String| { self.is_match(text) }
            ))),
            _ => Ok(None),
        }
    }
}
impl RegexUserData {
    fn is_match(&self, text: String) -> aviutl2::AnyResult<bool> {
        Ok(self.regex.is_match(&text))
    }
    fn find(&self, text: String) -> aviutl2::AnyResult<Option<(usize, usize)>> {
        Ok(self.regex.find(&text).map(|m| (m.start(), m.end())))
    }
    fn replace(&self, text: String, replacement: String) -> aviutl2::AnyResult<String> {
        Ok(self
            .regex
            .replace_all(&text, replacement.as_str())
            .to_string())
    }
    fn split(&self, text: String) -> aviutl2::AnyResult<Vec<String>> {
        Ok(self.regex.split(&text).map(|s| s.to_string()).collect())
    }
    fn captures(&self, text: String) -> aviutl2::AnyResult<Vec<String>> {
        Ok(self.regex.captures(&text).map_or(vec![], |caps| {
            caps.iter()
                .map(|m| m.map(|m| m.as_str().to_string()).unwrap_or_default())
                .collect()
        }))
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
            regex,
        }))
    }
}

aviutl2::register_script_module!(RegexModule);
