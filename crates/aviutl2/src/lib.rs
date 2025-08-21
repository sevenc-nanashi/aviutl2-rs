//! # aviutl2-rs
//!
//! AviUtl2 SDKのRustラッパー。
//! このクレートは、AviUtl2のプラグインをRustらしく開発できるようにすることを目的としています。
//!
//! ## モジュール
//!
//! - [`input`](input): AviUtl2の入力プラグインを実装するためのモジュール。
//! - [`output`](output): AviUtl2の出力プラグインを実装するためのモジュール。
//! - [`common`](common): 共通の型や関数を提供するモジュール。
//!
//! 詳細は各モジュールのドキュメントを参照してください。
pub use aviutl2_sys as sys;

#[cfg(feature = "input")]
pub mod input;

#[cfg(feature = "output")]
pub mod output;

pub mod common;
pub mod utils;
pub use anyhow;
pub use common::*;
