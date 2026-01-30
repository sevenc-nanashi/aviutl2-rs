#!/usr/bin/env ruby
# frozen_string_literal: true

# 古いリリースの説明文に最新リリースへのリンクを追加するスクリプト
#
# ```sh
# GITHUB_TOKEN=your_token DRY_RUN=true ruby update_old_releases.rb
# ```

require "json"
require "net/http"
require "uri"

def get_releases
  uri = URI("https://api.github.com/repos/sevenc-nanashi/aviutl2-rs/releases")
  request = Net::HTTP::Get.new(uri)
  request["Accept"] = "application/vnd.github+json"
  request["Authorization"] = "Bearer #{ENV["GITHUB_TOKEN"]}"
  request["X-GitHub-Api-Version"] = "2022-11-28"

  response =
    Net::HTTP.start(uri.hostname, uri.port, use_ssl: true) do |http|
      http.request(request)
    end

  unless response.is_a?(Net::HTTPSuccess)
    puts "Failed to fetch releases: #{response.code} #{response.message}"
    puts response.body
    exit 1
  end

  JSON.parse(response.body)
end

def update_release(release_id, body)
  uri =
    URI(
      "https://api.github.com/repos/sevenc-nanashi/aviutl2-rs/releases/#{release_id}"
    )
  request = Net::HTTP::Patch.new(uri)
  request["Accept"] = "application/vnd.github+json"
  request["Authorization"] = "Bearer #{ENV["GITHUB_TOKEN"]}"
  request["X-GitHub-Api-Version"] = "2022-11-28"
  request["Content-Type"] = "application/json"
  request.body = JSON.generate({ body: body })

  response =
    Net::HTTP.start(uri.hostname, uri.port, use_ssl: true) do |http|
      http.request(request)
    end

  unless response.is_a?(Net::HTTPSuccess)
    puts "Failed to update release #{release_id}: #{response.code} #{response.message}"
    puts response.body
    exit 1
  end

  JSON.parse(response.body)
end

def main
  unless ENV["GITHUB_TOKEN"]
    puts "Error: GITHUB_TOKEN environment variable is not set"
    exit 1
  end

  dry_run = ENV["DRY_RUN"] == "true"
  puts "Running in DRY RUN mode - no changes will be made" if dry_run

  releases = get_releases
  if releases.empty?
    puts "No releases found"
    return
  end

  # Filter to only non-draft, non-prerelease releases
  stable_releases = releases.reject { |r| r["draft"] || r["prerelease"] }

  if stable_releases.empty?
    puts "No stable releases found"
    return
  end

  # Sort by created_at to get the latest stable release
  stable_releases.sort_by! { |r| r["created_at"] }
  latest_release = stable_releases.last

  puts "Latest stable release: #{latest_release["tag_name"]}"

  # Get older stable releases
  old_releases = stable_releases[0..-2]

  if old_releases.empty?
    puts "No old releases to update"
    return
  end

  puts "Updating #{old_releases.length} old release(s)..."

  latest_url = latest_release["html_url"]
  latest_tag = latest_release["tag_name"]

  # Format the tag name for display (add 'v' prefix if not present)
  display_tag = latest_tag.start_with?("v") ? latest_tag : "v#{latest_tag}"

  # Marker to detect if the note already exists
  marker = "<!-- auto-updated-latest-release-link -->"

  old_releases.each do |release|
    tag_name = release["tag_name"]
    current_body = release["body"] || ""

    # Check if the link already exists by looking for the marker
    if current_body.include?(marker)
      puts "Skipping #{tag_name}: already has latest version link"
      next
    end

    current_body = current_body.sub(/\A.*#{marker}/m, "").lstrip

    # Prepend the link to the body
    new_body = <<~MARKDOWN
      > [!NOTE]
      > **最新版はこちらです！**
      >
      > [#{display_tag}](#{latest_url})

      #{marker}#{current_body}
    MARKDOWN

    puts "Updating #{tag_name}..."
    if dry_run
      puts "Would update with:"
      puts new_body[0..200] + "..."
    else
      update_release(release["id"], new_body)
      puts "✓ Updated #{tag_name}"
    end
  end

  puts "Done!"
end

main if __FILE__ == $PROGRAM_NAME
