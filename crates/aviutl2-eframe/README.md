# aviutl2-eframe

[![Crates.io](https://img.shields.io/crates/v/aviutl2-eframe.svg)](https://crates.io/crates/aviutl2-eframe)
[![Documentation](https://docs.rs/aviutl2-eframe/badge.svg)](https://docs.rs/aviutl2-eframe)

AviUtl2の汎用プラグインでegui/eframeを扱うためのライブラリ。

## 使い方

```rust
use aviutl2::AnyResult;
use aviutl2_eframe::EguiWindow;
use eframe::egui;

struct MyApp;

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello from Plugin Window!");
        });
    }
}

impl aviutl2::generic::GenericPlugin for MyPlugin {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        let window = aviutl2_eframe::EguiWindow::new(move |cc| {
            Ok(Box::new(MyApp))
        })?;

        Ok(MyPlugin { window })
    }

    fn register(&mut self, registry: &mut aviutl2::generic::HostAppHandle) {
        registry
            .register_window_client("My Plugin", &self.window)
            .unwrap();
    }
}

```

## ライセンス

MIT Licenseで公開されています。
