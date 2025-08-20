//! # aviutl2-rs / input
//!
//! AviUtl2の入力プラグインを実装するためのモジュール。
//! 大まかな流れ：
//! 1. [`InputPlugin`]トレイトを実装した構造体を定義する
//! 2. [`register_input_plugin!`]マクロを使用してプラグインを登録する
//!
//! ## サンプル
//!
//! TODO

mod binding;

pub use super::common::*;
pub use binding::*;

#[doc(hidden)]
#[path = "bridge.rs"]
pub mod __bridge;
