class Mockforge < Formula
  desc "API mocking and simulation toolkit"
  homepage "https://github.com/pulse-1/mockforge"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/pulse-1/mockforge/releases/download/v#{version}/mockforge-macos-arm64.tar.gz"
      sha256 :no_check
    end

    on_intel do
      url "https://github.com/pulse-1/mockforge/releases/download/v#{version}/mockforge-macos-x86_64.tar.gz"
      sha256 :no_check
    end
  end

  def install
    bin.install "mockforge"
  end

  test do
    system "#{bin}/mockforge", "--version"
  end
end
