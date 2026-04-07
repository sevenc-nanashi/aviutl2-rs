#!/usr/bin/env ruby
# frozen_string_literal: true

require "fileutils"

OUTPUT_PATH = File.expand_path("../.aviutl2-cli/package.txt", __dir__)
RELEASE_DIR = File.expand_path("../release", __dir__)

FileUtils.rm_rf(RELEASE_DIR)
FileUtils.mkdir_p(RELEASE_DIR)
FileUtils.mkdir_p(File.dirname(OUTPUT_PATH))

success =
  system("cargo", "about", "generate", "./about.package.hbs", "-o", OUTPUT_PATH)
abort("Failed to generate package.txt") unless success

content = File.binread(OUTPUT_PATH).gsub("\r\n", "\n").gsub("\n", "\r\n")
File.binwrite(OUTPUT_PATH, content)
