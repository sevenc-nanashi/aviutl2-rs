//! # aviutl2-sys
//!
//! AviUtl2 SDKのヘッダーのRust移植。
//!
//! - [`input2`][]：`input2.h`相当。
//! - [`output2`][]：`output2.h`相当。
//! - [`filter2`][]：`filter2.h`相当。
//!
//! - [`common`][]：共通定義。

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod input2;
pub mod output2;
pub mod filter2;
pub mod common;

