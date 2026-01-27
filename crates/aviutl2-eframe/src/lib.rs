//! # aviutl2-eframe
//!
//! AviUtl2の汎用プラグインでegui/eframeを扱うためのライブラリ。
use anyhow::Context;
use aviutl2::{AnyResult, log, raw_window_handle};
use std::{num::NonZeroIsize, sync::mpsc};
use windows::Win32::{
    Foundation::{HWND, SetLastError},
    UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GWL_STYLE, SetWindowLongPtrW, WS_CLIPSIBLINGS, WS_POPUP,
    },
};
use winit::{platform::windows::EventLoopBuilderExtWindows, raw_window_handle::HasWindowHandle};

pub use eframe;
pub use eframe::egui;
pub use windows;

/// eframeのウィンドウを表す構造体。
///
/// この構造体は、別スレッドで動作するegui/eframeウィンドウを管理します。
/// ウィンドウのハンドル（HWND）やeguiのコンテキストへのアクセスを提供します。
pub struct EframeWindow {
    hwnd: NonZeroIsize,
    thread: Option<std::thread::JoinHandle<()>>,
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
    pub fn new<F>(name: &str, app_creator: F) -> AnyResult<Self>
    where
        F: 'static
            + Send
            + FnOnce(
                &eframe::CreationContext<'_>,
                AviUtl2EframeHandle,
            )
                -> Result<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>>,
    {
        let (tx, rx) = mpsc::channel::<
            Result<(isize, egui::Context), Box<dyn std::error::Error + Send + Sync>>,
        >();
        let name = name.to_string();
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
                let result = eframe::run_native(
                    &name,
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
                            SetLastError(windows::Win32::Foundation::WIN32_ERROR(0));
                            let res_style = SetWindowLongPtrW(
                                HWND(hwnd),
                                GWL_STYLE,
                                (WS_CLIPSIBLINGS.0 | WS_POPUP.0) as isize,
                            );
                            if res_style == 0 && windows::Win32::Foundation::GetLastError().0 != 0 {
                                let err = windows::core::Error::from_win32();
                                return Err(anyhow::anyhow!("Failed to set window style: {}", err)
                                    .into_boxed_dyn_error());
                            }

                            SetLastError(windows::Win32::Foundation::WIN32_ERROR(0));
                            let res_exstyle = SetWindowLongPtrW(HWND(hwnd), GWL_EXSTYLE, 0);
                            if res_exstyle == 0 && windows::Win32::Foundation::GetLastError().0 != 0
                            {
                                let err = windows::core::Error::from_win32();
                                return Err(anyhow::anyhow!(
                                    "Failed to set window exstyle: {}",
                                    err
                                )
                                .into_boxed_dyn_error());
                            }
                        }
                        let app_handle = AviUtl2EframeHandle {
                            hwnd: NonZeroIsize::new(hwnd.hwnd.get()).context("HWND is null")?,
                        };
                        let app = app_creator(cc, app_handle)?;
                        tx.send(Ok((hwnd.hwnd.get(), cc.egui_ctx.clone())))
                            .context("Failed to send HWND")?;
                        log::debug!("Egui app created, with HWND: 0x{:016x}", hwnd.hwnd);
                        Ok(Box::new(WrappedApp {
                            hwnd: NonZeroIsize::new(hwnd.hwnd.get()).context("HWND is null")?,
                            internal_app: app,
                        }) as Box<dyn eframe::App>)
                    }),
                );

                if let Err(e) = result {
                    let _ = tx.send(Err(anyhow::anyhow!(
                        "Egui thread encountered an error: {}",
                        e
                    )
                    .into_boxed_dyn_error()));
                }
            }
        });
        let (hwnd, egui_ctx) = rx
            .recv()
            .context("Failed to receive HWND from Egui thread")?
            .map_err(|e| {
                anyhow::anyhow!("Egui thread reported an error during initialization: {}", e)
            })?;

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

/// aviutl2-eframeでウィンドウ内から呼び出される関数のハンドル。
pub struct AviUtl2EframeHandle {
    hwnd: NonZeroIsize,
}
impl raw_window_handle::HasWindowHandle for AviUtl2EframeHandle {
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

impl AviUtl2EframeHandle {
    /// 親ウィンドウのコンテキストメニューを表示する。
    ///
    /// # Note
    ///
    /// 「ウィンドウ配置」メニューを表示するために使用します。
    ///
    /// # Example
    ///
    /// ```rust
    /// # use aviutl2_eframe::{AviUtl2EframeHandle, egui};
    /// # struct YourAppStruct { handle: AviUtl2EframeHandle }
    /// # impl YourAppStruct {
    /// # fn ui(&self, ctx: &egui::Context) {
    /// egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
    ///     let clicked = ui
    ///         .heading("My Plugin")
    ///         .interact(egui::Sense::click());
    ///     if clicked.secondary_clicked() {
    ///         let _ = self.handle.show_context_menu();
    ///     }
    /// });
    /// # }
    /// # }
    /// ```
    pub fn show_context_menu(&self) -> AnyResult<()> {
        let parent_window = unsafe {
            windows::Win32::UI::WindowsAndMessaging::GetParent(HWND(
                self.hwnd.get() as *mut std::ffi::c_void
            ))
        }?;
        unsafe {
            let mut cursor_pos = windows::Win32::Foundation::POINT::default();
            windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut cursor_pos)?;
            windows::Win32::UI::WindowsAndMessaging::PostMessageW(
                Some(parent_window),
                windows::Win32::UI::WindowsAndMessaging::WM_CONTEXTMENU,
                windows::Win32::Foundation::WPARAM(self.hwnd.get() as _),
                windows::Win32::Foundation::LPARAM(makelparam(cursor_pos.x, cursor_pos.y) as _),
            )?;
        }
        Ok(())
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

pub fn aviutl2_visual() -> eframe::egui::Visuals {
    let mut visuals = eframe::egui::Visuals::dark().clone();

    let background = load_color("Background");
    let window_border = load_color("WindowBorder");
    let window_separator = load_color("WindowSeparator");
    let grouping = load_color("Grouping");
    let grouping_hover = load_color("GroupingHover");
    let grouping_select = load_color("GroupingSelect");
    let border = load_color("Border");
    let border_focus = load_color("BorderFocus");
    let border_select = load_color("BorderSelect");
    let text = load_color("Text");
    let text_disable = load_color("TextDisable");
    let text_select = load_color("TextSelect");
    let button_body = load_color("ButtonBody");
    let button_body_hover = load_color("ButtonBodyHover");
    let button_body_press = load_color("ButtonBodyPress");
    let button_body_select = load_color("ButtonBodySelect");

    visuals.override_text_color = Some(text);
    visuals.weak_text_color = Some(text_disable);
    visuals.selection = egui::style::Selection {
        bg_fill: text_select,
        stroke: egui::Stroke::new(1.0, border_select),
    };
    visuals.window_fill = background;
    visuals.window_stroke = egui::Stroke::new(1.0, window_border);
    visuals.panel_fill = background;
    visuals.faint_bg_color = grouping;
    visuals.extreme_bg_color = window_separator;
    visuals.text_edit_bg_color = Some(grouping_hover);
    visuals.code_bg_color = grouping_select;

    visuals.widgets.noninteractive.bg_fill = background;
    visuals.widgets.noninteractive.weak_bg_fill = background;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, text);

    visuals.widgets.inactive.bg_fill = button_body;
    visuals.widgets.inactive.weak_bg_fill = button_body;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text);

    visuals.widgets.hovered.bg_fill = button_body_hover;
    visuals.widgets.hovered.weak_bg_fill = button_body_hover;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, border_focus);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, text);

    visuals.widgets.active.bg_fill = button_body_press;
    visuals.widgets.active.weak_bg_fill = button_body_press;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, border_select);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, text);

    visuals.widgets.open.bg_fill = button_body_select;
    visuals.widgets.open.weak_bg_fill = button_body_select;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, border_select);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, text);

    visuals
}

fn load_color(key: &str) -> egui::Color32 {
    let (r, g, b) = aviutl2::config::get_color_code(key).expect("Key contains null byte");
    egui::Color32::from_rgb(r, g, b)
}
#[inline]
fn makelparam(low: i32, high: i32) -> isize {
    ((high as isize) << 16) | ((low as isize) & 0xFFFF)
}
