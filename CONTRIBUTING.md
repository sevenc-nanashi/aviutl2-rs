# Contribution Guide

## はじめに：設計思想

このクレートは **定義内の動作** のみを使用して実装することを目指しています。

リバースエンジニアリングなどの手法は強力ではありますが、リバースエンジニアリングによる動作（例えば非公開の内部で使われている関数をフックするなど）は、AviUtl2のバージョンアップ後に動作する保証がありません。
これはアップデートが起こる可能性が非常に低い場合（AviUtl1のように）や、古いバージョンから更新しないことが許容されている場合（任意のゲームのModdingなど）では問題ありませんが、
AviUtl2は現在このプラグインを設計している時点（2025/09/08）では、週に一回のペースで更新が行われています。
よって、リバースエンジニアリングに依存したプラグインは、古いバージョンのAviUtl2のみをサポートすることになり、これはコミュニティに対して有害であると考えています。
そのため、このクレートはAviUtl2 SDKで定義されている動作のみを実装することを目指しており、また定義外の動作を含むPRは拒否される可能性があります。

## コマンド集

- `cargo fmt`：コードをフォーマットします。
- `cargo clippy --all-targets --all-features -- -D warnings`：コードのLintを行います。
- `cargo test`：テストを実行します。
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p aviutl2 -p aviutl2-sys`：ドキュメントの生成とリンクなどのチェックを行います。

- `rake install[debug | release]`：AviUtl2のプラグインディレクトリにビルドしたDLLをコピーします。`debug`または`release`を指定します。
- `rake link[debug | release]`：AviUtl2のプラグインディレクトリにビルドしたDLLへのシンボリックリンクを作成します。`debug`または`release`を指定します。
- `rake release`：`./release`ディレクトリにビルドしたDLLをコピーし、また`./release/README.md`を生成します。

<!--
## 個人用メモ

- `gh workflow run build.yml -F "name=1.2.3"`：新しいバージョンをリリース
-->
