//! # aviutl2-rs / filter
//!
//! AviUtl2のフィルタプラグインを実装するためのモジュール。
//! 大まかな流れ：
//! 1. [`FilterPlugin`]トレイトを実装した構造体を定義する
//! 2. [`crate::register_filter_plugin!`]マクロを使用してプラグインを登録する
//!
//! サンプルは<https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples>を参照してください。

mod binding;
mod config;

pub use super::common::*;
pub use binding::*;
pub use config::*;

#[doc(hidden)]
#[path = "bridge.rs"]
pub mod __bridge;
