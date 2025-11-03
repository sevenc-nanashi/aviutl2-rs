mod ws_popup;
use aviutl2::{AnyResult, odbg, raw_window_handle::HasWindowHandle};
use ws_popup::WsPopup;

#[aviutl2::plugin(GenericPlugin)]
struct LocalAliasPlugin {
    webview: wry::WebView,
    window: WsPopup,
}
unsafe impl Send for LocalAliasPlugin {}
unsafe impl Sync for LocalAliasPlugin {}

impl aviutl2::generic::GenericPlugin for LocalAliasPlugin {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        let window = WsPopup::new("Rusty Local Alias Plugin", (800, 600))?;
        odbg!("Created window with handle: {:?}", window.hwnd());
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("rusty-local-alias-plugin");
        let mut web_context = wry::WebContext::new(Some(cache_dir));
        let webview = wry::WebViewBuilder::new_with_web_context(&mut web_context)
            .with_url("https://sevenc7c.com")
            .build(&window)?;
        Ok(LocalAliasPlugin { webview, window })
    }

    fn register(&mut self, registry: &mut aviutl2::generic::HostAppHandle) {
        registry.set_plugin_information(&format!(
            "Project Local Aliases for AviUtl, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/local-alias-plugin",
            version = env!("CARGO_PKG_VERSION")
        ));
        registry
            .register_window_client("Rusty Local Alias Plugin", &self.window)
            .unwrap();
    }
}

aviutl2::register_generic_plugin!(LocalAliasPlugin);
