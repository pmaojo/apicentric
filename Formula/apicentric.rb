class Apicentric < Formula
  desc "CLI tool and API simulator platform for developers"
  homepage "https://github.com/pmaojo/apicentric"
  version "0.1.1"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/pmaojo/apicentric/releases/download/v#{version}/apicentric-macos-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_ARM64"
    else
      url "https://github.com/pmaojo/apicentric/releases/download/v#{version}/apicentric-macos-x64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_X64"
    end
  end

  on_linux do
    url "https://github.com/pmaojo/apicentric/releases/download/v#{version}/apicentric-linux-x64.tar.gz"
    sha256 "PLACEHOLDER_SHA256_LINUX_X64"
  end

  def install
    bin.install "apicentric"
  end

  test do
    assert_match "apicentric", shell_output("#{bin}/apicentric --version")
  end
end
