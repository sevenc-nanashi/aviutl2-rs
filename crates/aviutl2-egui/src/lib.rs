use anyhow::Context;
use aviutl2::{AnyResult, raw_window_handle};
use eframe::egui;
use std::{num::NonZeroIsize, sync::mpsc};
use windows::Win32::UI::WindowsAndMessaging::{WS_CLIPSIBLINGS, WS_TABSTOP};
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{GWL_EXSTYLE, GWL_STYLE, SetWindowLongPtrW, WS_POPUP},
};
use winit::platform::windows::EventLoopBuilderExtWindows;
use winit::raw_window_handle::HasWindowHandle;

pub struct EguiWindow {
    hwnd: NonZeroIsize,
    _thread: std::thread::JoinHandle<AnyResult<()>>,
    egui_ctx: egui::Context,
}

impl EguiWindow {
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
        let thread = std::thread::spawn(move || {
            let native_options = eframe::NativeOptions {
                event_loop_builder: Some(Box::new(|builder| {
                    builder.with_any_thread(true).with_msg_hook(|msg| {
                        let msg = msg as *const windows::Win32::UI::WindowsAndMessaging::MSG;
                        let translated = unsafe {
                            windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&*msg).0 != 0
                        };
                        aviutl2::lprintln!(
                            verbose,
                            "with_msg_hook: msg={:?}, translated={}",
                            msg,
                            translated
                        );
                        if !msg.is_null() {
                            aviutl2::lprintln!(verbose, "with_msg_hook: msg={:?}", unsafe {
                                *msg
                            },);
                        }
                        false
                    });
                })),
                window_builder: Some(Box::new(|wb| wb.with_visible(false))),
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
                    aviutl2::lprintln!(
                        verbose,
                        "Egui window created with HWND: 0x{:016X}",
                        hwnd.hwnd,
                    );
                    tx.send((hwnd.hwnd.get(), cc.egui_ctx.clone()))
                        .context("Failed to send HWND")?;
                    app_creator(cc)
                }),
            )
            .map_err(|e| anyhow::anyhow!("Eframe error: {}", e))
        });
        let (hwnd, egui_ctx) = rx
            .recv()
            .context("Failed to receive HWND from Egui thread")?;

        let hwnd = NonZeroIsize::new(hwnd).context("Received null HWND from Egui thread")?;
        // Set window styles
        unsafe {
            let hwnd = hwnd.get() as *mut std::ffi::c_void;
            SetWindowLongPtrW(
                HWND(hwnd),
                GWL_STYLE,
                (WS_CLIPSIBLINGS.0 | WS_POPUP.0 | WS_TABSTOP.0) as isize,
            );
            SetWindowLongPtrW(HWND(hwnd), GWL_EXSTYLE, 0);
        }

        Ok(Self {
            hwnd,
            _thread: thread,
            egui_ctx,
        })
    }

    pub fn egui_ctx(&self) -> &egui::Context {
        &self.egui_ctx
    }
}

impl raw_window_handle::HasWindowHandle for EguiWindow {
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
