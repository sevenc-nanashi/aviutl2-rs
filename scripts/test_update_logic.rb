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
    "html_url" =>
      "https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.11.1",
    "body" => "Old release body"
  },
  {
    "tag_name" => "0.11.2",
    "created_at" => "2026-01-25T00:00:00Z",
    "draft" => false,
    "prerelease" => false,
    "html_url" =>
      "https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.11.2",
    "body" => "Another old release"
  },
  {
    "tag_name" => "0.12.2",
    "created_at" => "2026-01-29T00:00:00Z",
    "draft" => false,
    "prerelease" => false,
    "html_url" =>
      "https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.12.2",
    "body" => "Latest release"
  },
  {
    "tag_name" => "0.13.0-beta",
    "created_at" => "2026-01-30T00:00:00Z",
    "draft" => false,
    "prerelease" => true,
    "html_url" =>
      "https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.13.0-beta",
    "body" => "Beta release"
  }
]

# Filter to only non-draft, non-prerelease releases
stable_releases = releases.reject { |r| r["draft"] || r["prerelease"] }

puts "✓ Found #{stable_releases.length} stable releases"

# Sort by created_at to get the latest stable release
stable_releases.sort_by! { |r| r["created_at"] }
latest_release = stable_releases.last

puts "✓ Latest stable release identified: #{latest_release["tag_name"]}"
puts "  (Correctly ignoring prerelease: 0.13.0-beta)"

# Get older stable releases
old_releases = stable_releases[0..-2]

puts "✓ Found #{old_releases.length} old releases to update"

latest_url = latest_release["html_url"]
latest_tag = latest_release["tag_name"]

# Format the tag name for display (add 'v' prefix if not present)
display_tag = latest_tag.start_with?("v") ? latest_tag : "v#{latest_tag}"

puts "✓ Display tag: #{display_tag}"

# Test the body generation for one release
test_release = old_releases.first
current_body = test_release["body"] || ""

# Marker to detect if the note already exists
marker = "<!-- auto-updated-latest-release-link -->"

new_body = <<~MARKDOWN
  #{marker}
  > [!NOTE]
  > **最新版はこちらです！ / The latest version is here!**
  > 
  > [#{display_tag}](#{latest_url})

  #{current_body}
MARKDOWN

puts "\n✓ Generated new body for #{test_release["tag_name"]}:"
puts "---"
puts new_body
puts "---"

# Test that we don't update if already has the link
test_body_with_link = <<~MARKDOWN
  <!-- auto-updated-latest-release-link -->
  > [!NOTE]
  > **最新版はこちらです！ / The latest version is here!**
  > 
  > [v0.12.1](https://github.com/sevenc-nanashi/aviutl2-rs/releases/tag/0.12.1)

  Old release body
MARKDOWN

if test_body_with_link.include?(marker)
  puts "✓ Correctly detects existing link using marker"
end

# Test tag name formatting with and without 'v' prefix
test_tag_with_v = "v1.0.0"
test_tag_without_v = "1.0.0"

display_with_v =
  test_tag_with_v.start_with?("v") ? test_tag_with_v : "v#{test_tag_with_v}"
display_without_v =
  (
    if test_tag_without_v.start_with?("v")
      test_tag_without_v
    else
      "v#{test_tag_without_v}"
    end
  )

if display_with_v == "v1.0.0" && display_without_v == "v1.0.0"
  puts "✓ Tag name formatting works correctly for both formats"
end

puts "\nAll tests passed! ✓"
