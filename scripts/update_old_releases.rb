#!/usr/bin/env ruby
# frozen_string_literal: true

require "json"
require "net/http"
require "uri"

def get_releases
  uri = URI("https://api.github.com/repos/sevenc-nanashi/aviutl2-rs/releases")
  request = Net::HTTP::Get.new(uri)
  request["Accept"] = "application/vnd.github+json"
  request["Authorization"] = "Bearer #{ENV["GITHUB_TOKEN"]}"
  request["X-GitHub-Api-Version"] = "2022-11-28"

  response = Net::HTTP.start(uri.hostname, uri.port, use_ssl: true) do |http|
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
  uri = URI("https://api.github.com/repos/sevenc-nanashi/aviutl2-rs/releases/#{release_id}")
  request = Net::HTTP::Patch.new(uri)
  request["Accept"] = "application/vnd.github+json"
  request["Authorization"] = "Bearer #{ENV["GITHUB_TOKEN"]}"
  request["X-GitHub-Api-Version"] = "2022-11-28"
  request["Content-Type"] = "application/json"
  request.body = JSON.generate({ body: body })

  response = Net::HTTP.start(uri.hostname, uri.port, use_ssl: true) do |http|
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

  releases = get_releases
  if releases.empty?
    puts "No releases found"
    return
  end

  # Sort by created_at to get the latest
  releases.sort_by! { |r| r["created_at"] }
  latest_release = releases.last

  puts "Latest release: #{latest_release["tag_name"]}"

  # Get non-draft, non-prerelease releases
  old_releases = releases[0..-2].reject { |r| r["draft"] || r["prerelease"] }

  if old_releases.empty?
    puts "No old releases to update"
    return
  end

  puts "Updating #{old_releases.length} old release(s)..."

  latest_url = latest_release["html_url"]
  latest_tag = latest_release["tag_name"]

  old_releases.each do |release|
    tag_name = release["tag_name"]
    current_body = release["body"] || ""

    # Check if the link already exists
    if current_body.include?("最新版はこちらです") || current_body.include?("latest version is here")
      puts "Skipping #{tag_name}: already has latest version link"
      next
    end

    # Prepend the link to the body
    new_body = <<~MARKDOWN
      > [!NOTE]
      > **最新版はこちらです！ / The latest version is here!**
      > 
      > [v#{latest_tag}](#{latest_url})

      #{current_body}
    MARKDOWN

    puts "Updating #{tag_name}..."
    update_release(release["id"], new_body)
    puts "✓ Updated #{tag_name}"
  end

  puts "Done!"
end

main if __FILE__ == $PROGRAM_NAME
