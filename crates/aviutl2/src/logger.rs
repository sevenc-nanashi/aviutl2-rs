//! AviUtl2のロガーへのインターフェースを提供します。

pub use log::LevelFilter;
use crate::common::{CWString, NullByteError};

// NOTE:
// InitializeLoggerは可能な限り早く実行されるらしいので、まぁ捨てられるログはないとしていいはず...

/// フォーマッター。
///
/// # See Also
///
/// [`LogBuilder::formatter`]
pub type Formatter = dyn Fn(&log::Record) -> String + Send + Sync + 'static;

/// [`log`]クレートを使用してAviUtl2のログ出力を設定するためのビルダー。
#[must_use]
pub struct LogBuilder {
    filter: env_filter::Builder,
    formatter: Option<Box<Formatter>>,
}

impl LogBuilder {
    /// 新しい`LogBuilder`を作成します。
    pub fn new() -> Self {
        LogBuilder {
            filter: env_filter::Builder::new(),
            formatter: None,
        }
    }

    /// 全てのモジュールのログレベルを設定します。
    pub fn filter_level(mut self, level: log::LevelFilter) -> Self {
        self.filter.filter_level(level);
        self
    }

    /// 指定したモジュールのログレベルを設定します。
    pub fn filter_module(mut self, module: &str, level: log::LevelFilter) -> Self {
        self.filter.filter_module(module, level);
        self
    }

    /// ログのフォーマッタを設定します。
    pub fn formatter<F>(mut self, formatter: F) -> Self
    where
        F: Fn(&log::Record) -> String + Send + Sync + 'static,
    {
        self.formatter = Some(Box::new(formatter));
        self
    }

    /// ログのフォーマッターをデフォルトのものに設定します。
    pub fn default_formatter(mut self) -> Self {
        self.formatter = None;
        self
    }

    /// ロガーを初期化します。
    ///
    /// [`LogBuilder::try_init`]と異なり、エラーが発生した場合にパニックします。
    pub fn init(self) {
        self.try_init().expect("Failed to initialize logger")
    }

    /// ロガーを初期化します。
    ///
    /// # Errors
    ///
    /// ロガーを2回以上初期化しようとした場合にエラーを返します。
    pub fn try_init(self) -> Result<(), log::SetLoggerError> {
        let LogBuilder {
            mut filter,
            formatter,
        } = self;
        let filter = filter.build();
        let logger = InternalLogger::new(formatter.unwrap_or_else(|| {
            Box::new(|record: &log::Record| format!("[{}] {}", record.target(), record.args()))
        }));
        log::set_max_level(filter.filter());
        let logger = env_filter::FilteredLog::new(logger, filter);
        log::set_boxed_logger(Box::new(logger))?;
        Ok(())
    }
}

impl Default for LogBuilder {
    fn default() -> Self {
        Self::new()
    }
}
struct InternalLogger {
    formatter: Box<Formatter>,
}

impl InternalLogger {
    fn new(formatter: Box<Formatter>) -> Self {
        InternalLogger { formatter }
    }
}

// NOTE: env_filterがいい感じにやってくれるらしい（ありがたい）
impl log::Log for InternalLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let message = (self.formatter)(record);
        send_record(record.level(), message);
    }

    fn flush(&self) {
        // No-op
    }
}

/// プラグイン用ログに出力する[`dbg!`]マクロ。
///
/// # See Also
/// <https://github.com/rust-lang/rust/blob/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/macros.rs#L352>
#[macro_export]
macro_rules! ldbg {
    () => {
        $crate::lprintln!(debug, "[{}:{}:{}]", ::std::file!(), ::std::line!(), ::std::column!());
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::lprintln!(debug, "[{}:{}:{}] {} = {:#?}",
                    ::std::file!(),
                    ::std::line!(),
                    ::std::column!(),
                    ::std::stringify!($val),
                    &&tmp as &dyn ::std::fmt::Debug,
                );
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::ldbg!($val)),+,)
    };
}

/// プラグイン用ログに出力する[`println!`]マクロ。
///
/// ```rust
/// # use aviutl2::lprintln;
/// lprintln!("This is a plugin log message.");  // デフォルトはpluginログに出力
/// lprintln!(plugin, "This is also a plugin log message.");
/// lprintln!(info, "This is an info log message.");
/// lprintln!(warn, "This is a warning log message.");
/// lprintln!(error, "This is an error log message.");
/// lprintln!(debug, "This is a debug log message.");
/// ```
#[macro_export]
macro_rules! lprintln {
    (plugin, $($arg:tt)*) => {
        let message = format!($($arg)*);
        let _ = $crate::logger::write_plugin_log(&message);
    };
    (info, $($arg:tt)*) => {
        let message = format!($($arg)*);
        let _ = $crate::logger::write_info_log(&message);
    };
    (warn, $($arg:tt)*) => {
        let message = format!($($arg)*);
        let _ = $crate::logger::write_warn_log(&message);
    };
    (error, $($arg:tt)*) => {
        let message = format!($($arg)*);
        let _ = $crate::logger::write_error_log(&message);
    };
    (debug, $($arg:tt)*) => {
        let message = format!($($arg)*);
        let _ = $crate::logger::write_debug_log(&message);
    };
    ($($arg:tt)*) => {
        $crate::lprintln!(plugin, $($arg)*);
    };
}

/// プラグイン用ログにメッセージを書き込みます。
///
/// # Note
///
/// ロガーが初期化されていない場合は何も行いません。
///
/// # See Also
///
/// - [`ldbg!`]
/// - [`lprintln!`]
pub fn write_plugin_log(message: &str) -> Result<(), NullByteError> {
    with_logger_handle(|handle| unsafe {
        let wide_message = CWString::new(message)?;
        ((*handle).log)(handle, wide_message.as_ptr());
        Ok(())
    })
    .unwrap_or(Ok(()))
}

#[duplicate::duplicate_item(
    level       function_name;
    ["ERROR"]   [write_error_log];
    ["WARN"]    [write_warn_log];
    ["INFO"]    [write_info_log];
    ["DEBUG"]   [write_debug_log];
)]
#[doc = concat!("ログに", level, "レベルのメッセージを書き込みます。")]
///
/// # Note
///
/// ロガーが初期化されていない場合は何も行いません。
///
/// # See Also
///
/// - [`ldbg!`]
/// - [`lprintln!`]
pub fn function_name(message: &str) -> Result<(), NullByteError> {
    with_logger_handle(|handle| unsafe {
        let wide_message = CWString::new(message)?;
        ((*handle).log)(handle, wide_message.as_ptr());
        Ok(())
    })
    .unwrap_or(Ok(()))
}

struct InternalLoggerHandle(*mut aviutl2_sys::logger2::LOG_HANDLE);
unsafe impl Send for InternalLoggerHandle {}

static LOGGER_HANDLE: std::sync::OnceLock<std::sync::Mutex<InternalLoggerHandle>> =
    std::sync::OnceLock::new();

#[doc(hidden)]
pub fn __initialize_logger(handle: *mut aviutl2_sys::logger2::LOG_HANDLE) {
    let internal_handle = InternalLoggerHandle(handle);
    LOGGER_HANDLE
        .set(std::sync::Mutex::new(internal_handle))
        .unwrap_or_else(|_| {
            panic!("Logger has already been initialized");
        });
}

impl InternalLoggerHandle {
    fn ptr(&self) -> *mut aviutl2_sys::logger2::LOG_HANDLE {
        self.0
    }
}

fn encode_utf16_with_nul(message: &str) -> Vec<u16> {
    message.encode_utf16().chain(std::iter::once(0)).collect()
}

fn with_logger_handle<F, T>(f: F) -> Option<T>
where
    F: FnOnce(*mut aviutl2_sys::logger2::LOG_HANDLE) -> T,
{
    let handle = LOGGER_HANDLE.get()?;
    let handle = handle.lock().unwrap();
    let handle_ptr = handle.ptr();
    Some(f(handle_ptr))
}

fn send_record(level: log::Level, message: String) {
    let wide_message = encode_utf16_with_nul(&message);
    with_logger_handle(|handle| unsafe {
        match level {
            log::Level::Error => ((*handle).error)(handle, wide_message.as_ptr()),
            log::Level::Warn => ((*handle).warn)(handle, wide_message.as_ptr()),
            log::Level::Info => ((*handle).info)(handle, wide_message.as_ptr()),
            log::Level::Debug | log::Level::Trace => {
                ((*handle).verbose)(handle, wide_message.as_ptr())
            }
        }
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_can_compile_ldbg() {
        let x = 42;
        ldbg!();
        ldbg!(x);
        ldbg!(x + 1, x * 2);
    }
}
