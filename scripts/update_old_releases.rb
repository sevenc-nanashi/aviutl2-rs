#!/usr/bin/env ruby
# frozen_string_literal: true

# 古いリリースの説明文に最新リリースへのリンクを追加するスクリプト
#
# ```sh
# GITHUB_TOKEN=your_token DRY_RUN=true ruby update_old_releases.rb
# ```

require "octokit"

REPO = "sevenc-nanashi/aviutl2-rs"
MARKER = "<!-- auto-updated-latest-release-link -->"

def build_client
  token = ENV["GITHUB_TOKEN"]
  unless token && !token.empty?
    puts "Error: GITHUB_TOKEN environment variable is not set"
    exit 1
  end

  client = Octokit::Client.new(access_token: token)
  client.auto_paginate = true
  client
end

def fetch_releases(client)
  client.releases(REPO)
rescue Octokit::Error => e
  puts "Failed to fetch releases: #{e.class} #{e.message}"
  exit 1
end

def update_release(client, release_id, body)
  client.edit_release(REPO, release_id, body: body)
rescue Octokit::Error => e
  puts "Failed to update release #{release_id}: #{e.class} #{e.message}"
  exit 1
end

def main
  dry_run = ENV["DRY_RUN"] == "true"
  puts "Running in DRY RUN mode - no changes will be made" if dry_run

  client = build_client
  releases = fetch_releases(client)
  if releases.empty?
    puts "No releases found"
    return
  end

  # Filter to only non-draft, non-prerelease releases
  stable_releases = releases.reject { |r| r[:draft] || r[:prerelease] }

  if stable_releases.empty?
    puts "No stable releases found"
    return
  end

  # Sort by created_at to get the latest stable release
  stable_releases.sort_by! { |r| r[:created_at] }
  latest_release = stable_releases.last

  puts "Latest stable release: #{latest_release[:tag_name]}"

  # Get older stable releases
  old_releases = stable_releases[0..-2]

  if old_releases.empty?
    puts "No old releases to update"
    return
  end

  puts "Updating #{old_releases.length} old release(s)..."

  latest_url = latest_release[:html_url]
  latest_tag = latest_release[:tag_name]

  # Format the tag name for display (add 'v' prefix if not present)
  display_tag = latest_tag.start_with?("v") ? latest_tag : "v#{latest_tag}"

  old_releases.each do |release|
    tag_name = release[:tag_name]
    current_body = release[:body] || ""

    # Check if the link already exists by looking for the marker
    if current_body.include?(MARKER)
      puts "Skipping #{tag_name}: already has latest version link"
      next
    end

    current_body = current_body.sub(/\A.*#{MARKER}\n/m, "").lstrip

    # Prepend the link to the body
    new_body = <<~MARKDOWN
      > [!NOTE]
      > 新しいバージョンがリリースされています！
      >
      > [#{display_tag}](#{latest_url})

      #{MARKER}
      #{current_body}
    MARKDOWN

    puts "Updating #{tag_name}..."
    if dry_run
      puts "Would update with:"
      puts new_body[0..200] + "..."
    else
      update_release(client, release[:id], new_body)
      puts "✓ Updated #{tag_name}"
    end
  end

  puts "Done!"
end

main if __FILE__ == $PROGRAM_NAME
