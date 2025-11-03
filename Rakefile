# frozen_string_literal: true
require "syntax_tree/rake_tasks"
require "tomlrb"

SyntaxTree::Rake::WriteTask.new do |t|
  t.source_files = FileList[%w[./Rakefile]]
end
SyntaxTree::Rake::CheckTask.new do |t|
  t.source_files = FileList[%w[./Rakefile]]
end

suffixes = {
  "_input" => ".aui2",
  "_output" => ".auo2",
  "_filter" => ".auf2",
  "_module" => ".mod2"
}

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

desc "C:/ProgramData/AviUtl2/Pluginまたは指定したディレクトリにビルドしたプラグインのシンボリックリンクを作成します"
task :link, %w[target dest] do |task, args|
  if !(target = args.target)
    puts "Usage: rake link[target[,dest]]"
    puts "Example: rake link[debug]"
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
      if File.exist?(dest_path)
        puts "Skip: #{dest_path} already exists"
        next
      else
        FileUtils.ln_s(from_path, dest_path, verbose: true)
      end
    end
end

desc "リリースアセットを作成します"
task :release, ["tag"] do |task, args|
  if !(tag = args.tag)
    puts "Usage: rake release[tag]"
    puts "Example: rake release[0.1.0]"
    exit 1
  end
  dest_dir = "./release"
  FileUtils.mkdir_p(dest_dir) unless Dir.exist?(dest_dir)
  plugins = {}
  Dir
    .glob("./examples/*")
    .each do |dir|
      cargo_toml = File.join(dir, "Cargo.toml")
      lib_name = Tomlrb.load_file(cargo_toml)["lib"]["name"]
      plugin_name = replace_suffix(lib_name, "release", suffixes)
      plugins[plugin_name] = dir
      FileUtils.cp(
        File.join("target/release", "#{lib_name}.dll"),
        File.join(dest_dir, plugin_name),
        verbose: true
      )
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
    "cargo doc --no-deps -p aviutl2 -p aviutl2-sys -p aviutl2-macros"
  )
end

desc "コードをテストします"
task :test do
  sh "cargo test --all-features"
end
