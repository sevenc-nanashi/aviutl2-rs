use aviutl2::AnyResult;
use std::{ffi::OsStr, iter, os::windows::ffi::OsStrExt, sync::Once};
use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow,
            RegisterClassW, WNDCLASSW, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_EX_TOOLWINDOW,
            WS_POPUP,
        },
    },
    core::PCWSTR,
};

const CLASS_NAME: &str = "WsPopupWindow";
static REGISTER_ONCE: Once = Once::new();

pub struct WsPopup {
    hwnd: HWND,
}

impl WsPopup {
    pub fn new(title: &str, size: (i32, i32)) -> AnyResult<Self> {
        register_class();

        let hmodule = unsafe { GetModuleHandleW(None).map_err(|e| anyhow::anyhow!("{e}"))? };
        let hinstance = HINSTANCE(hmodule.0);
        let title_w = to_wide(title);
        let class_w = to_wide(CLASS_NAME);

        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW,
                PCWSTR(class_w.as_ptr()),
                PCWSTR(title_w.as_ptr()),
                WS_POPUP | WS_CLIPSIBLINGS | WS_CLIPCHILDREN,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                size.0,
                size.1,
                None,
                None,
                hinstance,
                None,
            )
        }?;

        Ok(Self { hwnd })
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }
}

impl Drop for WsPopup {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

impl raw_window_handle::HasWindowHandle for WsPopup {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        let raw =
            raw_window_handle::RawWindowHandle::Win32(raw_window_handle::Win32WindowHandle::new(
                std::num::NonZero::<isize>::new(self.hwnd.0 as isize)
                    .ok_or(raw_window_handle::HandleError::Unavailable)?,
            ));
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(raw) })
    }
}

fn register_class() {
    REGISTER_ONCE.call_once(|| unsafe {
        let class_w = to_wide(CLASS_NAME);
        let hmodule = GetModuleHandleW(None).unwrap();
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            hInstance: HINSTANCE(hmodule.0),
            lpszClassName: PCWSTR(class_w.as_ptr()),
            ..Default::default()
        };
        let _ = RegisterClassW(&wc);
    });
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, w, l) }
}

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(iter::once(0)).collect()
}
