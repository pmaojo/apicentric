class Mockforge < Formula
  desc "API mocking and simulation toolkit"
  homepage "https://github.com/your-org/mockforge"
  version "0.1.0"
  url "https://github.com/your-org/mockforge/releases/download/v#{version}/mockforge-#{version}-x86_64-apple-darwin.tar.gz"
  sha256 "0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5" # TODO: replace with real sha256

  def install
    bin.install "mockforge"
  end

  test do
    system "#{bin}/mockforge", "--version"
  end
end
