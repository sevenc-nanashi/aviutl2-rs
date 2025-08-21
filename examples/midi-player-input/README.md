# Rusty Sine Midi Player Input

入力プラグインのサンプルです。
[midly](https://docs.rs/midly/latest/midly/)を使用してMIDIを読み込み、sin波で再生します。

## 使い方

- トラック0はすべてのトラックを再生します。
- トラック1以降は、対応するトラック番号のMIDIイベントを再生します。

## インストール

`C:\ProgramData\aviutl2\Plugin` に `rusty_sine_midi_player.aui2` を配置してください。

`./piano.sf2` は CC-BY 3.0 でリリースされている [Yamaha YPT 220 soundfont studio version by TheSoundfontMaker](https://musical-artifacts.com/artifacts/2573) を使用しています。
