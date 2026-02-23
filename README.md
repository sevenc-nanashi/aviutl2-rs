# aviutl2-rs

AviUtl2 SDKのRust版です。

紹介動画：<https://www.nicovideo.jp/watch/sm45355531>

> [!WARNING]
> このcrateはできてから日が浅く、まだ不安定です。APIが大きく変わる可能性があります。

## 構成

- `./crates/`：ライブラリとして提供するクレート群。
  - [`./crates/aviutl2`](https://crates.io/crates/aviutl2)：AviUtl2 SDKのRustラッパー。`aviutl2-sys`を使用して、よりRustらしいAPIを提供します。
  - [`./crates/aviutl2-sys`](https://crates.io/crates/aviutl2-sys)：AviUtl2 SDKのRustバインディング。ほぼ1:1の対応です。
  - [`./crates/aviutl2-macros`](https://crates.io/crates/aviutl2-macros)：`aviutl2`で使用するマクロを提供します。
  - [`./crates/aviutl2-alias`](https://crates.io/crates/aviutl2-alias)：プロジェクトファイル（`*.aup2`）とエイリアスファイル（`*.object`、`*.effect`）で使われているデータ構造を読み書きするクレート。
  - [`./crates/aviutl2-eframe`](https://crates.io/crates/aviutl2-eframe)：汎用プラグインでeframe（egui）を使用するためのクレート。

- `./examples/`：AviUtl2 SDKを使用したサンプルコード。
  - 入力プラグイン：
    - [`./examples/image-rs-input`](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/sevenc-nanashi.aviutl2-rs.rusty_image_rs_input)：image-rsを使用して画像を読み込むサンプル。
    - [`./examples/midi-player-input`](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/sevenc-nanashi.aviutl2-rs.rusty_midi_player)：midlyを使用してMIDIを読み込み、ピアノで再生するサンプル。
    - `./examples/pixel-format-test-input`：それぞれのピクセルフォーマットをテストするサンプル。
  - 出力プラグイン：
    - [`./examples/image-rs-output`](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/sevenc-nanashi.aviutl2-rs.rusty_image_rs_output)：image-rsを使用して画像を連番で保存するサンプル。
    - [`./examples/image-rs-single-output`](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/sevenc-nanashi.aviutl2-rs.rusty_image_rs_single_output)：image-rsを使用して画像を静止画で保存するサンプル。
    - [`./examples/ffmpeg-output`](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/sevenc-nanashi.aviutl2-rs.rusty_ffmpeg)：FFmpegを使用して動画を出力するサンプル。
    - [`./examples/statistics-output`](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/sevenc-nanashi.aviutl2-rs.rusty_statistics)：フレームごとの描画時間を計測し、統計情報を出力するサンプル。
  - フィルタプラグイン（フィルタ効果）：
    - [`./examples/binaural-filter`](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/sevenc-nanashi.aviutl2-rs.rusty_binaural)：hrtfクレートを使用してバイノーラルなパン振りを実装したサンプル。
    - [`./examples/equalizer-filter`](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/sevenc-nanashi.aviutl2-rs.rusty_equalizer)：biquadクレートを使用してイコライザーフィルタを実装したサンプル。
    - [`./examples/pixelsort-filter`](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/sevenc-nanashi.aviutl2-rs.rusty_pixelsort)：ピクセルソートフィルタのサンプル。
  - フィルタプラグイン（カスタムオブジェクト）：
    - [`./examples/chiptune-filter`](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/sevenc-nanashi.aviutl2-rs.rusty_chiptune)：チップチューンの音を生成するサンプル。
    - [`./examples/random-color-filter`](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/sevenc-nanashi.aviutl2-rs.rusty_random_color)：ランダムな色を表示するサンプル。
  - スクリプトモジュール：
    - `./examples/username-module`：ユーザー名を取得するスクリプトモジュールのサンプル。
  - 汎用プラグイン：
    - [`./examples/metronome-plugin`](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/sevenc-nanashi.aviutl2-rs.rusty_metronome_plugin)：クリックに合わせてBPMを調整したり、メトロノーム音を再生したりする汎用プラグインのサンプル。
    - [`./examples/local-alias-plugin`](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/sevenc-nanashi.aviutl2-rs.rusty_local_alias_plugin)：プロジェクトローカルにオブジェクトを保存する汎用プラグインのサンプル。
    - [`./examples/srt-file-plugin`](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/sevenc-nanashi.aviutl2-rs.rusty_srt_file)：SRTファイルをインポート/エクスポートする汎用プラグインのサンプル。
    - [`./examples/restart-shortcut-plugin`](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/sevenc-nanashi.aviutl2-rs.rusty_restart_shortcut)：AviUtl2を再起動する編集メニューを追加する汎用プラグインのサンプル。
    - [`./examples/scripts-search-plugin`](https://aviutl2-catalog-badge.sevenc7c.workers.dev/package/sevenc-nanashi.aviutl2-rs.rusty_scripts_search)：オブジェクト・エフェクトを検索してタイムラインに配置する汎用プラグインのサンプル。

## ライセンス

MIT Licenseで公開されています。
このcrateを使用してプラグインを作ったり、サンプルのプラグインを使って動画を作ったりした際は動画やコモンズに`sm45355531`を親登録していただけると嬉しいです。（任意）

また、これはAviUtl2 SDKに基づいています。

<details>
<summary>AviUtl2 SDK License</summary>

```
---------------------------------
AviUtl ExEdit2 Plugin SDK License
---------------------------------

The MIT License

Copyright (c) 2025 Kenkun

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
```

</details>
