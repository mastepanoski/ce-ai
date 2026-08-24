class CeAi < Formula
  desc "Compound Engineering AI plugin installer and manager"
  homepage "https://github.com/mastepanoski/ce-ai"
  version "1.21.3"
  license "MIT"

  # Managed by scripts/release-integrity.sh — regenerate with:
  #   TAG_NAME=v1.21.3 GH_REPO=mastepanoski/ce-ai ./scripts/release-integrity.sh
  # Do not edit URLs or checksums by hand.
  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/mastepanoski/ce-ai/releases/download/v1.21.3/ce-ai-x86_64-apple-darwin.tar.gz"
      sha256 "51b86d3b9e681e402bf70cc5717a1f38ceb08e23d92b2d51a2bcac7febcf8cfd"
    else
      url "https://github.com/mastepanoski/ce-ai/releases/download/v1.21.3/ce-ai-aarch64-apple-darwin.tar.gz"
      sha256 "cacad1b98e93cf341542abc6bce51b40888fec0ef889a63d969e0c612e508572"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/mastepanoski/ce-ai/releases/download/v1.21.3/ce-ai-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "9bcb9291accb4595d653969b7e03d85d0fa45806ac42ffa324c2849472504c40"
    else
      url "https://github.com/mastepanoski/ce-ai/releases/download/v1.21.3/ce-ai-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "5e48ea3b33108e4d79af535c6c3eeb2d89bb982cc0a52f54156c38224b8efcea"
    end
  end

  def install
    bin.install "ce-ai"
  end

  test do
    assert_match "ce-ai", shell_output("#{bin}/ce-ai --version")
  end
end
