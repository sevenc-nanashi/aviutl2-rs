# aviutl2-rs

AviUtl2 SDKのRust版です。
紹介動画：<https://www.nicovideo.jp/watch/sm45355531>

> [!WARNING]
> このcrateはできてから日が浅く、まだ不安定です。APIが大きく変わる可能性があります。

## 構成

- `./crates/`：ライブラリとして提供するクレート群。
  - `./crates/aviutl2`：AviUtl2 SDKのRustラッパー。`aviutl2-sys`を使用して、よりRustらしいAPIを提供します。
  - `./crates/aviutl2-sys`：AviUtl2 SDKのRustバインディング。ほぼ1:1の対応です。
  - `./crates/aviutl2-macros`：`aviutl2`で使用するマクロを提供します。
  - `./crates/aviutl2-alias`：プロジェクトファイル（`*.aup2`）とエイリアスファイル（`*.object`、`*.effect`）で使われているデータ構造を読み書きするクレート。

- `./examples/`：AviUtl2 SDKを使用したサンプルコード。
  - 入力プラグイン：
    - `./examples/image-rs-input`：image-rsを使用して画像を読み込むサンプル。
    - `./examples/midi-player-input`：midlyを使用してMIDIを読み込み、ピアノで再生するサンプル。
    - `./examples/pixel-format-test-input`：それぞれのピクセルフォーマットをテストするサンプル。
  - 出力プラグイン：
    - `./examples/image-rs-output`：image-rsを使用して画像を連番で保存するサンプル。
    - `./examples/ffmpeg-output`：FFmpegを使用して動画を出力するサンプル。
    - `./examples/statistics-output`：フレームごとの描画時間を計測し、統計情報を出力するサンプル。
  - フィルタプラグイン（フィルタ効果）：
    - `./examples/binaural-filter`：hrtfクレートを使用してバイノーラルなパン振りを実装したサンプル。
    - `./examples/equalizer-filter`：biquadクレートを使用してイコライザーフィルタを実装したサンプル。
    - `./examples/pixelsort-filter`：ピクセルソートフィルタのサンプル。
  - フィルタプラグイン（カスタムオブジェクト）：
    - `./examples/chiptune-filter`：チップチューンの音を生成するサンプル。
    - `./examples/random-color-filter`：ランダムな色を表示するサンプル。
  - スクリプトモジュール：
    - `./examples/username-module`：ユーザー名を取得するスクリプトモジュールのサンプル。

## ライセンス

MIT Licenseで公開されています。
このcrateを使用してプラグインを作った場合は紹介動画に`sm45355531`を親登録していただけると嬉しいです。（必須ではありません）

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
