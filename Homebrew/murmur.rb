class Murmur < Formula
  desc "macOS voice-to-text dictation daemon"
  homepage "https://github.com/matovu-farid/murmur"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/matovu-farid/murmur/releases/download/v#{version}/murmur-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ARM64_SHA256"
    end

    on_intel do
      url "https://github.com/matovu-farid/murmur/releases/download/v#{version}/murmur-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_X86_64_SHA256"
    end
  end

  def install
    bin.install "murmur"
  end

  def caveats
    <<~EOS
      Murmur needs Input Monitoring permission to detect the fn key.
      Open System Settings → Privacy & Security → Input Monitoring,
      then enable Murmur after first run.

      To start the daemon:
        murmur start

      To install for auto-start on login:
        murmur install

      To configure interactively:
        murmur config
    EOS
  end

  test do
    assert_match "murmur", shell_output("#{bin}/murmur --version")
  end
end
