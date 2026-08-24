class CeAi < Formula
  desc "Compound Engineering AI plugin installer and manager"
  homepage "https://github.com/mastepanoski/ce-ai"
  version "1.21.2"
  license "MIT"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/mastepanoski/ce-ai/releases/download/v1.21.2/ce-ai-x86_64-apple-darwin.tar.gz"
      sha256 "663f4f1db137ab0e0cf8d09ada449edbc15192118ebcc3fe726c0f1ae9af6303"
    else
      url "https://github.com/mastepanoski/ce-ai/releases/download/v1.21.2/ce-ai-aarch64-apple-darwin.tar.gz"
      sha256 "c8b8c48427c7f58d8049cf49babb3567dee7e5825ddbfe71e873ac60d0553dcd"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/mastepanoski/ce-ai/releases/download/v1.21.2/ce-ai-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "e0fe33bad21027fb41fd14702bbb3c3c55a4681a587d1287994fc73074fd15a3"
    else
      url "https://github.com/mastepanoski/ce-ai/releases/download/v1.21.2/ce-ai-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "868ce5582c687f4be39738b328a0221715a95f30a942b54d196a01f1df22f946"
    end
  end

  def install
    bin.install "ce-ai"
  end

  test do
    assert_match "ce-ai", shell_output("#{bin}/ce-ai --version")
  end
end
