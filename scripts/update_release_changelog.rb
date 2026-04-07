#!/usr/bin/env ruby
# frozen_string_literal: true

ROOT = File.expand_path("..", __dir__)
RELEASE_DIR = File.join(ROOT, "release")
RELEASE_STAGE_DIR = File.join(ROOT, ".aviutl2-cli", "release-stage")
CHANGELOG_PATH = File.join(ROOT, "CHANGELOG.md")
PACKAGE_INI_PATH = File.join(RELEASE_STAGE_DIR, "package.ini")

def release_version
  package_ini = File.read(PACKAGE_INI_PATH, encoding: "UTF-8")
  version =
    package_ini[/^name=AviUtl2-rs Demo Plugins v([^\r\n]+)$/, 1] ||
      package_ini[/^information=AviUtl2-rs Demo Plugins v([^\r\n]+)$/, 1]
  return version if version

  raise "Failed to detect release version from #{PACKAGE_INI_PATH}"
end

def write_release_changelog(version)
  changelog = File.read(CHANGELOG_PATH, encoding: "UTF-8")
  changelog.sub!(
    "## Unreleased",
    "## [#{version}](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/#{version})"
  )
  changelog.sub!(/(?<=# 変更履歴\n\n)/, <<~MARKDOWN)
    ## Unreleased

    （なし）

    ### デモプラグイン

    （なし）

    MARKDOWN
  File.write(File.join(RELEASE_DIR, "CHANGELOG.md"), changelog)
end

version = release_version
write_release_changelog(version)
