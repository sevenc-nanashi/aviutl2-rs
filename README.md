# aviutl2-rs

AviUtl2 SDKのRust版です。

> [!NOTE]
> これは自分の好奇心で作成したものであり、パフォーマンスやユーザー数などの問題から、使用することは推奨されていません。
> 基本的には枯れているAviUtl1のプラグインを使用することをお勧めします。

## 構成

- `./crates/`：AviUtl2 SDKのRustクレート。
    - `./crates/aviutl2-sys`：AviUtl2 SDKのRustバインディング。ほぼ1:1の対応です。
    - `./crates/aviutl2`：AviUtl2 SDKのRustラッパー。`aviutl2-sys`を使用して、よりRustらしいAPIを提供します。

- `./examples/`：AviUtl2 SDKを使用したサンプルコード。
    - `./examples/image-rs-input`：image-rsを使用して画像を読み込むサンプル。
    - `./examples/image-rs-output`：image-rsを使用して画像を連番で保存するサンプル。
    - `./examples/midi-player-input`：midlyを使用してMIDIを読み込み、ピアノで再生するサンプル。
    - `./examples/pixel-format-test-input`：それぞれのピクセルフォーマットをテストするサンプル。
    - `./examples/ffmpeg-output`：FFmpegを使用して動画を出力するサンプル。
    - `./examples/statistics-output`：フレームごとの描画時間を計測し、統計情報を出力するサンプル。

## ライセンス

MIT Licenseで公開されています。

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
