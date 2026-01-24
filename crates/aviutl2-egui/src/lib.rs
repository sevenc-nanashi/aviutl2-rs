use anyhow::Context;
use aviutl2::{AnyResult, raw_window_handle};
use aviutl2::raw_window_handle::HasWindowHandle;
use eframe::egui;
use winit::platform::windows::EventLoopBuilderExtWindows;
use windows::core::BOOL;
use windows::Win32::{
    Foundation::{HWND, LPARAM},
    System::Threading::GetCurrentProcessId,
    UI::WindowsAndMessaging::{
        EnumWindows, FindWindowW, GetParent, GetWindowLongPtrW, GetWindowTextLengthW, GetWindowTextW,
        GetWindowThreadProcessId, IsWindowVisible, SetParent, SetWindowLongPtrW, SetWindowPos,
        GWL_EXSTYLE, GWL_STYLE, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, WS_CHILD,
        WS_EX_APPWINDOW, WS_EX_TOOLWINDOW, WS_POPUP,
    },
};
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::core::PCWSTR;
use std::{
    any::Any,
    num::NonZeroIsize,
    panic::AssertUnwindSafe,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
        Arc,
    },
};

pub struct EguiWindow {
    hwnd: NonZeroIsize,
    title: String,
    egui_ctx: egui::Context,
}

impl EguiWindow {
    pub fn egui_ctx(&self) -> &egui::Context {
        &self.egui_ctx
    }

    pub fn ensure_embedded(&self) {
        ensure_embedded(self.hwnd, &self.title, None);
    }

    pub fn ensure_embedded_with_parent_title(&self, parent_title: &str) {
        ensure_embedded(self.hwnd, &self.title, Some(parent_title));
    }

    pub fn focus(&self) {
        focus_hwnd(self.hwnd);
    }
}

impl raw_window_handle::HasWindowHandle for EguiWindow {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        let raw = raw_window_handle::RawWindowHandle::Win32(
            raw_window_handle::Win32WindowHandle::new(self.hwnd),
        );
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(raw) })
    }
}

pub fn spawn_eframe_window<F>(
    title: impl Into<String>,
    size: (f32, f32),
    app_creator: F,
) -> AnyResult<EguiWindow>
where
    F: FnOnce(
            &eframe::CreationContext<'_>,
        )
            -> Result<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>>
        + Send
        + 'static,
{
    let title = title.into();
    let window_title = title.clone();
    let size = egui::vec2(size.0, size.1);
    let (handle_tx, handle_rx) = mpsc::channel::<AnyResult<NonZeroIsize>>();
    let (ctx_tx, ctx_rx) = mpsc::channel::<egui::Context>();
    let handle_sent = Arc::new(AtomicBool::new(false));

    std::thread::Builder::new()
        .name(format!("aviutl2_egui_{window_title}"))
        .spawn(move || {
            let options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_title(window_title.clone())
                    .with_inner_size(size)
                    .with_decorations(false)
                    .with_taskbar(false),
                event_loop_builder: Some(Box::new(|builder| {
                    builder.with_any_thread(true);
                })),
                ..Default::default()
            };
            let mut app_creator = Some(app_creator);
            let handle_tx_for_run = handle_tx.clone();
            let ctx_tx_for_run = ctx_tx.clone();
            let handle_sent_in_cc = Arc::clone(&handle_sent);
            let run_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                eframe::run_native(
                    &window_title,
                    options,
                    Box::new(move |cc| {
                        match extract_hwnd(cc) {
                        Ok(hwnd) => {
                            let _ = handle_tx_for_run.send(Ok(hwnd));
                            handle_sent_in_cc.store(true, Ordering::SeqCst);
                        }
                            Err(err) => {
                                let message = err.to_string();
                                let _ =
                                    handle_tx_for_run.send(Err(anyhow::anyhow!(message.clone())));
                                handle_sent_in_cc.store(true, Ordering::SeqCst);
                                return Err(anyhow::anyhow!(message).into());
                            }
                        }
                        let _ = ctx_tx_for_run.send(cc.egui_ctx.clone());
                        let creator = app_creator
                            .take()
                            .expect("app creator must be called only once");
                        creator(cc)
                    }),
                )
            }));
            match run_result {
                Ok(Ok(())) => {
                    if !handle_sent.load(Ordering::SeqCst) {
                        let _ = handle_tx.send(Err(anyhow::anyhow!(
                            "eframe window started without exposing a handle"
                        )));
                    }
                }
                Ok(Err(err)) => {
                    if !handle_sent.load(Ordering::SeqCst) {
                        let _ = handle_tx.send(Err(anyhow::anyhow!(
                            "failed to start eframe window: {err}"
                        )));
                    }
                    eprintln!("Failed to run eframe window: {err}");
                }
                Err(payload) => {
                    let message = panic_message(payload.as_ref());
                    if !handle_sent.load(Ordering::SeqCst) {
                        let _ = handle_tx.send(Err(anyhow::anyhow!(
                            "eframe window panicked during startup: {message}"
                        )));
                    }
                    eprintln!("eframe window panicked during startup: {message}");
                }
            }
        })
        .context("failed to spawn egui window thread")?;

    let hwnd = handle_rx
        .recv()
        .context("failed to receive egui window handle")??;
    let egui_ctx = ctx_rx
        .recv()
        .context("failed to receive egui context")?;
    Ok(EguiWindow {
        hwnd,
        title,
        egui_ctx,
    })
}

fn extract_hwnd(cc: &eframe::CreationContext<'_>) -> AnyResult<NonZeroIsize> {
    let raw = cc
        .window_handle()
        .context("failed to get eframe window handle")?
        .as_raw();
    match raw {
        raw_window_handle::RawWindowHandle::Win32(handle) => Ok(handle.hwnd),
        _ => Err(anyhow::anyhow!("only Win32WindowHandle is supported")),
    }
}

fn ensure_embedded(hwnd: NonZeroIsize, exclude_title: &str, parent_title: Option<&str>) {
    unsafe {
        let hwnd = HWND(hwnd.get() as *mut core::ffi::c_void);
        let parent = GetParent(hwnd).unwrap_or(HWND(std::ptr::null_mut()));
        if parent.0.is_null() {
            if let Some(main_hwnd) = find_main_window(hwnd, exclude_title, parent_title) {
                let _ = SetParent(hwnd, Some(main_hwnd));
            }
        }

        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        let new_style = (style | WS_CHILD.0 | windows::Win32::UI::WindowsAndMessaging::WS_TABSTOP.0)
            & !WS_POPUP.0;
        if new_style != style {
            let _ = SetWindowLongPtrW(hwnd, GWL_STYLE, new_style as isize);
        }

        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let new_style = (ex_style & !WS_EX_APPWINDOW.0) | WS_EX_TOOLWINDOW.0;
        if new_style != ex_style {
            let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style as isize);
        }
        let _ = SetWindowPos(hwnd, None, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED);
        let _ = SetFocus(Some(hwnd));
    }
}

fn find_main_window(
    exclude_hwnd: HWND,
    exclude_title: &str,
    parent_title: Option<&str>,
) -> Option<HWND> {
    if let Some(parent_title) = parent_title {
        if let Some(hwnd) = find_window_by_title(parent_title) {
            let mut pid = 0;
            unsafe {
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
            }
            if pid == unsafe { GetCurrentProcessId() } {
                return Some(hwnd);
            }
        }
    }

    #[repr(C)]
    struct EnumData {
        pid: u32,
        exclude_hwnd: HWND,
        exclude_title: Vec<u16>,
        hwnd: Option<HWND>,
    }

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        unsafe {
            let data = &mut *(lparam.0 as *mut EnumData);
            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid != data.pid {
                return BOOL(1);
            }
            if hwnd == data.exclude_hwnd {
                return BOOL(1);
            }
            if !IsWindowVisible(hwnd).as_bool() {
                return BOOL(1);
            }
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
            if style & WS_CHILD.0 != 0 {
                return BOOL(1);
            }
            let title_len = GetWindowTextLengthW(hwnd);
            if title_len == 0 {
                return BOOL(1);
            }
            let mut buf = vec![0u16; (title_len + 1) as usize];
            if GetWindowTextW(hwnd, &mut buf) == 0 {
                return BOOL(1);
            }
            if buf == data.exclude_title {
                return BOOL(1);
            }
            data.hwnd = Some(hwnd);
            BOOL(0)
        }
    }

    let mut exclude_title_w: Vec<u16> = exclude_title.encode_utf16().collect();
    exclude_title_w.push(0);
    let mut data = EnumData {
        pid: unsafe { GetCurrentProcessId() },
        exclude_hwnd,
        exclude_title: exclude_title_w,
        hwnd: None,
    };
    unsafe {
        let _ = EnumWindows(
            Some(enum_proc),
            LPARAM(&mut data as *mut EnumData as isize),
        );
    }
    data.hwnd
}

fn find_window_by_title(title: &str) -> Option<HWND> {
    let mut title_w: Vec<u16> = title.encode_utf16().collect();
    title_w.push(0);
    unsafe { FindWindowW(None, PCWSTR(title_w.as_ptr())).ok() }
}

pub fn focus_hwnd(hwnd: NonZeroIsize) {
    unsafe {
        let hwnd = HWND(hwnd.get() as *mut core::ffi::c_void);
        let _ = SetFocus(Some(hwnd));
    }
}

fn panic_message(payload: &dyn Any) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".to_string()
    }
}
