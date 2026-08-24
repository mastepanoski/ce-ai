class CeAi < Formula
  desc "Compound Engineering AI plugin installer and manager"
  homepage "https://github.com/mastepanoski/ce-ai"
  url "https://github.com/mastepanoski/ce-ai/archive/refs/tags/v1.18.0.tar.gz"
  version "1.18.0"
  license "MIT"

  if OS.mac? && Hardware::CPU.intel?
    url "https://github.com/mastepanoski/ce-ai/releases/download/v1.10.0/ce-ai-x86_64-apple-darwin.tar.gz"
  elsif OS.mac? && Hardware::CPU.arm?
    url "https://github.com/mastepanoski/ce-ai/releases/download/v1.10.0/ce-ai-aarch64-apple-darwin.tar.gz"
  elsif OS.linux? && Hardware::CPU.intel?
    url "https://github.com/mastepanoski/ce-ai/releases/download/v1.10.0/ce-ai-x86_64-unknown-linux-gnu.tar.gz"
  elsif OS.linux? && Hardware::CPU.arm?
    url "https://github.com/mastepanoski/ce-ai/releases/download/v1.10.0/ce-ai-aarch64-unknown-linux-gnu.tar.gz"
  end

  def install
    bin.install "ce-ai"
  end

  test do
    assert_match "ce-ai", shell_output("#{bin}/ce-ai --version")
  end
end
