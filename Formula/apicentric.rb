class Apicentric < Formula
  desc "CLI tool and API simulator platform for developers"
  homepage "https://github.com/pmaojo/apicentric"
  version "1.0.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/pmaojo/apicentric/releases/download/v#{version}/apicentric-macos-arm64.tar.gz"
      sha256 "46f92400dde3e96f24efdada2d7dd8f868fd0c916e967dd9ec4517da5eec35f4"
    else
      url "https://github.com/pmaojo/apicentric/releases/download/v#{version}/apicentric-macos-x64.tar.gz"
      sha256 "7974297d5820ce6e300a6d243be45eb40ae325a749a885c8bf7264a6029714d8"
    end
  end

  on_linux do
    url "https://github.com/pmaojo/apicentric/releases/download/v#{version}/apicentric-linux-x64.tar.gz"
    sha256 "25b59eb8ffd7ede14c6481e2f42c1109e4f0eb4e73b517170ab0a85117cff309"
  end

  def install
    bin.install "apicentric"
  end

  test do
    assert_match "apicentric", shell_output("#{bin}/apicentric --version")
  end
end
