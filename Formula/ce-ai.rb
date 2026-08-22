class CeAi < Formula
  desc "CLI for managing the compound-engineering plugin across AI harnesses"
  homepage "https://github.com/mastepanoski/ce-ai"
  version "1.0.8"
  license "MIT"

  if OS.mac? && Hardware::CPU.intel?
    url "https://github.com/mastepanoski/ce-ai/releases/download/v1.0.8/ce-ai-x86_64-apple-darwin.tar.gz"
    sha256 "5316fffb1938b45e9c0658d8754c99146ab76e805faa27347d680dfb58253a37"
  elsif OS.mac? && Hardware::CPU.arm?
    url "https://github.com/mastepanoski/ce-ai/releases/download/v1.0.8/ce-ai-aarch64-apple-darwin.tar.gz"
    sha256 "8016f761aa5e23dd23d86ed92528899d36963673621eed04cc8cd661dd0e37fd"
  elsif OS.linux? && Hardware::CPU.intel?
    url "https://github.com/mastepanoski/ce-ai/releases/download/v1.0.8/ce-ai-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "6295bc42d277bb36fa47af1e7c24dbbf1657d1e358ba6d8214481dd7cc987869"
  elsif OS.linux? && Hardware::CPU.arm?
    url "https://github.com/mastepanoski/ce-ai/releases/download/v1.0.8/ce-ai-aarch64-unknown-linux-gnu.tar.gz"
    sha256 "a6939c69bd5f209f6d2000d9135b27894346377e080ecf33506d9945b3918f5f"
  end

  def install
    bin.install "ce-ai"
  end

  test do
    assert_match "ce-ai", shell_output("#{bin}/ce-ai --version")
  end
end
