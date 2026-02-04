//! # aviutl2-eframe
//!
//! AviUtl2の汎用プラグインでegui/eframeを扱うためのライブラリ。
use anyhow::Context;
use aviutl2::{AnyResult, log, raw_window_handle};
use eframe::EframeWinitApplication;
use std::{num::NonZeroIsize, sync::mpsc};
use windows::Win32::{
    Foundation::{HWND, SetLastError},
    UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GWL_STYLE, SetWindowLongPtrW, ShowWindow, WS_CLIPSIBLINGS, WS_POPUP,
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
    thread_terminator: std::sync::Arc<std::sync::OnceLock<()>>,
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

// # Note
//
// デフォルトのEframeWinitApplicationはAviUtl2のメインスレッドからの終了要求に対して終了しないため、
// 一段階wrapして終了要求を処理するようにする。
// （これによってActiveEventLoopへのアクセスが可能になり、exit()を呼び出せる）
// 少なくとも2026/02/01現在、これで正常に動作しているので、まぁ...
//
// TODO: "You forgot to call destroy() on the egui glow painter. Resources will leak!" を直す

struct WinitEventLoopApp<'a> {
    app: EframeWinitApplication<'a>,
    thread_terminator: std::sync::Arc<std::sync::OnceLock<()>>,
}
impl<'a> WinitEventLoopApp<'a> {
    fn maybe_exit(&self, event_loop: &winit::event_loop::ActiveEventLoop) -> bool {
        if self.thread_terminator.get().is_some() {
            log::debug!("Egui window thread exiting...");
            event_loop.exit();
            return true;
        }
        false
    }
}
impl<'a> winit::application::ApplicationHandler<eframe::UserEvent> for WinitEventLoopApp<'a> {
    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        if self.maybe_exit(event_loop) {
            return;
        }
        self.app.new_events(event_loop, cause);
    }

    fn user_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: eframe::UserEvent,
    ) {
        if self.maybe_exit(event_loop) {
            return;
        }
        self.app.user_event(event_loop, event);
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if self.maybe_exit(event_loop) {
            return;
        }
        self.app.device_event(event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.maybe_exit(event_loop) {
            return;
        }
        self.app.about_to_wait(event_loop);
    }

    fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.maybe_exit(event_loop) {
            return;
        }
        self.app.suspended(event_loop);
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.maybe_exit(event_loop) {
            return;
        }
        self.app.exiting(event_loop);
    }

    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.maybe_exit(event_loop) {
            return;
        }
        self.app.memory_warning(event_loop);
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.maybe_exit(event_loop) {
            return;
        }
        self.app.resumed(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if self.maybe_exit(event_loop) {
            return;
        }
        self.app.window_event(event_loop, window_id, event);
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
        let thread_terminator = std::sync::Arc::new(std::sync::OnceLock::new());
        let thread = std::thread::spawn({
            let thread_terminator = thread_terminator.clone();
            move || {
                let native_options = eframe::NativeOptions {
                    viewport: egui::ViewportBuilder::default()
                        .with_visible(false)
                        .with_decorations(false)
                        .with_icon(egui::IconData::default()),
                    window_builder: Some(Box::new(|wb| {
                        wb.with_visible(false).with_decorations(false)
                    })),
                    ..Default::default()
                };

                let event_loop =
                    winit::event_loop::EventLoop::<eframe::UserEvent>::with_user_event()
                        .with_any_thread(true)
                        .build()
                        .unwrap();
                event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
                let app = eframe::create_native(
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
                            let _ = ShowWindow(
                                HWND(hwnd),
                                windows::Win32::UI::WindowsAndMessaging::SW_HIDE,
                            );
                            if windows::Win32::Foundation::GetLastError().0 != 0 {
                                let err = windows::core::Error::from_win32();
                                return Err(anyhow::anyhow!("Failed to hide window: {}", err)
                                    .into_boxed_dyn_error());
                            }

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
                    &event_loop,
                );

                let mut egui_app = WinitEventLoopApp {
                    app,
                    thread_terminator,
                };
                let res = event_loop.run_app(&mut egui_app);
                log::debug!("Egui event loop exited: {:?}", res);
            }
        });
        let (hwnd, egui_ctx) = match rx.recv() {
            Ok(Ok((hwnd, egui_ctx))) => (hwnd, egui_ctx),
            Ok(Err(e)) => {
                return Err(anyhow::anyhow!("Failed to create Egui app: {}", e));
            }
            Err(e) => {
                if thread.is_finished() {
                    let thread_err = thread.join();
                    if let Err(join_err) = thread_err {
                        if let Some(panic_info) = join_err.downcast_ref::<String>() {
                            return Err(anyhow::anyhow!("Egui thread panicked: {}", panic_info));
                        } else if let Some(panic_info) = join_err.downcast_ref::<&str>() {
                            return Err(anyhow::anyhow!("Egui thread panicked: {}", panic_info));
                        } else {
                            return Err(anyhow::anyhow!("Egui thread panicked with unknown type"));
                        }
                    }
                }
                return Err(anyhow::anyhow!("Failed to receive HWND: {}", e));
            }
        };

        let hwnd = NonZeroIsize::new(hwnd).context("Received null HWND from Egui thread")?;

        Ok(Self {
            hwnd,
            thread: Some(thread),
            egui_ctx,
            thread_terminator,
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
            log::debug!("Terminating Egui window thread...");
            self.thread_terminator.set(()).ok();
            unsafe {
                windows::Win32::UI::WindowsAndMessaging::PostMessageW(
                    Some(HWND(self.hwnd.get() as *mut std::ffi::c_void)),
                    windows::Win32::UI::WindowsAndMessaging::WM_NULL,
                    windows::Win32::Foundation::WPARAM(0),
                    windows::Win32::Foundation::LPARAM(0),
                )
                .ok();
            }
            log::debug!("Waiting for Egui window thread to exit...");
            let _ = thread.join();
            log::debug!("Egui window thread exited.");
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

/// AviUtl2のテーマに基づいたeguiのVisualsを取得する。
///
/// # Example
///
/// ```rust
/// # use aviutl2_eframe::{self, eframe};
/// # fn test(cc: &eframe::CreationContext<'_>) {
/// cc.egui_ctx.all_styles_mut(|style| {
///     style.visuals = aviutl2_eframe::aviutl2_visuals();
/// });
/// # }
/// ```
pub fn aviutl2_visuals() -> eframe::egui::Visuals {
    let mut visuals = eframe::egui::Visuals::dark();

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

    if let Some(text) = text {
        visuals.override_text_color = Some(text);
        visuals.widgets.noninteractive.fg_stroke.color = text;
        visuals.widgets.inactive.fg_stroke.color = text;
        visuals.widgets.hovered.fg_stroke.color = text;
        visuals.widgets.active.fg_stroke.color = text;
        visuals.widgets.open.fg_stroke.color = text;
    }
    if let Some(text_disable) = text_disable {
        visuals.weak_text_color = Some(text_disable);
    }
    if let (Some(text_select), Some(border_select)) = (text_select, border_select) {
        visuals.selection = egui::style::Selection {
            bg_fill: text_select,
            stroke: egui::Stroke::new(1.0, border_select),
        };
    }
    if let Some(background) = background {
        visuals.window_fill = background;
        visuals.panel_fill = background;
        visuals.widgets.noninteractive.bg_fill = background;
        visuals.widgets.noninteractive.weak_bg_fill = background;
    }
    if let Some(window_border) = window_border {
        visuals.window_stroke = egui::Stroke::new(1.0, window_border);
    }
    if let Some(grouping) = grouping {
        visuals.faint_bg_color = grouping;
    }
    if let Some(window_separator) = window_separator {
        visuals.extreme_bg_color = window_separator;
    }
    if let Some(grouping_hover) = grouping_hover {
        visuals.text_edit_bg_color = Some(grouping_hover);
    }
    if let Some(grouping_select) = grouping_select {
        visuals.code_bg_color = grouping_select;
    }

    if let Some(border) = border {
        visuals.widgets.noninteractive.bg_stroke.color = border;
        visuals.widgets.inactive.bg_stroke.color = border;
    }
    if let Some(border_focus) = border_focus {
        visuals.widgets.hovered.bg_stroke.color = border_focus;
    }
    if let Some(border_select) = border_select {
        visuals.widgets.active.bg_stroke.color = border_select;
        visuals.widgets.open.bg_stroke.color = border_select;
    }

    if let Some(button_body) = button_body {
        visuals.widgets.inactive.bg_fill = button_body;
        visuals.widgets.inactive.weak_bg_fill = button_body;
    }
    if let Some(button_body_hover) = button_body_hover {
        visuals.widgets.hovered.bg_fill = button_body_hover;
        visuals.widgets.hovered.weak_bg_fill = button_body_hover;
    }
    if let Some(button_body_press) = button_body_press {
        visuals.widgets.active.bg_fill = button_body_press;
        visuals.widgets.active.weak_bg_fill = button_body_press;
    }
    if let Some(button_body_select) = button_body_select {
        visuals.widgets.open.bg_fill = button_body_select;
        visuals.widgets.open.weak_bg_fill = button_body_select;
    }

    visuals
}

fn load_color(key: &str) -> Option<egui::Color32> {
    let (r, g, b) = aviutl2::config::get_color_code(key).expect("Key contains null byte")?;
    Some(egui::Color32::from_rgb(r, g, b))
}
#[inline]
fn makelparam(low: i32, high: i32) -> isize {
    ((high as isize) << 16) | ((low as isize) & 0xFFFF)
}
