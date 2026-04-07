#!/usr/bin/env ruby
# frozen_string_literal: true

require "fileutils"

PAGE_DIR = File.expand_path("../examples/statistics-output/page", __dir__)

def run!(*command, **options)
  success = system(*command, **options)
  return if success

  abort("Command failed: #{command.join(" ")}")
end

unless Dir.exist?(PAGE_DIR)
  abort("Statistics page directory not found: #{PAGE_DIR}")
end

FileUtils.mkdir_p(File.join(PAGE_DIR, "dist"))
run!("ni", "--frozen", chdir: PAGE_DIR)
run!("nr", "build", chdir: PAGE_DIR)
