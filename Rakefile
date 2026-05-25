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
