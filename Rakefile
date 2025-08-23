# frozen_string_literal: true
require "bundler/inline"

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
  suffix = target == "release" ? "" : "_#{target}"
  Dir.mkdir(dest_dir) unless Dir.exist?(dest_dir)
  Dir
    .glob("./target/#{target}/*.dll")
    .each do |file|
      dest_name =
        File
          .basename(file)
          .sub(/_output\.dll$/, "#{suffix}.auo2")
          .sub(/_input\.dll$/, "#{suffix}.aui2")
      raise "Invalid file name: #{file}" if dest_name == File.basename(file)
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
  suffix = target == "release" ? "" : "_#{target}"
  Dir.mkdir(dest_dir) unless Dir.exist?(dest_dir)
  Dir
    .glob("./target/#{target}/*.dll")
    .each do |file|
      dest_name =
        File
          .basename(file)
          .sub(/_output\.dll$/, "#{suffix}.auo2")
          .sub(/_input\.dll$/, "#{suffix}.aui2")
      from_path = File.absolute_path(file)
      raise "Invalid file name: #{file}" if dest_name == File.basename(file)
      FileUtils.ln_s(from_path, File.join(dest_dir, dest_name), verbose: true)
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
      plugin_name =
        if lib_name.end_with?("_output")
          "#{lib_name.delete_suffix("_output")}.auo2"
        elsif lib_name.end_with?("_input")
          "#{lib_name.delete_suffix("_input")}.aui2"
        else
          raise "Invalid library name: #{lib_name}"
        end
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
