//! AviUtl2のロガーへのインターフェースを提供します。
//!
//! # Examples
//!
//! AviUtl2のロガーに直接書き込むことができます。
//!
//! ```rust
//! aviutl2::logger::write_plugin_log("This is a plugin log message.").unwrap();
//! aviutl2::logger::write_info_log("This is an info log message.").unwrap();
//! aviutl2::logger::write_warn_log("This is a warning log message.").unwrap();
//! aviutl2::logger::write_error_log("This is an error log message.").unwrap();
//! aviutl2::logger::write_verbose_log("This is a verbose log message.").unwrap();
//!
//! aviutl2::lprintln!("This is a plugin log message.");  // デフォルトはpluginログに出力
//! aviutl2::lprintln!(plugin, "This is also a plugin log message.");
//! aviutl2::lprintln!(info, "This is an info log message.");
//! aviutl2::lprintln!(warn, "This is a warning log message.");
//! aviutl2::lprintln!(error, "This is an error log message.");
//! aviutl2::lprintln!(verbose, "This is a verbose log message.");
//!
//! aviutl2::ldbg!(42); // dbg!マクロに相当
//! ```
//!
//! [`tracing`]クレートと組み合わせることもできます。
//!
//! ```rust
//! aviutl2::tracing_subscriber::fmt()
//!     .with_max_level(if cfg!(debug_assertions) {
//!         tracing::Level::DEBUG
//!     } else {
//!         tracing::Level::INFO
//!     })
//!     .event_format(aviutl2::logger::AviUtl2Formatter)
//!     .with_writer(aviutl2::logger::AviUtl2LogWriter)
//!     .init();
//!
//! tracing::info!("This is an info log message using tracing.");
//! ```

use crate::common::{CWString, NullByteError};
use tracing_subscriber::fmt::FormatFields;

// NOTE:
// InitializeLoggerは可能な限り早く実行されるらしいので、まぁ捨てられるログはないとしていいはず...

/// [`tracing_subscriber::fmt::FormatEvent`]を実装する構造体。
///
/// AviUtl2風のログフォーマットでイベントをフォーマットします。
#[derive(Debug, Clone, Default)]
pub struct AviUtl2Formatter;

impl<C, N> tracing_subscriber::fmt::FormatEvent<C, N> for AviUtl2Formatter
where
    C: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> tracing_subscriber::fmt::FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, C, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let target = event.metadata().target();
        write!(writer, "[{target}] ")?;
        ctx.format_fields(writer.by_ref(), event)?;
        writer.write_str("\n")?;
        Ok(())
    }
}

/// [`tracing_subscriber::fmt::MakeWriter`]を実装する構造体。
///
/// AviUtl2のログに書き込みます。
#[derive(Debug, Clone, Default)]
pub struct AviUtl2LogWriter;

impl tracing_subscriber::fmt::MakeWriter<'_> for AviUtl2LogWriter {
    type Writer = LockedInternalWriter;

    fn make_writer(&self) -> Self::Writer {
        LockedInternalWriter::plugin()
    }

    fn make_writer_for(&'_ self, meta: &tracing::Metadata<'_>) -> Self::Writer {
        match *meta.level() {
            tracing::Level::ERROR => LockedInternalWriter::error(),
            tracing::Level::WARN => LockedInternalWriter::warn(),
            tracing::Level::INFO => LockedInternalWriter::info(),
            tracing::Level::DEBUG | tracing::Level::TRACE => LockedInternalWriter::verbose(),
        }
    }
}

static INTERNAL_WRITER_MUTEX_PLUGIN: std::sync::LazyLock<std::sync::Mutex<InternalWriter>> =
    std::sync::LazyLock::new(|| {
        std::sync::Mutex::new(InternalWriter::new(InternalWriterLevel::Plugin))
    });
static INTERNAL_WRITER_MUTEX_INFO: std::sync::LazyLock<std::sync::Mutex<InternalWriter>> =
    std::sync::LazyLock::new(|| {
        std::sync::Mutex::new(InternalWriter::new(InternalWriterLevel::Info))
    });
static INTERNAL_WRITER_MUTEX_WARN: std::sync::LazyLock<std::sync::Mutex<InternalWriter>> =
    std::sync::LazyLock::new(|| {
        std::sync::Mutex::new(InternalWriter::new(InternalWriterLevel::Warn))
    });
static INTERNAL_WRITER_MUTEX_ERROR: std::sync::LazyLock<std::sync::Mutex<InternalWriter>> =
    std::sync::LazyLock::new(|| {
        std::sync::Mutex::new(InternalWriter::new(InternalWriterLevel::Error))
    });
static INTERNAL_WRITER_MUTEX_VERBOSE: std::sync::LazyLock<std::sync::Mutex<InternalWriter>> =
    std::sync::LazyLock::new(|| {
        std::sync::Mutex::new(InternalWriter::new(InternalWriterLevel::Verbose))
    });

pub struct LockedInternalWriter {
    mutex: &'static std::sync::Mutex<InternalWriter>,
}

impl LockedInternalWriter {
    pub fn plugin() -> Self {
        Self {
            mutex: &INTERNAL_WRITER_MUTEX_PLUGIN,
        }
    }

    pub fn info() -> Self {
        Self {
            mutex: &INTERNAL_WRITER_MUTEX_INFO,
        }
    }

    pub fn warn() -> Self {
        Self {
            mutex: &INTERNAL_WRITER_MUTEX_WARN,
        }
    }

    pub fn error() -> Self {
        Self {
            mutex: &INTERNAL_WRITER_MUTEX_ERROR,
        }
    }

    pub fn verbose() -> Self {
        Self {
            mutex: &INTERNAL_WRITER_MUTEX_VERBOSE,
        }
    }
}
impl std::io::Write for LockedInternalWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut writer = self.mutex.lock().unwrap();
        writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut writer = self.mutex.lock().unwrap();
        writer.flush()
    }
}

enum InternalWriterLevel {
    Plugin,
    Info,
    Warn,
    Error,
    Verbose,
}

struct InternalWriter {
    level: InternalWriterLevel,
    buffer: Vec<u8>,
}
impl InternalWriter {
    fn new(level: InternalWriterLevel) -> Self {
        Self {
            level,
            buffer: Vec::new(),
        }
    }
}

impl std::io::Write for InternalWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        self.flush()?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        while let Some(pos) = self.buffer.iter().position(|&b| b == b'\n') {
            let line = self.buffer.drain(..=pos).collect::<Vec<u8>>();
            let line = String::from_utf8_lossy(&line);
            let line = line.trim_end_matches('\n');
            match self.level {
                InternalWriterLevel::Plugin => {
                    let _ = write_plugin_log(line);
                }
                InternalWriterLevel::Info => {
                    let _ = write_info_log(line);
                }
                InternalWriterLevel::Warn => {
                    let _ = write_warn_log(line);
                }
                InternalWriterLevel::Error => {
                    let _ = write_error_log(line);
                }
                InternalWriterLevel::Verbose => {
                    let _ = write_verbose_log(line);
                }
            }
        }
        Ok(())
    }
}

/// プラグイン用ログに出力する[`dbg!`]マクロ。
///
/// # See Also
/// <https://github.com/rust-lang/rust/blob/29483883eed69d5fb4db01964cdf2af4d86e9cb2/library/std/src/macros.rs#L352>
#[macro_export]
macro_rules! ldbg {
    () => {
        $crate::lprintln!(verbose, "[{}:{}:{}]", ::std::file!(), ::std::line!(), ::std::column!());
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::lprintln!(verbose, "[{}:{}:{}] {} = {:#?}",
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
/// lprintln!(verbose, "This is a verbose log message.");
/// ```
#[macro_export]
macro_rules! lprintln {
    (plugin, $($arg:tt)*) => {
        ::std::mem::drop($crate::logger::write_plugin_log(&format!($($arg)*)));
    };
    (info, $($arg:tt)*) => {
        ::std::mem::drop($crate::logger::write_info_log(&format!($($arg)*)));
    };
    (warn, $($arg:tt)*) => {
        ::std::mem::drop($crate::logger::write_warn_log(&format!($($arg)*)));
    };
    (error, $($arg:tt)*) => {
        ::std::mem::drop($crate::logger::write_error_log(&format!($($arg)*)));
    };
    (verbose, $($arg:tt)*) => {
        ::std::mem::drop($crate::logger::write_verbose_log(&format!($($arg)*)));
    };
    ($($arg:tt)*) => {
        $crate::lprintln!(plugin, $($arg)*);
    };
}

#[cfg(feature = "wrap_log")]
fn log_length_limit(kind_length: usize) -> usize {
    static DLL_LENGTH: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    let dll_length = *DLL_LENGTH.get_or_init(|| {
        process_path::get_dylib_path()
            .map_or(0, |path| path.file_name().unwrap().to_string_lossy().len())
    });
    // [01/23 08:43:47] [VERBOSE] [Plugin::vi5.aux2] ...
    1023 - 35 - dll_length - kind_length
}
#[cfg(not(feature = "wrap_log"))]
fn log_length_limit(_kind_length: usize) -> usize {
    // wrap_logが無効な場合は制限なし
    usize::MAX
}

fn split_into_chunks(message: &str, kind_length: usize) -> Vec<String> {
    // 二分探索みたいなことをすればもっと効率的にできるけど面倒なので...
    let log_length_limit = log_length_limit(kind_length);
    let mut chunks = Vec::with_capacity(message.len() / log_length_limit + 1);
    let mut current_chunk = String::new();
    for letter in message.chars() {
        let letter_len = letter.len_utf8();
        if current_chunk.len() + letter_len > log_length_limit {
            chunks.push(std::mem::take(&mut current_chunk));
        }
        current_chunk.push(letter);
    }
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }
    chunks
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
        for chunk in split_into_chunks(message, "PLUGIN".len()) {
            let wide_message = CWString::new(&chunk)?;
            ((*handle).log)(handle, wide_message.as_ptr());
        }
        Ok(())
    })
    .unwrap_or(Ok(()))
}

#[duplicate::duplicate_item(
    level       function_name       log_method;
    ["ERROR"]   [write_error_log]   [error];
    ["WARN"]    [write_warn_log]    [warn];
    ["INFO"]    [write_info_log]    [info];
    ["VERBOSE"] [write_verbose_log] [verbose];
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
        for chunk in split_into_chunks(message, level.len()) {
            let wide_message = CWString::new(&chunk)?;
            ((*handle).log_method)(handle, wide_message.as_ptr());
        }
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

#[doc(hidden)]
pub fn __initialize_logger_unwind(handle: *mut aviutl2_sys::logger2::LOG_HANDLE) {
    if let Err(panic_info) =
        crate::utils::catch_unwind_with_panic_info(|| __initialize_logger(handle))
    {
        crate::tracing::error!("Panic occurred during InitializeLogger: {}", panic_info);
        let _ = crate::logger::write_error_log(&panic_info);
    }
}

impl InternalLoggerHandle {
    fn ptr(&self) -> *mut aviutl2_sys::logger2::LOG_HANDLE {
        self.0
    }
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_can_compile_ldbg() {
        let x = 42;
        ldbg!();
        ldbg!(x);
        ldbg!(x + 1, x * 2);
    }

    #[test]
    fn test_can_compile_lprintln() {
        lprintln!("This is a test log message.");
        lprintln!(info, "This is an info log message.");
        lprintln!(warn, "This is a warning log message.");
        lprintln!(error, "This is an error log message.");
        lprintln!(verbose, "This is a verbose log message.");
    }

    #[test]
    #[cfg(feature = "wrap_log")]
    fn test_split_into_chunks() {
        let message = "a".repeat(5000);
        let chunks = super::split_into_chunks(&message, "VERBOSE".len());
        let dylib_name = process_path::get_dylib_path()
            .and_then(|path| {
                path.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            })
            .unwrap();
        for chunk in chunks {
            assert!(chunk.len() <= super::log_length_limit("VERBOSE".len()));
            assert!(
                format!(
                    "[01/23 08:43:47] [VERBOSE] [Plugin::{}] {}",
                    dylib_name, chunk
                )
                .len()
                    <= 1023
            );
        }
    }
}
