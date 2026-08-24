class CeAi < Formula
  desc "Compound Engineering AI plugin installer and manager"
  homepage "https://github.com/mastepanoski/ce-ai"
  version "1.21.4"
  license "MIT"

  # Managed by scripts/release-integrity.sh — regenerate with:
  #   TAG_NAME=v1.21.4 GH_REPO=mastepanoski/ce-ai ./scripts/release-integrity.sh
  # Do not edit URLs or checksums by hand.
  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/mastepanoski/ce-ai/releases/download/v1.21.4/ce-ai-x86_64-apple-darwin.tar.gz"
      sha256 "54c68bab195ecde9bb59ddf937c81846dc989c1e2208c0d9c558bc5d7a45d5f1"
    else
      url "https://github.com/mastepanoski/ce-ai/releases/download/v1.21.4/ce-ai-aarch64-apple-darwin.tar.gz"
      sha256 "2fa09ec42fb9aa34c0ed9586c5bc94f71972f221893c811fb2349eabef02941c"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/mastepanoski/ce-ai/releases/download/v1.21.4/ce-ai-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "218d338bc7360eb94c5a81e958764d4c0720dc7a41e2c194b378dfe40a7f5368"
    else
      url "https://github.com/mastepanoski/ce-ai/releases/download/v1.21.4/ce-ai-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "d12e0f2db36ba506ad5b8b93fa32ebf5dd373baf298c095ed600f9c9141511cf"
    end
  end

  def install
    bin.install "ce-ai"
  end

  test do
    assert_match "ce-ai", shell_output("#{bin}/ce-ai --version")
  end
end
