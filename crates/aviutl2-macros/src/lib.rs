//! # aviutl2-macros
//!
//! [aviutl2-rs](https://docs.rs/aviutl2)のためのproc macroを提供するクレート。
//! 詳細は[aviutl2-rs](https://docs.rs/aviutl2)のドキュメントを参照してください。

mod filter_config_items;
mod filter_config_select_items;
mod from_script_module_param;
mod into_script_module_return_value;
mod module_functions;
mod plugin;
mod utils;

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
/// - `name`: トラックバーの名前。省略した場合、フィールド名が使用されます。
/// - `range`: トラックバーの範囲。閉区間で指定します（例: `0.0..=1.0`）。
/// - `default`: トラックバーの初期値。
/// - `step`: トラックバーのステップ値。`1.0`, `0.1`, `0.01`, `0.001` のいずれかを指定します。
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
/// - `name`: チェックボックスの名前。省略した場合、フィールド名が使用されます。
/// - `default`: チェックボックスの初期値。
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
/// - `name`: 色選択の名前。省略した場合、フィールド名が使用されます。
/// - `default`: 色の初期値。`0xRRGGBB`形式の整数、`"#RRGGBB"`形式の文字列、または`(R, G, B)`形式のタプルで指定します。
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
/// ```rust
/// #[derive(aviutl2::filter::FilterConfigSelectItems)]
/// enum MySelectItem {
///    #[item(name = "Hoge")]
///    Hoge,
///    #[item(name = "Fuga")]
///    Fuga,
/// }
///
/// #[derive(aviutl2_macros::FilterConfigItems)]
/// struct MyConfig {
///     #[select(
///         name = "サンプルセレクトボックス",
///         items = MySelectItem,
///         default = MySelectItem::Hoge
///     )]
///     select_field: MySelectItem,
/// }
/// ```
///
/// - `name`: セレクトボックスの名前。省略した場合、フィールド名が使用されます。
/// - `items`: セレクトボックスの項目のリスト、または`aviutl2::filter::FilterConfigSelectItems`を実装したenumの名前。
/// - `default`: セレクトボックスの初期値。`items`のインデックス、またはenumのVariantを指定します。
///
/// - 値の型は`default`が`items`のインデックスの場合は`usize`、
///   `default`がenumのVariantの場合はそのenumである必要があります。
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
/// - `name`: ファイル選択の名前。省略した場合、フィールド名が使用されます。
/// - `filters`: ファイルフィルタのリスト。キーがフィルタ名、値が拡張子のリストです。
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
///
/// # See Also
///
/// - [`FilterConfigSelectItems`]
#[proc_macro_derive(FilterConfigItems, attributes(track, check, color, select, file))]
pub fn filter_config_items(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    filter_config_items::filter_config_items(item.into())
        .unwrap_or_else(|e| e)
        .into()
}

/// `FilterConfigSelectItems` を自動で実装するためのマクロ。
///
/// # Attributes
///
/// - enumのフィールドはすべて最大1つのitem属性を持つことができます。
/// - enumは値を持つことができません（Unit-only Enumである必要があります）。
///
/// ## `item`
///
/// ```rust
/// # #[derive(aviutl2::filter::FilterConfigSelectItems)]
/// # enum MySelectItem {
/// #[item(name = "hoge")]
/// Hoge,
/// #[item(name = "fuga")]
/// Fuga = 4,
/// Piyo,
/// # }
/// ```
///
/// - `name`: AviUtl2上で表示されるテキスト。省略された場合はVariantの名前になります。
///
/// # Example
///
/// ```rust
/// #[derive(Debug, aviutl2::filter::FilterConfigSelectItems)]
/// enum MySelectItem {
///     #[item(name = "Hoge")]
///     Hoge,
///     #[item(name = "Fuga")]
///     Fuga,
/// }
/// ```
#[proc_macro_derive(FilterConfigSelectItems, attributes(item))]
pub fn filter_config_select_items(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    filter_config_select_items::filter_config_select_items(item.into())
        .unwrap_or_else(|e| e)
        .into()
}

/// `ScriptModuleFunctions` を実装するためのマクロ。
///
/// このマクロは`impl`ブロックに対して適用されます。
/// `impl`ブロック内で定義された関数がスクリプトモジュールの関数として登録されます。
///
/// # Attributes
///
/// ### `direct`
///
/// 関数の引数を手動で処理する関数として登録します。
/// 関数のシグネチャは以下のようになります。
///
/// ```rust
/// fn function_name(params: &mut aviutl2::module::ScriptModuleCallHandle) { /* ... */ }
/// ```
///
/// # Example
///
/// ```rust
/// use aviutl2::module::IntoScriptModuleReturnValue;
///
/// #[aviutl2::plugin(ScriptModule)]
/// struct MyModule {
///     counter: std::sync::atomic::AtomicI32,
/// }
/// # impl aviutl2::module::ScriptModule for MyModule {
/// #     fn new(_info: aviutl2::AviUtl2Info) -> aviutl2::AnyResult<Self> {
/// #         unimplemented!()
/// #     }
/// #     fn plugin_info(&self) -> aviutl2::module::ScriptModuleTable {
/// #         unimplemented!()
/// #     }
/// # }
/// #[aviutl2::module::functions]
/// impl MyModule {
///     fn sum(a: i32, b: i32) -> i32 {
///         a + b
///     }
///
///     fn return_overload(a: i32) -> impl aviutl2::module::IntoScriptModuleReturnValue {
///         if a % 2 == 0 {
///             return "Even".into_return_values().map_err(anyhow::Error::from);
///         } else {
///             return ("Odd", a).into_return_values();
///         }
///     }
///
///     fn increment_counter(&self) -> i32 {
///         self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
///     }
///
///     #[direct]
///     fn direct_sum(params: &mut aviutl2::module::ScriptModuleCallHandle) {
///         let a: i32 = params.get_param(0).unwrap_or(0);
///         let b: i32 = params.get_param(1).unwrap_or(0);
///         params.push_result(a + b);
///     }
///
///     #[direct]
///     fn direct_sum_with_counter(
///         &self,
///         params: &mut aviutl2::module::ScriptModuleCallHandle,
///     ) {
///         let a: i32 = params.get_param(0).unwrap_or(0);
///         let b: i32 = params.get_param(1).unwrap_or(0);
///         let count = self.increment_counter();
///         params.push_result((a + b, count));
///     }
/// }
/// # fn main() {}
/// ```
#[proc_macro_attribute]
pub fn module_functions(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    module_functions::module_functions(item.into())
        .unwrap_or_else(|e| e)
        .into()
}

/// `FromScriptModuleParam` を自動で実装するためのマクロ。
///
/// このマクロを利用するには、構造体の各フィールドが `aviutl2::module::FromScriptModuleParamValue`
/// トレイトを実装している必要があります。
///
/// # Example
///
/// ```rust
/// #[derive(aviutl2::module::FromScriptModuleParam)]
/// struct MyStruct {
///     foo: i32,
///     bar: String,
/// }
/// ```
#[proc_macro_derive(FromScriptModuleParam)]
pub fn from_script_module_param(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    from_script_module_param::from_script_module_param(item.into())
        .unwrap_or_else(|e| e)
        .into()
}

/// `IntoScriptModuleReturnValue` を自動で実装するためのマクロ。
///
/// このマクロを利用するには、構造体の各フィールドが
/// `aviutl2::module::IntoScriptModuleReturnValueValue` トレイトを実装している、かつすべてのフィールドが
/// `T` または `Option<T>` 型である必要があります。
///
/// # Example
///
/// ```rust
/// #[derive(aviutl2::module::IntoScriptModuleReturnValue)]
/// struct MyStruct {
///     foo: String,
///     bar: String,
/// }
/// ```
///
/// # See Also
///
/// - [`FromScriptModuleParam`]
#[proc_macro_derive(IntoScriptModuleReturnValue)]
pub fn into_script_module_return_value(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    into_script_module_return_value::into_script_module_return_value(item.into())
        .unwrap_or_else(|e| e)
        .into()
}

/// プラグインを定義するためのマクロ。
///
/// # Attributes
///
/// - 引数には`InputPlugin`、`OutputPlugin`、`FilterPlugin`、`ScriptModule`のいずれかを指定します。
///
/// # Example
///
/// ```rust
/// #[aviutl2::plugin(OutputPlugin)]
/// struct MyOutputPlugin;
///
/// impl aviutl2::output::OutputPlugin for MyOutputPlugin {
///     // ...
/// #   fn new(_info: aviutl2::AviUtl2Info) -> aviutl2::AnyResult<Self> {
/// #       todo!()
/// #   }
/// #
/// #   fn plugin_info(&self) -> aviutl2::output::OutputPluginTable {
/// #       todo!()
/// #   }
/// #
/// #   fn output(&self, info: aviutl2::output::OutputInfo) -> aviutl2::AnyResult<()> {
/// #       todo!()
/// #   }
/// }
///
/// aviutl2::register_output_plugin!(MyOutputPlugin);
///
/// # fn main() {}
/// ```
#[proc_macro_attribute]
pub fn plugin(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    plugin::plugin(attr.into(), item.into())
        .unwrap_or_else(|e| e)
        .into()
}
