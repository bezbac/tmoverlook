class Tmoverlook < Formula
  desc "Exclude development files from Time Machine backups"
  homepage "https://github.com/bezbac/tmoverlook"
  url "https://github.com/bezbac/tmoverlook/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "b241102a283527a32ed88a484f5cd8f7e14b67d82649c82b2944557c5a3bdf94"
  license "BSD 3-Clause"
  head "https://github.com/bezbac/tmoverlook.git", branch: "main"

  livecheck do
    url :stable
    strategy :github_latest
  end

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release"
    bin.install "target/release/tmoverlook"
  end

  test do
    system "#{bin}/tmoverlook", "-V"
  end
end
