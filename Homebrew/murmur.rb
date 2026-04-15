cask "murmur" do
  version "0.1.0"
  sha256 :no_check

  url "https://github.com/OWNER/murmur/releases/download/v#{version}/Murmur_#{version}_aarch64.dmg"
  name "Murmur"
  desc "macOS voice-to-text dictation tool"
  homepage "https://github.com/OWNER/murmur"

  app "Murmur.app"

  zap trash: [
    "~/.config/murmur",
  ]
end
