#!/usr/bin/env ruby
# frozen_string_literal: true

require "fileutils"
require "tomlrb"

ROOT = File.expand_path("..", __dir__)
RELEASE_DIR = File.join(ROOT, "release")
RELEASE_STAGE_DIR = File.join(ROOT, ".aviutl2-cli", "release-stage")
PACKAGE_INI_PATH = File.join(RELEASE_STAGE_DIR, "package.ini")
PACKAGE_TXT_PATH = File.join(RELEASE_STAGE_DIR, "package.txt")

SUFFIXES = {
  "_input" => ".aui2",
  "_output" => ".auo2",
  "_filter" => ".auf2",
  "_module" => ".mod2",
  "_plugin" => ".aux2"
}.freeze

def replace_suffix(name)
  SUFFIXES.each do |key, value|
    return name.sub(/#{Regexp.escape(key)}$/, value) if name.end_with?(key)
  end
  raise "Invalid file name: #{name}"
end

def release_version
  package_ini = File.read(PACKAGE_INI_PATH, encoding: "UTF-8")
  version =
    package_ini[/^name=AviUtl2-rs Demo Plugins v([^\r\n]+)$/, 1] ||
      package_ini[/^information=AviUtl2-rs Demo Plugins v([^\r\n]+)$/, 1]
  return version if version

  raise "Failed to detect release version from #{PACKAGE_INI_PATH}"
end

def plugins
  Dir
    .glob(File.join(ROOT, "examples", "*", "Cargo.toml"))
    .sort
    .filter_map do |manifest|
      cargo_toml = Tomlrb.load_file(manifest)
      lib = cargo_toml["lib"]
      next unless lib && Array(lib["crate-type"]).include?("cdylib")

      dir = File.basename(File.dirname(manifest))
      [replace_suffix(lib.fetch("name")), dir]
    end
end

def write_readme(version)
  description = +<<~MARKDOWN
    # AviUtl2-rs Demo Plugins

    AviUtl2-rsのデモプラグイン集です。
    `aviutl2-rs-v#{version}.au2pkg.zip` はすべてのプラグインをまとめたパッケージです。プレビュー画面にドラッグアンドドロップしてインストールできます。

    また、個別にプラグインをダウンロードしてインストールすることもできます。
    `C:/ProgramData/AviUtl2/Plugin`に放り込んでください。
    ただし、`mod2`は`C:/ProgramData/AviUtl2/Script`に放り込んでください。

    これらのプラグインを使って動画を作ったりした際は動画やコモンズに`sm45355531`を親登録していただけると嬉しいです。（任意）

    ## 説明書
    変更履歴：<https://github.com/sevenc-nanashi/aviutl2-rs/blob/#{version}/CHANGELOG.md>
  MARKDOWN

  plugins.each do |plugin_name, dir|
    description << "- `#{plugin_name}`：<https://github.com/sevenc-nanashi/aviutl2-rs/blob/#{version}/examples/#{dir}/README.md>\n"
  end

  File.write(File.join(RELEASE_DIR, "README.md"), description)
end

def copy_release_assets
  [PACKAGE_INI_PATH, PACKAGE_TXT_PATH].each do |path|
    next unless File.file?(path)

    FileUtils.cp(path, File.join(RELEASE_DIR, File.basename(path)))
  end

  %w[Plugin Script Language].each do |dir_name|
    source_dir = File.join(RELEASE_STAGE_DIR, dir_name)
    next unless Dir.exist?(source_dir)

    Dir
      .glob(File.join(source_dir, "*"))
      .sort
      .each do |path|
        next unless File.file?(path)

        FileUtils.cp(path, File.join(RELEASE_DIR, File.basename(path)))
      end
  end
end

def write_third_party_notices
  success =
    system(
      "cargo",
      "about",
      "generate",
      "./about.hbs",
      "-o",
      File.join(RELEASE_DIR, "THIRD_PARTY_NOTICES.md")
    )
  abort("Failed to generate THIRD_PARTY_NOTICES.md") unless success
end

FileUtils.mkdir_p(RELEASE_DIR)
version = release_version
copy_release_assets
write_readme(version)
write_third_party_notices
