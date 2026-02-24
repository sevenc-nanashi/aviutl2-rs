# 変更履歴

## Unreleased

- **Breaking**: デバッグビルドでの`(Debug)`付与を削除

### デモプラグイン

（なし）

## [0.18.1](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.18.1)

- generic: `GlobalEditHandle::is_ready`を追加
- generic: `EditHandle::is_ready`を追加
- generic: `EditHandle`に`std::hash::Hash`を実装

### デモプラグイン

（なし）

## [0.18.0](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.18.0)

- **Breaking**: 最小バージョンをbeta34に引き上げ
- **Breaking**: `aviutl2::config::translate`の戻り値を`Result<String>`から`String`に変更
- output: `aviutl2::output::OutputType::Image`を追加
- eframe: persistence feature flagを追加
- generic: デフォルトでエラー時にビープ音を鳴らすように
- generic: `EditSection::objects_in_layer`を追加

### デモプラグイン

- image-rs-input: アルファ値周りの処理を修正
- image-rs-single-outputを追加（静止画出力）

## [0.17.0](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.17.0)

- **Breaking**: 最小バージョンをbeta33に引き上げ
- **Breaking**: generic: `try_get_edit_info`を削除
- generic: [beta33の変更](https://github.com/aviutl2/aviutl2_sdk_mirror/commit/c5a364cffbab058bed2b4aa8fd5c4931e3c3f817)に追従

### デモプラグイン

（なし）

## [0.16.0](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.16.0)

- **Breaking**: 依存関係を更新
- **Breaking**: デフォルトのエラーダイアログを削除
- **Breaking**: generic: register_menu系のコールバックをFnに変更
- **Breaking**: aviutl2-macros: `generic::menus`の引数から`&mut EditSection`を削除
- **Breaking**: aviutl2-macros: `error="alert"`を削除
- generic: `get_host_app_window`、`get_host_app_window_raw`を追加
- eframe: 終了処理を改善

### デモプラグイン

- metronome、local-alias、scripts-search: ヘッダーを折りたためるように

## [0.15.1](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.15.1)

- aviutl2-eframe: ウィンドウ作成時の挙動を改善

### デモプラグイン

（なし）

## [0.15.0](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.15.0)

- **Breaking**: パスを受け取る箇所で`AsRef<std::path::Path>`を使うように

### デモプラグイン

（なし）

## [0.14.0](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.14.0)

- **Breaking**: aviutl2の最小サポートをbeta31に引き上げ
- **Breaking**: aviutl2-eframe: `new_with_force_kill_timeout`を削除
- **Breaking**: config: `get_color_code`の戻り値をOptionに変更
- config: `get_all_color_codes`を追加

### デモプラグイン

（なし）

## [0.13.0](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.13.0)

- **Breaking**: aviutl2-eframe: eframeを0.33.3に更新
- **Breaking**: filter: `file`、`folder`属性の型を`Option<std::path::PathBuf>`または`std::path::PathBuf`に変更
- **Breaking**: filter: `button`属性の戻り値を`anyhow::Result<()>`に変更
- **Breaking**: generic: `SubPlugin::new`の引数を`&AviUtl2Info`に変更
- **Breaking**: `au2pkg.zip`の名前にバージョンを含めるように
- aviutl2-alias: get_tableで`.`を使うと子テーブルを辿れるように
- generic: `try_get_edit_info`、`get_effect_item_parsed`を追加
- `registered_effect_name`を追加
- package.txtを追加

### デモプラグイン

- scripts-search-plugin: エフェクトを読み込んでもマウスオーバーしないと名前が表示されなかったのを修正
- 英語翻訳を追加
- 再起動できなくなるバグを修正

## [0.12.2](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.12.2)

（なし）

### デモプラグイン

- pixelsort: 高速化
- local-alias、scripts-search: 同梱するM+を変更

## [0.12.1](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.12.1)

- **Breaking**: beta30a未満を明示的に弾くように
- **Breaking**: generic: get_effect_namesをget_effectsに変更
- module: i32・f64以外にもFromScriptModuleParamを実装
- generic: enumerate_modulesを追加
- aviutl2-eframeを追加
- config2.h相当を実装
- ログを自動で折り返すように
- デフォルトでpanicしたときにunwindするように

### デモプラグイン

- pixelsort: ソートを高速化
- Scripts Search Pluginを追加
- aviutl2-rs.au2pkg.zipを追加

## [0.11.3](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.11.3)

- module: functionsでparamsという引数名が使えなかったのを修正

### デモプラグイン

- Restart Shortcut Pluginを追加

## [0.11.2](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.11.2)

- module: ユーザーポインタを取得・返却できるように

### デモプラグイン

（なし）

## [0.11.1](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.11.1)

- **Breaking:** generic: ProjectFileのserdeの読み書きの戻り値をanyhow::Resultから変更
- generic: ProjectFileのserdeの読み書きのzstdを削除

### デモプラグイン

- local-alias: トップページが表示されないバグを修正

## [0.11.0](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.11.0)

- **Breaking:** APIからのResultはanyhow::Resultを使わないように変更
- **Breaking:** `odbg!`、`oprintln!`を削除
- **Breaking:** module: `IntoScriptModuleReturnValue` traitの返り値をanyhow::Errorから変更
- **Breaking:** module: `&[T]`、`[T]`は`Vec<T>`として返すように変更
- **Breaking:** filter: `FilterPluginTable::filter_type`、`as_object`を`FilterPluginTable::flags`に統合
- **Breaking:** filter: `#[derive(FilterConfigItems)]`を`#[aviutl2::filter::filter_config_items]`に変更
- **Breaking:** filter: `#[aviutl2::filter::filter_config_items]`の`#[path]`が`std::path::PathBuf`を受け取るように変更
- 汎用プラグインを追加
- `log`クレートで本体のログに出力できるように
- aviutl2-aliasを追加
- filter: `ObjectInfo::effect_id`、`FilterProcVideo::get_image_texture2d`、`FilterProcVideo::get_framebuffer_texture2d`を追加
- filter: `#[aviutl2::filter::filter_config_items]`に`#[data]`、`#[group]`、`#[string]`、`#[text]`、`#[folder]`属性を追加

### デモプラグイン

- binaural、equalizer: effect_idに対応
- wgpu: バグが多かったので削除

## [0.10.0](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.10.0)

- **Breaking:** AviUtl2Version::newの引数をu32からu8に変更
- **Breaking:** プラグイン定義に`#[aviutl2::plugin(...)]`属性を必須化
- **Breaking:** VideoFrame系を`aviutl2::output::video_frame`モジュールに移動
- スクリプトモジュールを追加

### デモプラグイン

- ユーザー名を取得するスクリプトモジュールを追加

## [0.9.6](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.9.6)

- ドキュメントのビルドが通らなかったのを修正

### デモプラグイン

- pixelsort: パフォーマンスを向上

## [0.9.5](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.9.5)

（なし）

### デモプラグイン

- binaural: 今度こそクラッシュを修正

## [0.9.4](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.9.4)

（なし）

### デモプラグイン

- binaural: クラッシュを修正

## [0.9.2](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.9.2)

- 公開手順を修正

### デモプラグイン

（なし）

## <s>[0.9.1](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.9.1)</s>

- **Breaking:** MSRVを1.90.0に
- crates.ioへの公開を同時に行うようにする

### デモプラグイン

（なし）

## <s>[0.9.0](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.9.0)</s>

- **Breaking:** AviUtl2の最小サポートをbeta14に引き上げ
- **Breaking:** aviutl2-sysの共通定義をaviutl2_sys::commonに移動
- **Breaking:** `InputPlugin::new`、`OutputPlugin::new`の引数に`AvUtl2Info`を追加

- 変更履歴を追加
- フィルタプラグインを追加
- `aviutl2-macros`クレートを追加
- `aviutl2::file_filters!`マクロを追加

### デモプラグイン

- **Breaking:** 名前を`Rusty [...] (Input|Output|Filter)`に変更
