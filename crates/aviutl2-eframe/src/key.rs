pub struct WindowsKeyMessage {
    pub wparam: usize,
    pub lparam: isize,
}

pub fn egui_key_to_windows_key_message(
    logical_key: eframe::egui::Key,
    physical_key: Option<eframe::egui::Key>,
    pressed: bool,
    keyboard_layout: Option<windows::Win32::UI::Input::KeyboardAndMouse::HKL>,
) -> Option<WindowsKeyMessage> {
    if let Some(scancode) = physical_key.and_then(egui_physical_key_to_windows_scancode) {
        let wparam = unsafe {
            windows::Win32::UI::Input::KeyboardAndMouse::MapVirtualKeyExW(
                scancode,
                windows::Win32::UI::Input::KeyboardAndMouse::MAPVK_VSC_TO_VK_EX,
                keyboard_layout,
            )
        };
        if wparam != 0 {
            return Some(WindowsKeyMessage {
                wparam: wparam as usize,
                lparam: key_lparam(scancode, pressed),
            });
        }
    }

    let wparam = egui_key_to_windows_key(logical_key)?;
    let scancode = unsafe {
        windows::Win32::UI::Input::KeyboardAndMouse::MapVirtualKeyExW(
            wparam,
            windows::Win32::UI::Input::KeyboardAndMouse::MAPVK_VK_TO_VSC_EX,
            keyboard_layout,
        )
    };
    Some(WindowsKeyMessage {
        wparam: wparam as usize,
        lparam: key_lparam(scancode, pressed),
    })
}

fn egui_key_to_windows_key(key: eframe::egui::Key) -> Option<u32> {
    use eframe::egui::Key::*;
    match key {
        ArrowDown => Some(0x28),
        ArrowLeft => Some(0x25),
        ArrowRight => Some(0x27),
        ArrowUp => Some(0x26),
        Escape => Some(0x1B),
        Tab => Some(0x09),
        Backspace => Some(0x08),
        Enter => Some(0x0D),
        Space => Some(0x20),
        Insert => Some(0x2D),
        Delete => Some(0x2E),
        Home => Some(0x24),
        End => Some(0x23),
        PageUp => Some(0x21),
        PageDown => Some(0x22),
        Copy | Cut | Paste => None,
        Colon | Semicolon => Some(0xBA),
        Comma => Some(0xBC),
        Backslash | Pipe => Some(0xDC),
        Slash | Questionmark => Some(0xBF),
        Exclamationmark => Some(0x31),
        OpenBracket | OpenCurlyBracket => Some(0xDB),
        CloseBracket | CloseCurlyBracket => Some(0xDD),
        Backtick => Some(0xC0),
        Minus => Some(0xBD),
        Period => Some(0xBE),
        Plus | Equals => Some(0xBB),
        Quote => Some(0xDE),
        Num0 => Some(0x30),
        Num1 => Some(0x31),
        Num2 => Some(0x32),
        Num3 => Some(0x33),
        Num4 => Some(0x34),
        Num5 => Some(0x35),
        Num6 => Some(0x36),
        Num7 => Some(0x37),
        Num8 => Some(0x38),
        Num9 => Some(0x39),
        A => Some(0x41),
        B => Some(0x42),
        C => Some(0x43),
        D => Some(0x44),
        E => Some(0x45),
        F => Some(0x46),
        G => Some(0x47),
        H => Some(0x48),
        I => Some(0x49),
        J => Some(0x4A),
        K => Some(0x4B),
        L => Some(0x4C),
        M => Some(0x4D),
        N => Some(0x4E),
        O => Some(0x4F),
        P => Some(0x50),
        Q => Some(0x51),
        R => Some(0x52),
        S => Some(0x53),
        T => Some(0x54),
        U => Some(0x55),
        V => Some(0x56),
        W => Some(0x57),
        X => Some(0x58),
        Y => Some(0x59),
        Z => Some(0x5A),
        F1 => Some(0x70),
        F2 => Some(0x71),
        F3 => Some(0x72),
        F4 => Some(0x73),
        F5 => Some(0x74),
        F6 => Some(0x75),
        F7 => Some(0x76),
        F8 => Some(0x77),
        F9 => Some(0x78),
        F10 => Some(0x79),
        F11 => Some(0x7A),
        F12 => Some(0x7B),
        F13 => Some(0x7C),
        F14 => Some(0x7D),
        F15 => Some(0x7E),
        F16 => Some(0x7F),
        F17 => Some(0x80),
        F18 => Some(0x81),
        F19 => Some(0x82),
        F20 => Some(0x83),
        F21 => Some(0x84),
        F22 => Some(0x85),
        F23 => Some(0x86),
        F24 => Some(0x87),
        F25 | F26 | F27 | F28 | F29 | F30 | F31 | F32 | F33 | F34 | F35 => None,
        BrowserBack => Some(0xA6),
    }
}

fn egui_physical_key_to_windows_scancode(key: eframe::egui::Key) -> Option<u32> {
    use eframe::egui::Key::*;
    match key {
        ArrowDown => Some(0xe050),
        ArrowLeft => Some(0xe04b),
        ArrowRight => Some(0xe04d),
        ArrowUp => Some(0xe048),
        Escape => Some(0x0001),
        Tab => Some(0x000f),
        Backspace => Some(0x000e),
        Enter => Some(0x001c),
        Space => Some(0x0039),
        Insert => Some(0xe052),
        Delete => Some(0xe053),
        Home => Some(0xe047),
        End => Some(0xe04f),
        PageUp => Some(0xe049),
        PageDown => Some(0xe051),
        Copy | Cut | Paste => None,
        Colon | Semicolon => Some(0x0027),
        Comma => Some(0x0033),
        Backslash | Pipe => Some(0x002b),
        Slash | Questionmark => Some(0x0035),
        Exclamationmark => Some(0x0002),
        OpenBracket | OpenCurlyBracket => Some(0x001a),
        CloseBracket | CloseCurlyBracket => Some(0x001b),
        Backtick => Some(0x0029),
        Minus => Some(0x000c),
        Period => Some(0x0034),
        Plus | Equals => Some(0x000d),
        Quote => Some(0x0028),
        Num0 => Some(0x000b),
        Num1 => Some(0x0002),
        Num2 => Some(0x0003),
        Num3 => Some(0x0004),
        Num4 => Some(0x0005),
        Num5 => Some(0x0006),
        Num6 => Some(0x0007),
        Num7 => Some(0x0008),
        Num8 => Some(0x0009),
        Num9 => Some(0x000a),
        A => Some(0x001e),
        B => Some(0x0030),
        C => Some(0x002e),
        D => Some(0x0020),
        E => Some(0x0012),
        F => Some(0x0021),
        G => Some(0x0022),
        H => Some(0x0023),
        I => Some(0x0017),
        J => Some(0x0024),
        K => Some(0x0025),
        L => Some(0x0026),
        M => Some(0x0032),
        N => Some(0x0031),
        O => Some(0x0018),
        P => Some(0x0019),
        Q => Some(0x0010),
        R => Some(0x0013),
        S => Some(0x001f),
        T => Some(0x0014),
        U => Some(0x0016),
        V => Some(0x002f),
        W => Some(0x0011),
        X => Some(0x002d),
        Y => Some(0x0015),
        Z => Some(0x002c),
        F1 => Some(0x003b),
        F2 => Some(0x003c),
        F3 => Some(0x003d),
        F4 => Some(0x003e),
        F5 => Some(0x003f),
        F6 => Some(0x0040),
        F7 => Some(0x0041),
        F8 => Some(0x0042),
        F9 => Some(0x0043),
        F10 => Some(0x0044),
        F11 => Some(0x0057),
        F12 => Some(0x0058),
        F13 => Some(0x0064),
        F14 => Some(0x0065),
        F15 => Some(0x0066),
        F16 => Some(0x0067),
        F17 => Some(0x0068),
        F18 => Some(0x0069),
        F19 => Some(0x006a),
        F20 => Some(0x006b),
        F21 => Some(0x006c),
        F22 => Some(0x006d),
        F23 => Some(0x006e),
        F24 => Some(0x0076),
        F25 | F26 | F27 | F28 | F29 | F30 | F31 | F32 | F33 | F34 | F35 => None,
        BrowserBack => Some(0xe06a),
    }
}

fn key_lparam(scancode: u32, pressed: bool) -> isize {
    let mut lparam = 1 | (((scancode & 0xff) as isize) << 16);
    if scancode & 0xff00 == 0xe000 {
        lparam |= 1 << 24;
    }
    if !pressed {
        lparam |= 1 << 30;
        lparam |= 1 << 31;
    }
    lparam
}
