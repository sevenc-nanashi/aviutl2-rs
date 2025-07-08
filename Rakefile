task :default => [:download_sdk]

task :download_sdk do
  require 'open-uri'

  url =  "https://spring-fragrance.mints.ne.jp/aviutl/aviutl2_sdk.zip"
  filename = "aviutl2_sdk.zip"
  File.open(filename, 'wb') do |file|
    file.write(URI.open(url).read)
  end
  puts "Downloaded #{filename} from #{url}"

  sh "powershell -ExecutionPolicy Bypass -Command \"Expand-Archive -Path '#{filename}' -DestinationPath sdk/aviutl2_sdk\""
end
