class Apicentric < Formula
  desc "API mocking and simulation toolkit"
  homepage "https://github.com/pmaojo/apicentric"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/pmaojo/apicentric/releases/download/v#{version}/apicentric-macos-arm64.tar.gz"
      sha256 :no_check
    end

    on_intel do
      url "https://github.com/pmaojo/apicentric/releases/download/v#{version}/apicentric-macos-x86_64.tar.gz"
      sha256 :no_check
    end
  end

  def install
    bin.install "apicentric"
  end

  test do
    system "#{bin}/apicentric", "--version"
  end
end
