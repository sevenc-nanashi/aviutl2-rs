# frozen_string_literal: true
require "bundler/inline"

suffixes = { "_input" => ".aui2", "_output" => ".auo2", "_filter" => ".auf2" }

def replace_suffix(name, target, suffixes)
  target_suffix = target == "release" ? "" : "_#{target}"
  suffixes.each do |key, value|
    if name.end_with?(key)
      return name.sub(/#{key}$/, "#{target_suffix}#{value}")
    end
  end
  raise "Invalid file name: #{name}"
end

gemfile(true) do
  source "https://rubygems.org"
  gem "tomlrb", "~> 2.0", ">= 2.0.3"
  gem "racc", "~> 1.8", ">= 1.8.1"
end

desc "ビルドしたプラグインをC:/ProgramData/AviUtl2/Pluginまたは指定したディレクトリにインストールします"
task :install, %w[target dest] do |task, args|
  if !(target = args.target)
    puts "Usage: rake install[target[,dest]]"
    puts "Example: rake install[debug]"
    exit 1
  end

  dest_dir = args.dest || "C:/ProgramData/AviUtl2/Plugin"
  Dir.mkdir(dest_dir) unless Dir.exist?(dest_dir)
  Dir
    .glob("./examples/*/Cargo.toml")
    .each do |manifest|
      cargo_toml = Tomlrb.load_file(manifest)
      name = cargo_toml["lib"]["name"]
      file = "./target/#{target}/#{name}.dll"
      dest_name = replace_suffix(name, target, suffixes)
      raise "Invalid file name: #{file}" if dest_name == name
      FileUtils.cp(file, File.join(dest_dir, dest_name), verbose: true)
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
  Dir.mkdir(dest_dir) unless Dir.exist?(dest_dir)
  Dir
    .glob("./examples/*/Cargo.toml")
    .each do |manifest|
      cargo_toml = Tomlrb.load_file(manifest)

      source = "./target/#{target}/#{cargo_toml["lib"]["name"]}.dll"
      dest_name = replace_suffix(cargo_toml["lib"]["name"], target, suffixes)
      raise "Invalid file name: #{source}" if dest_name == File.basename(source)
      from_path = File.absolute_path(source)
      if File.exist?(File.join(dest_dir, dest_name))
        puts "Skip: #{File.join(dest_dir, dest_name)} already exists"
        next
      else
        FileUtils.ln_s(from_path, File.join(dest_dir, dest_name), verbose: true)
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

    ## 説明書
  MARKDOWN
  plugins.each do |lib_name, dir|
    description << "- `#{lib_name}`：<https://github.com/sevenc-nanashi/aviutl2-rs/blob/#{tag}/examples/#{File.basename(dir)}/README.md>\n"
  end

  File.write(File.join(dest_dir, "README.md"), description)
end
