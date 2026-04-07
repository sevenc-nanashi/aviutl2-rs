#!/usr/bin/env ruby
# frozen_string_literal: true

path = File.expand_path("../aviutl2.toml", __dir__)
version =
  ARGV.fetch(0) do
    abort("Usage: ruby ./scripts/set_aviutl2_version.rb <version>")
  end

lines = File.readlines(path, chomp: true)
in_project = false
updated = false

lines.map! do |line|
  in_project = line == "[project]" if line.start_with?("[")

  if in_project && line.start_with?("version = ")
    updated = true
    %(version = "#{version}")
  else
    line
  end
end

abort("Failed to update project.version in aviutl2.toml") unless updated

File.write(path, lines.join("\n") + "\n")
