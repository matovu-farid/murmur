cask "wipr" do
  version "0.1.0"
  sha256 :no_check

  url "https://github.com/OWNER/wipr/releases/download/v#{version}/Wipr_#{version}_aarch64.dmg"
  name "Wipr"
  desc "macOS voice-to-text dictation tool"
  homepage "https://github.com/OWNER/wipr"

  app "Wipr.app"

  zap trash: [
    "~/.config/wipr",
  ]
end
