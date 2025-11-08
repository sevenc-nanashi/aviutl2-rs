//! [`log`](https://crates.io/crates/log) クレートを使用したロギング機能を提供します。

use std::sync::{OnceLock, mpsc};
use std::thread;

/// フォーマッター。
pub type Formatter = dyn Fn(&log::Record) -> String + Send + Sync + 'static;

/// ログのBuilder。
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
    /// [`try_init`]と異なり、エラーが発生した場合にパニックします。
    pub fn init(self) {
        self.try_init().expect("Failed to initialize logger")
    }

    /// ロガーを初期化します。
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

/// 各クレートのログ設定を表す構造体。
pub struct CrateLogConfig {
    /// クレート名。
    ///
    /// env_loggerと同じような仕様で判定されます。
    pub crate_name: String,
    /// ログレベル。
    pub level: log::LevelFilter,
}

pub use log::LevelFilter;

enum LogCommand {
    Record {
        level: log::Level,
        message: Vec<u16>,
    },
    Plugin {
        message: Vec<u16>,
    },
}

struct InternalLogger {
    formatter: Box<Formatter>,
}

impl InternalLogger {
    fn new(formatter: Box<Formatter>) -> Self {
        ensure_logger_dispatcher();
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
        let leak_manager = message
            .encode_utf16()
            .chain(std::iter::once(0)) // null terminator
            .collect::<Vec<u16>>();
        let _ = logger_sender().send(LogCommand::Record {
            level: record.level(),
            message: leak_manager,
        });
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
        $crate::oprintln!("[{}:{}:{}]", ::std::file!(), ::std::line!(), ::std::column!());
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::lprintln!("[{}:{}:{}] {} = {:#?}",
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
        ($($crate::odbg!($val)),+,)
    };
}

/// プラグイン用ログに出力する[`println!`]マクロ。
#[macro_export]
macro_rules! lprintln {
    ($($arg:tt)*) => {
        let message = format!($($arg)*);
        $crate::log::write_plugin_log(&message);
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
pub fn write_plugin_log(message: &str) {
    let mut leak_manager = message.encode_utf16().collect::<Vec<u16>>();
    leak_manager.push(0); // null terminator
    let _ = logger_sender().send(LogCommand::Plugin {
        message: leak_manager,
    });
}

struct InternalLoggerHandle(*mut aviutl2_sys::logger2::LOG_HANDLE);
unsafe impl Send for InternalLoggerHandle {}

static LOGGER_HANDLE: std::sync::OnceLock<std::sync::Mutex<InternalLoggerHandle>> =
    std::sync::OnceLock::new();
static LOGGER_DISPATCHER: OnceLock<mpsc::Sender<LogCommand>> = OnceLock::new();

#[doc(hidden)]
pub fn __initialize_logger(handle: *mut aviutl2_sys::logger2::LOG_HANDLE) {
    let internal_handle = InternalLoggerHandle(handle);
    LOGGER_HANDLE
        .set(std::sync::Mutex::new(internal_handle))
        .unwrap_or_else(|_| {
            panic!("Logger has already been initialized");
        });
}

fn ensure_logger_dispatcher() {
    LOGGER_DISPATCHER.get_or_init(spawn_logger_dispatcher);
}

fn logger_sender() -> &'static mpsc::Sender<LogCommand> {
    ensure_logger_dispatcher();
    LOGGER_DISPATCHER
        .get()
        .expect("logger dispatcher should be initialized")
}

fn spawn_logger_dispatcher() -> mpsc::Sender<LogCommand> {
    let (tx, rx) = mpsc::channel();
    thread::Builder::new()
        .name("aviutl2-logger".into())
        .spawn(move || {
            let internal_handle = LOGGER_HANDLE
                .get()
                .expect("logger handle should be initialized")
                .lock()
                .unwrap();
            while let Ok(command) = rx.recv() {
                match command {
                    LogCommand::Record { level, message } => {
                        let wide_message = message.as_ptr();
                        unsafe {
                            match level {
                                log::Level::Error => {
                                    ((*internal_handle.0).error)(internal_handle.0, wide_message)
                                }
                                log::Level::Warn => {
                                    ((*internal_handle.0).warn)(internal_handle.0, wide_message)
                                }
                                log::Level::Info => {
                                    ((*internal_handle.0).info)(internal_handle.0, wide_message)
                                }
                                log::Level::Debug | log::Level::Trace => {
                                    ((*internal_handle.0).verbose)(internal_handle.0, wide_message)
                                }
                            }
                        }
                    }
                    LogCommand::Plugin { message } => {
                        let wide_message = message.as_ptr();
                        unsafe {
                            ((*internal_handle.0).log)(internal_handle.0, wide_message);
                        }
                    }
                }
            }
        })
        .expect("failed to spawn aviutl2 logger dispatcher");
    tx
}
