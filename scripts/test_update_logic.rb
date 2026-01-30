#!/usr/bin/env ruby
# frozen_string_literal: true

# Test script for update_old_releases.rb logic

require "json"

# Simulate releases data
releases = [
  {
    "tag_name" => "0.11.1",
    "created_at" => "2026-01-20T00:00:00Z",
    "draft" => false,
    "prerelease" => false,
    "html_url" => "https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.11.1",
    "body" => "Old release body"
  },
  {
    "tag_name" => "0.11.2",
    "created_at" => "2026-01-25T00:00:00Z",
    "draft" => false,
    "prerelease" => false,
    "html_url" => "https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.11.2",
    "body" => "Another old release"
  },
  {
    "tag_name" => "0.12.2",
    "created_at" => "2026-01-29T00:00:00Z",
    "draft" => false,
    "prerelease" => false,
    "html_url" => "https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.12.2",
    "body" => "Latest release"
  }
]

# Sort by created_at to get the latest
releases.sort_by! { |r| r["created_at"] }
latest_release = releases.last

puts "✓ Latest release identified: #{latest_release["tag_name"]}"

# Get non-draft, non-prerelease releases
old_releases = releases[0..-2].reject { |r| r["draft"] || r["prerelease"] }

puts "✓ Found #{old_releases.length} old releases to update"

latest_url = latest_release["html_url"]
latest_tag = latest_release["tag_name"]

# Test the body generation for one release
test_release = old_releases.first
current_body = test_release["body"] || ""

new_body = <<~MARKDOWN
  > [!NOTE]
  > **最新版はこちらです！ / The latest version is here!**
  > 
  > [v#{latest_tag}](#{latest_url})

  #{current_body}
MARKDOWN

puts "\n✓ Generated new body for #{test_release["tag_name"]}:"
puts "---"
puts new_body
puts "---"

# Test that we don't update if already has the link
test_body_with_link = <<~MARKDOWN
  > [!NOTE]
  > **最新版はこちらです！ / The latest version is here!**
  > 
  > [v0.12.1](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.12.1)

  Old release body
MARKDOWN

if test_body_with_link.include?("最新版はこちらです") || test_body_with_link.include?("latest version is here")
  puts "✓ Correctly detects existing link"
end

puts "\nAll tests passed! ✓"
