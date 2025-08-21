//! # aviutl2-rs / output
//!
//! AviUtl2の出力プラグインを実装するためのモジュール。
//! 大まかな流れ：
//! 1. [`OutputPlugin`]トレイトを実装した構造体を定義する
//! 2. [`register_output_plugin!`]マクロを使用してプラグインを登録する
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
