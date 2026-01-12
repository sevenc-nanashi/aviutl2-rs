pub use eframe;

use eframe::{Storage, egui, glow};
use windows::Win32::{
    Foundation::HWND,
    UI::Input::KeyboardAndMouse::SetFocus,
    UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GWL_STYLE, GetWindowLongPtrW, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE,
        SWP_NOZORDER, SetWindowLongPtrW, SetWindowPos, WS_BORDER, WS_CAPTION, WS_CHILD,
        WS_DLGFRAME, WS_EX_CLIENTEDGE, WS_EX_DLGMODALFRAME, WS_EX_STATICEDGE, WS_EX_WINDOWEDGE,
        WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
    },
};
use winit::raw_window_handle::{self, HasWindowHandle};

struct WrappedApp {
    hwnd: HWND,
    inner: Box<dyn eframe::App>,
}
impl eframe::App for WrappedApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        self.ensure_child_window_style();
        self.ensure_window_focus(ctx);
        self.inner.update(ctx, frame);
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        self.inner.save(storage);
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        self.inner.on_exit(gl);
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        self.inner.clear_color(visuals)
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        self.inner.raw_input_hook(ctx, raw_input);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        self.inner.auto_save_interval()
    }

    fn persist_egui_memory(&self) -> bool {
        self.inner.persist_egui_memory()
    }
}

pub fn run_native(
    app_name: &str,
    mut options: eframe::NativeOptions,
    app_creator: eframe::AppCreator<'static>,
) -> eframe::Result<()> {
    options.event_loop_builder = Some(any_thread_event_loop_builder());
    let window_builder = options.window_builder.take();
    options.window_builder = Some(Box::new(|wb| {
        let wb = if let Some(original_wb) = window_builder {
            original_wb(wb)
        } else {
            wb
        };
        // defaultにしてaviutl2のアイコンを侵食しないようにする
        wb.with_icon(egui::IconData::default())
    }));
    eframe::run_native(
        app_name,
        options,
        Box::new(|cc| {
            let app = app_creator(cc)?;
            let window_handle = cc.window_handle()?;
            let hwnd = match window_handle.as_raw() {
                raw_window_handle::RawWindowHandle::Win32(handle) => HWND(handle.hwnd.get() as _),
                _ => {
                    unreachable!("Expected Windows window handle");
                }
            };
            Ok(Box::new(WrappedApp { hwnd, inner: app }))
        }),
    )
}

type ThreadEventLoopBuilder<T> = Box<dyn FnOnce(&mut winit::event_loop::EventLoopBuilder<T>)>;
fn any_thread_event_loop_builder<T: 'static>() -> ThreadEventLoopBuilder<T> {
    Box::new(|builder| {
        use winit::platform::windows::EventLoopBuilderExtWindows;
        builder.with_any_thread(true);
    })
}

impl WrappedApp {
    fn ensure_child_window_style(&self) {
        unsafe {
            let style = GetWindowLongPtrW(self.hwnd, GWL_STYLE) as u32;
            let new_style = (style
                & !WS_POPUP.0
                & !WS_BORDER.0
                & !WS_DLGFRAME.0
                & !WS_CAPTION.0
                & !WS_THICKFRAME.0
                & !WS_MINIMIZEBOX.0
                & !WS_MAXIMIZEBOX.0
                & !WS_SYSMENU.0)
                | WS_CHILD.0;
            if style != new_style {
                SetWindowLongPtrW(self.hwnd, GWL_STYLE, new_style as isize);
            }

            let ex_style = GetWindowLongPtrW(self.hwnd, GWL_EXSTYLE) as u32;
            let new_ex_style = ex_style
                & !WS_EX_CLIENTEDGE.0
                & !WS_EX_WINDOWEDGE.0
                & !WS_EX_DLGMODALFRAME.0
                & !WS_EX_STATICEDGE.0;
            if ex_style != new_ex_style {
                SetWindowLongPtrW(self.hwnd, GWL_EXSTYLE, new_ex_style as isize);
            }

            if style != new_style || ex_style != new_ex_style {
                let _ = SetWindowPos(
                    self.hwnd,
                    None,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
                );
            }
        }
    }

    fn ensure_window_focus(&self, ctx: &eframe::egui::Context) {
        let pressed = ctx.input(|input| input.pointer.any_pressed());
        if pressed {
            unsafe {
                let _ = SetFocus(self.hwnd);
            }
        }
    }
}
