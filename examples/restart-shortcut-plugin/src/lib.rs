use aviutl2::AnyResult;

fn tr(text: &str) -> String {
    aviutl2::config::translate(text).unwrap_or_else(|_| text.to_string())
}

#[aviutl2::plugin(GenericPlugin)]
struct RestartHostAppPlugin {
    edit_handle: Option<aviutl2::generic::EditHandle>,
    config: Config,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Config {
    confirm_on_restart: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            confirm_on_restart: true,
        }
    }
}

impl aviutl2::generic::GenericPlugin for RestartHostAppPlugin {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        let config = RestartHostAppPlugin::load_or_default_config()?;
        Ok(Self {
            edit_handle: None,
            config,
        })
    }

    fn register(&mut self, registry: &mut aviutl2::generic::HostAppHandle) {
        registry.set_plugin_information(&format!(
            "Rusty Restart Shortcut Plugin, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/restart-shortcut-plugin",
            version = env!("CARGO_PKG_VERSION")
        ));
        self.edit_handle = Some(registry.create_edit_handle());
        registry.register_menus::<RestartHostAppPlugin>();
    }
}

#[aviutl2::generic::menus]
impl RestartHostAppPlugin {
    #[edit(name = "AviUtl2を再起動")]
    fn restart_menu(&mut self, _edit_section: &mut aviutl2::generic::EditSection) -> AnyResult<()> {
        let Some(edit_handle) = self.edit_handle.as_ref() else {
            aviutl2::anyhow::bail!("編集ハンドルの取得に失敗しました。");
        };
        if !self.config.confirm_on_restart || shift_key_pressed() {
            edit_handle.restart_host_app();
        } else {
            let restart = native_dialog::DialogBuilder::message()
                .set_title(tr("AviUtl2を再起動"))
                .set_text(tr(
                    "AviUtl2を再起動しますか？（Shiftキーを押しながらメニューを選択すると確認なしで再起動します）",
                ))
                .confirm()
                .show()
                .map_err(|e| aviutl2::anyhow::anyhow!(e))?;
            if restart {
                edit_handle.restart_host_app();
            }
        }
        Ok(())
    }

    #[config(name = "[Rusty Restart Shortcut Plugin] 確認ダイアログの有無を切り替え")]
    fn toggle_confirm_on_restart(&mut self, _hwnd: aviutl2::Win32WindowHandle) -> AnyResult<()> {
        self.config.confirm_on_restart = !self.config.confirm_on_restart;
        let state = if self.config.confirm_on_restart {
            tr("有効")
        } else {
            tr("無効")
        };
        let template = tr("再起動時の確認ダイアログ表示設定を「{}」に変更しました。");
        let message = template.replace("{}", &state);
        native_dialog::DialogBuilder::message()
            .set_title(tr("設定を変更しました"))
            .set_text(&message)
            .alert()
            .show()?;
        Ok(())
    }
}

fn config_path() -> std::path::PathBuf {
    process_path::get_dylib_path()
        .unwrap()
        .parent()
        .unwrap()
        .join("rusty_restart_host_app_plugin.json")
}

impl RestartHostAppPlugin {
    fn load_or_default_config() -> AnyResult<Config> {
        let config_path = config_path();
        if config_path.exists() {
            let config_str = std::fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&config_str)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }
}
impl Drop for RestartHostAppPlugin {
    fn drop(&mut self) {
        let config_path = config_path();
        if let Ok(config_str) = serde_json::to_string_pretty(&self.config) {
            let _ = std::fs::write(config_path, config_str);
        }
    }
}

fn shift_key_pressed() -> bool {
    const VK_SHIFT: i32 = 0x10;
    unsafe {
        (windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(VK_SHIFT) & 0x8000u16 as i16)
            != 0
    }
}

aviutl2::register_generic_plugin!(RestartHostAppPlugin);
