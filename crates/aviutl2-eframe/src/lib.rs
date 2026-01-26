//! # aviutl2-eframe
//!
//! AviUtl2の汎用プラグインでegui/eframeを扱うためのライブラリ。
use anyhow::Context;
use aviutl2::{AnyResult, log, raw_window_handle};
use eframe::egui;
use std::{num::NonZeroIsize, sync::mpsc};
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GWL_STYLE, SetWindowLongPtrW, WS_CLIPSIBLINGS, WS_POPUP,
    },
};
use winit::{platform::windows::EventLoopBuilderExtWindows, raw_window_handle::HasWindowHandle};

/// eframeのウィンドウを表す構造体。
///
/// この構造体は、別スレッドで動作するegui/eframeウィンドウを管理します。
/// ウィンドウのハンドル（HWND）やeguiのコンテキストへのアクセスを提供します。
pub struct EframeWindow {
    hwnd: NonZeroIsize,
    thread: Option<std::thread::JoinHandle<AnyResult<()>>>,
    egui_ctx: egui::Context,
}

struct WrappedApp {
    hwnd: NonZeroIsize,
    internal_app: Box<dyn eframe::App>,
}

impl eframe::App for WrappedApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.internal_app.update(ctx, frame);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.internal_app.save(storage);
    }

    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        self.internal_app.on_exit(gl);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        self.internal_app.auto_save_interval()
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        self.internal_app.clear_color(visuals)
    }

    fn persist_egui_memory(&self) -> bool {
        self.internal_app.persist_egui_memory()
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        // なぜかフォーカスがされないので、クリックされたら強制的にフォーカスを当てる
        // （こうしないとキーボードが使えない）
        if !raw_input.focused {
            let is_clicked = raw_input
                .events
                .iter()
                .any(|event| matches!(event, egui::Event::PointerButton { .. }));
            if is_clicked {
                log::trace!("Egui window clicked while unfocused, forcing focus");
                let focus_result = unsafe {
                    windows::Win32::UI::Input::KeyboardAndMouse::SetFocus(Some(HWND(
                        self.hwnd.get() as *mut std::ffi::c_void,
                    )))
                };
                match focus_result {
                    Ok(_) => {
                        log::trace!("SetFocus succeeded");
                    }
                    Err(e) => {
                        log::warn!("SetFocus failed: {:?}", e);
                    }
                }
            }
        }
        self.internal_app.raw_input_hook(ctx, raw_input);
    }
}

impl EframeWindow {
    /// 新しいEframeWindowを作成する。
    ///
    /// `app_creator`は`eframe::run_native`と同様のclosureです。
    pub fn new<F>(app_creator: F) -> AnyResult<Self>
    where
        F: 'static
            + Send
            + FnOnce(
                &eframe::CreationContext<'_>,
            )
                -> Result<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>>,
    {
        let (tx, rx) = mpsc::channel();
        let thread = std::thread::spawn({
            move || {
                let native_options = eframe::NativeOptions {
                    event_loop_builder: Some(Box::new(move |builder| {
                        builder.with_any_thread(true);
                    })),
                    viewport: egui::ViewportBuilder::default()
                        .with_visible(false)
                        .with_icon(egui::IconData::default()),
                    ..Default::default()
                };
                eframe::run_native(
                    "Egui Window",
                    native_options,
                    Box::new(|cc| {
                        let raw_window_handle::RawWindowHandle::Win32(hwnd) = cc
                            .window_handle()
                            .expect("Failed to get window handle")
                            .as_raw()
                        else {
                            unreachable!("Not a Win32 window handle");
                        };
                        unsafe {
                            // Set window styles
                            let hwnd = hwnd.hwnd.get() as _;
                            let res_style = SetWindowLongPtrW(
                                HWND(hwnd),
                                GWL_STYLE,
                                (WS_CLIPSIBLINGS.0 | WS_POPUP.0) as isize,
                            );
                            if res_style == 0 {
                                log::warn!(
                                    "Failed to set window style: {}",
                                    windows::core::Error::from_win32()
                                );
                            }
                            let res_exstyle = SetWindowLongPtrW(HWND(hwnd), GWL_EXSTYLE, 0);
                            if res_exstyle == 0 {
                                log::warn!(
                                    "Failed to set window exstyle: {}",
                                    windows::core::Error::from_win32()
                                );
                            }
                        }
                        tx.send((hwnd.hwnd.get(), cc.egui_ctx.clone()))
                            .context("Failed to send HWND")?;
                        let app = app_creator(cc)?;
                        log::debug!("Egui app created, with HWND: 0x{:016x}", hwnd.hwnd);
                        Ok(Box::new(WrappedApp {
                            hwnd: NonZeroIsize::new(hwnd.hwnd.get()).context("HWND is null")?,
                            internal_app: app,
                        }) as Box<dyn eframe::App>)
                    }),
                )
                .map_err(|e| anyhow::anyhow!("Eframe error: {}", e))
            }
        });
        let (hwnd, egui_ctx) = rx
            .recv()
            .context("Failed to receive HWND from Egui thread")?;

        let hwnd = NonZeroIsize::new(hwnd).context("Received null HWND from Egui thread")?;

        Ok(Self {
            hwnd,
            thread: Some(thread),
            egui_ctx,
        })
    }

    /// eguiのコンテキストへの参照を取得する。
    pub fn egui_ctx(&self) -> &egui::Context {
        &self.egui_ctx
    }
}

impl Drop for EframeWindow {
    fn drop(&mut self) {
        // ウィンドウスレッドが終了するのを待つ
        if let Some(thread) = self.thread.take() {
            self.egui_ctx
                .send_viewport_cmd(egui::ViewportCommand::Close);
            self.egui_ctx.request_repaint(); // ウィンドウを閉じるトリガー
            let _ = thread.join();
        }
    }
}

impl raw_window_handle::HasWindowHandle for EframeWindow {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        let handle = raw_window_handle::Win32WindowHandle::new(self.hwnd);
        Ok(unsafe {
            raw_window_handle::WindowHandle::borrow_raw(raw_window_handle::RawWindowHandle::Win32(
                handle,
            ))
        })
    }
}
