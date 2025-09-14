class Mockforge < Formula
  desc "API mocking and simulation toolkit"
  homepage "https://github.com/your-org/mockforge"
  version "0.1.0"
  url "https://github.com/your-org/mockforge/releases/download/v#{version}/mockforge-#{version}-x86_64-apple-darwin.tar.gz"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000" # TODO: replace with real sha256

  def install
    bin.install "mockforge"
  end

  test do
    system "#{bin}/mockforge", "--version"
  end
end
