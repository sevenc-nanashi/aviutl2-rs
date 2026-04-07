# frozen_string_literal: true
require "bundler/setup"
require "syntax_tree/rake_tasks"
require "fileutils"
require "tomlrb"
require "shellwords"
require "tempfile"

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

def au2(*args)
  command = ["au2", *args].map { |arg| Shellwords.escape(arg.to_s) }.join(" ")
  sh command
end

def replace_suffix(name, suffixes)
  suffixes.each do |key, value|
    return name.sub(/#{key}$/, "#{value}") if name.end_with?(key)
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
  FileUtils.mkdir_p(dest_dir)
  FileUtils.mkdir_p(script_dir)
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
      dest_name = replace_suffix(name, suffixes)
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
  au2("prepare", "--force")
end

desc "リリースアセットを作成します"
task :release, ["tag"] do |task, args|
  if !(tag = args.tag)
    puts "Usage: rake release[tag]"
    puts "Example: rake release[0.1.0]"
    exit 1
  end
  FileUtils.rm_rf("./release")
  au2("release", "--set-version", tag)
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
