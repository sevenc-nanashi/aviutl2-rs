//! # aviutl2-rs
//!
//! AviUtl2 SDKのRustラッパー。
//! このクレートは、AviUtl2のプラグインをRustらしく開発できるようにすることを目的としています。
//!
//! ## モジュール
//!
//! - [`input`][]：AviUtl2の入力プラグインを実装するためのモジュール。
//! - [`output`][]：AviUtl2の出力プラグインを実装するためのモジュール。
//! - [`filter`][]：AviUtl2のフィルタプラグインを実装するためのモジュール。
//! - [`module`][]：AviUtl2のスクリプトモジュールプラグインを実装するためのモジュール。
//! - [`logger`][]：[log](https://crates.io/crates/log)クレートを使用してAviUtl2のログ出力を行うためのモジュール。
//! - [`common`][]：共通の型や関数を提供するモジュール。
//! - [`utils`][]：ユーティリティ関数を提供するモジュール。
//!
//! 詳細は各モジュールのドキュメントを参照してください。
//!
//! ## Feature Flags
//!
//! - `input`（デフォルト）：入力プラグイン機能を有効にします。
//! - `output`（デフォルト）：出力プラグイン機能を有効にします。
//! - `filter`（デフォルト）：フィルタプラグイン機能を有効にします。
//! - `module`（デフォルト）：スクリプトモジュールプラグイン機能を有効にします。
//! - `image`：`image`クレートを使用して画像の読み書きをサポートします。
//!
//! # 内部関数について
//!
//! このクレートにて、`__`で始まる名前のモジュールやマクロは、内部実装用です。
//! これらの関数はsemverの保証対象外であり、将来的に変更または削除される可能性があります。
//!
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "aviutl2-alias")]
pub use aviutl2_alias as alias;
pub use aviutl2_sys as sys;
pub use log;

#[doc(inline)]
pub use aviutl2_macros::plugin;

#[cfg(feature = "input")]
pub mod input;

#[cfg(feature = "output")]
pub mod output;

#[cfg(feature = "filter")]
pub mod filter;

#[cfg(feature = "module")]
pub mod module;

#[doc(hidden)]
#[path = "internal_base.rs"]
pub mod __internal_base;

pub mod common;
pub mod logger;
pub mod utils;
pub use anyhow;
pub use common::*;

#[macro_export]
#[doc(hidden)]
macro_rules! __internal_module {
    ($($item:item)*) => {
        const _: () = {
            $($item)*
        };
    }
}
