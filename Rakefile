# frozen_string_literal: true
require "bundler/setup"
require "syntax_tree/rake_tasks"
require "tomlrb"
require "fileutils"

SyntaxTree::Rake::WriteTask.new do |t|
  t.source_files = FileList[%w[./Rakefile ./scripts/**/*.rb]]
end
SyntaxTree::Rake::CheckTask.new do |t|
  t.source_files = FileList[%w[./Rakefile ./scripts/**/*.rb]]
end

suffixes = {
  "_input" => ".aui2",
  "_output" => ".auo2",
  "_filter" => ".auf2",
  "_module" => ".mod2",
  "_plugin" => ".aux2"
}
main_crates = %w[
  aviutl2
  aviutl2-sys
  aviutl2-macros
  aviutl2-alias
  aviutl2-eframe
]

def replace_suffix(name, target, suffixes)
  target_suffix = target == "release" ? "" : "_#{target}"
  suffixes.each do |key, value|
    if name.end_with?(key)
      return name.sub(/#{key}$/, "#{target_suffix}#{value}")
    end
  end
  raise "Invalid file name: #{name}"
end

desc "ビルドしたプラグインをC:/ProgramData/AviUtl2/Pluginまたは指定したディレクトリにインストールします"
task :install, %w[target dest] do |task, args|
  if !(target = args.target)
    puts "Usage: rake install[target[,dest]]"
    puts "Example: rake install[debug]"
    exit 1
  end

  dest_dir = args.dest || "C:/ProgramData/AviUtl2/Plugin"
  script_dir = dest_dir + "/../Script"
  Dir.mkdir(dest_dir) unless Dir.exist?(dest_dir)
  Dir.mkdir(script_dir) unless Dir.exist?(script_dir)
  Dir
    .glob("./examples/*/Cargo.toml")
    .each do |manifest|
      cargo_toml = Tomlrb.load_file(manifest)
      unless cargo_toml.key?("lib") &&
               cargo_toml["lib"]["crate-type"]&.include?("cdylib")
        puts "Skip: #{manifest} is not a cdylib"
        next
      end
      name = cargo_toml["lib"]["name"]
      file = "./target/#{target}/#{name}.dll"
      dest_name = replace_suffix(name, target, suffixes)
      raise "Invalid file name: #{file}" if dest_name == name
      if dest_name.end_with?("mod2")
        FileUtils.cp(file, File.join(script_dir, dest_name), verbose: true)
      else
        FileUtils.cp(file, File.join(dest_dir, dest_name), verbose: true)
      end
    end
end

desc "./test_environment下にAviUtl2をセットアップし、debugビルドへのシンボリックリンクを作成します"
task :debug_setup do |task, args|
  require "zip"

  zip_path = "./test_environment/aviutl2_latest.zip"
  mkdir_p("./test_environment") unless Dir.exist?("./test_environment")
  File.open(zip_path, "wb") do |file|
    require "open-uri"
    URI.open("https://api.aviutl2.jp/download?version=latest&type=zip") do |uri|
      file.write(uri.read)
    end
  end
  Zip::File.open(zip_path) do |zip_file|
    zip_file.each do |entry|
      dest_path = File.join("./test_environment", entry.name)
      unless Dir.exist?(File.dirname(dest_path))
        mkdir_p(File.dirname(dest_path))
      end
      rm_rf(dest_path) if File.exist?(dest_path)
      zip_file.extract(entry, dest_path)
    end
  end
  rm(zip_path)

  dest_dir = "./test_environment/data/Plugin"
  script_dir = dest_dir + "/../Script"
  target = "debug"
  FileUtils.mkdir_p(dest_dir) unless Dir.exist?(dest_dir)
  FileUtils.mkdir_p(script_dir) unless Dir.exist?(script_dir)
  Dir
    .glob("./examples/*/Cargo.toml")
    .each do |manifest|
      cargo_toml = Tomlrb.load_file(manifest)
      unless cargo_toml.key?("lib") &&
               cargo_toml["lib"]["crate-type"]&.include?("cdylib")
        puts "Skip: #{manifest} is not a cdylib"
        next
      end

      source = "./target/#{target}/#{cargo_toml["lib"]["name"]}.dll"
      dest_name = replace_suffix(cargo_toml["lib"]["name"], target, suffixes)
      raise "Invalid file name: #{source}" if dest_name == File.basename(source)
      from_path = File.absolute_path(source)
      dest_path =
        if dest_name.end_with?("mod2")
          File.join(script_dir, dest_name)
        else
          File.join(dest_dir, dest_name)
        end
      if File.exist?(dest_path) || File.symlink?(dest_path)
        puts "Skip: #{dest_path} already exists"
        next
      else
        FileUtils.ln_s(from_path, dest_path, verbose: true)
      end
    end
end

desc "リリースアセットを作成します"
task :release, ["tag"] do |task, args|
  require "zip"

  if !(tag = args.tag)
    puts "Usage: rake release[tag]"
    puts "Example: rake release[0.1.0]"
    exit 1
  end
  dest_dir = "./release"
  FileUtils.mkdir_p(dest_dir) unless Dir.exist?(dest_dir)
  plugins = {}
  plugin_files = {}
  Dir
    .glob("./examples/*")
    .each do |dir|
      cargo_toml = Tomlrb.load_file(File.join(dir, "Cargo.toml"))
      unless cargo_toml.key?("lib") &&
               cargo_toml["lib"]["crate-type"]&.include?("cdylib")
        next
      end
      lib_name = cargo_toml["lib"]["name"]
      plugin_name = replace_suffix(lib_name, "release", suffixes)
      plugins[plugin_name] = dir
      source_path = File.join("target/release", "#{lib_name}.dll")
      plugin_files[plugin_name] = source_path
      cp(source_path, File.join(dest_dir, plugin_name))
    end
  zip_path = File.join(dest_dir, "aviutl2-rs.au2pkg.zip")
  rm_f(zip_path)
  puts "Creating zip: #{zip_path}"
  Zip::File.open(zip_path, create: true) do |zip|
    zip.mkdir("Plugin")
    zip.mkdir("Script")
    plugin_files.each do |plugin_name, source_path|
      dest_dir_name = plugin_name.end_with?("mod2") ? "Script" : "Plugin"
      zip.add(File.join(dest_dir_name, plugin_name), source_path)
    end
  end

  description = +<<~MARKDOWN
    # AviUtl2-rs Demo Plugins
    AviUtl2-rsのデモプラグイン集です。
    `C:/ProgramData/AviUtl2/Plugin`に放り込めば動きます。
    ただし、`mod2`は`C:/ProgramData/AviUtl2/Script`に放り込んでください。

    ## 説明書
    変更履歴：<https://github.com/sevenc-nanashi/aviutl2-rs/blob/#{tag}/CHANGELOG.md>
  MARKDOWN
  plugins.each do |lib_name, dir|
    description << "- `#{lib_name}`：<https://github.com/sevenc-nanashi/aviutl2-rs/blob/#{tag}/examples/#{File.basename(dir)}/README.md>\n"
  end

  File.write(File.join(dest_dir, "README.md"), description)

  changelog = File.read("./CHANGELOG.md")
  changelog.sub!(
    "## Unreleased",
    "## [#{tag}](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/#{tag})"
  )
  changelog.sub!(/(?<=# 変更履歴\n\n)/, <<~MARKDOWN)
    ## Unreleased

    （なし）

    ### デモプラグイン

    （なし）

    MARKDOWN
  File.write(File.join(dest_dir, "CHANGELOG.md"), changelog)

  sh "cargo about generate ./about.hbs -o #{File.join(dest_dir, "THIRD_PARTY_NOTICES.md")}"
end

desc "コードをフォーマットします"
task :format do
  Rake::Task["stree:write"].invoke
  sh "cargo fmt"
end

desc "コードのフォーマットをチェックします"
task :check_format do
  Rake::Task["stree:check"].invoke
  sh "cargo fmt -- --check"
end

desc "コードをLintします"
task :lint do
  sh "cargo clippy --all-targets --all-features -- -D warnings"
  sh(
    { "RUSTDOCFLAGS" => "-D warnings" },
    "cargo doc --no-deps #{main_crates.map { |c| "--package #{c}" }.join(" ")}"
  )
end

desc "コードをテストします"
task :test do
  sh "cargo test --all-features"
end

desc "ドキュメントを生成します"
task :doc do
  FileUtils.rm_rf("./target/doc")
  # NOTE:
  # cargo-docs-rsは複数パッケージのドキュメントを一括で生成できないので使わない
  sh(
    { "RUSTDOCFLAGS" => "--cfg docsrs" },
    "cargo +nightly doc --no-deps --all-features #{main_crates.map { |c| "--package #{c}" }.join(" ")}"
  )

  File.write("./target/doc/_redirects", <<~TEXT)
      / /aviutl2/ 308
    TEXT
end
