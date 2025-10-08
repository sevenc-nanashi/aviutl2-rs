//! # aviutl2-macros
//!
//! [aviutl2-rs](https://docs.rs/aviutl2)のためのproc macroを提供するクレート。
//! 詳細は[aviutl2-rs](https://docs.rs/aviutl2)のドキュメントを参照してください。

mod filter_config_items;

/// `FilterConfigItems` を自動で実装するためのマクロ。
///
/// # Attributes
///
/// - structのフィールドはすべてちょうど1つの属性を持つ必要があります。
///
/// ## `track`
///
/// ```rust
/// # #[derive(aviutl2_macros::FilterConfigItems)]
/// # struct S {
/// #[track(name = "サンプル整数", range = 0..=100, default = 50, step = 1.0)]
/// int_field: i32,
/// #[track(name = "サンプル小数", range = 0.0..=1.0, default = 0.5, step = 0.01)]
/// float_field: f64,
/// # }
/// ```
///
/// `name` - トラックバーの名前。省略した場合、フィールド名が使用されます。
/// `range` - トラックバーの範囲。閉区間で指定します（例: `0.0..=1.0`）。
/// `default` - トラックバーの初期値。
/// `step` - トラックバーのステップ値。`1.0`, `0.1`, `0.01`, `0.001` のいずれかを指定します。
///
/// - `range`、`default`は`step`で割り切れる値である必要があります。
/// - 値の型はプリミティブ、厳密には`value as _`で変換可能な型である必要があります。
///
/// ## `check`
///
/// ```rust
/// # #[derive(aviutl2_macros::FilterConfigItems)]
/// # struct S {
/// #[check(name = "サンプルチェックボックス", default = true)]
/// bool_field: bool,
/// # }
/// ```
///
/// `name` - チェックボックスの名前。省略した場合、フィールド名が使用されます。
/// `default` - チェックボックスの初期値。
///
/// - 値の型は`bool`である必要があります。
///
/// ## `color`
///
/// ```rust
/// # #[derive(aviutl2_macros::FilterConfigItems)]
/// # struct S {
/// #[color(name = "サンプルカラー", default = 0x48b0d5)]
/// color_field: aviutl2::filter::FilterConfigColorValue,
/// #[color(name = "サンプルカラー2", default = "#48b0d5")]
/// color_field2: aviutl2::filter::FilterConfigColorValue,
/// #[color(name = "サンプルカラー3", default = (72, 176, 213))]
/// color_field3: aviutl2::filter::FilterConfigColorValue,
/// # }
/// ```
///
/// `name` - 色選択の名前。省略した場合、フィールド名が使用されます。
/// `default` - 色の初期値。`0xRRGGBB`形式の整数、`"#RRGGBB"`形式の文字列、または`(R, G, B)`形式のタプルで指定します。
///
/// - 値の型は`From<aviutl2::filter::FilterConfigColorValue>`を実装している必要があります。
///
/// ## `select`
///
/// ```rust
/// # #[derive(aviutl2_macros::FilterConfigItems)]
/// # struct S {
/// #[select(
///     name = "サンプルセレクトボックス",
///     items = ["オプション1", "オプション2", "オプション3"],
///     default = 0
/// )]
/// select_field: usize,
/// # }
/// ```
///
/// `name` - セレクトボックスの名前。省略した場合、フィールド名が使用されます。
/// `items` - セレクトボックスの項目のリスト。
/// `default` - セレクトボックスの初期値。`items`のインデックスで指定します。
///
/// - 値の型は`usize`、または`usize as _`で変換可能な型である必要があります。
///
/// ## `file`
///
/// ```rust
/// # #[derive(aviutl2_macros::FilterConfigItems)]
/// # struct S {
/// #[file(name = "サンプルファイル", filters = {
///     "テキストファイル" => ["txt"],
///     "すべてのファイル" => []
/// })]
/// file_field: Option<std::path::PathBuf>,
/// # }
/// ```
///
/// `name` - ファイル選択の名前。省略した場合、フィールド名が使用されます。
/// `filters` - ファイルフィルタのリスト。キーがフィルタ名、値が拡張子のリストです。
///
/// - 値の型は`Option<std::path::PathBuf>`である必要があります。
///
/// # Example
///
/// ```rust
/// #[derive(Debug, aviutl2::filter::FilterConfigItems)]
/// struct FilterConfig {
///     #[track(name = "サンプル整数", range = -100..=100, default = 0, step = 1.0)]
///     sample_integer: i32,
///     #[track(name = "サンプル小数", range = -1.0..=1.0, default = 0.0, step = 0.01)]
///     sample_float: f64,
///     #[check(name = "サンプルチェックボックス", default = true)]
///     sample_checkbox: bool,
///     #[select(
///         name = "サンプルセレクトボックス",
///         items = ["オプション1", "オプション2", "オプション3"],
///         default = 0
///     )]
///     sample_select: usize,
///     #[color(name = "サンプルカラー", default = 0x48b0d5)]
///     sample_color: aviutl2::filter::FilterConfigColorValue,
///     #[file(name = "サンプルファイル", filters = {
///         "テキストファイル" => ["txt"],
///         "すべてのファイル" => [],
///     })]
///     sample_file: Option<std::path::PathBuf>,
/// }
/// ```
#[proc_macro_derive(FilterConfigItems, attributes(track, check, color, select, file))]
pub fn filter_config_items(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    filter_config_items::filter_config_items(item.into())
        .unwrap_or_else(|e| e)
        .into()
}
