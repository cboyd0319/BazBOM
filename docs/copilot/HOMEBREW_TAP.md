# Homebrew Tap Setup for BazBOM

Goal: Publish BazBOM to a user-owned tap first, then to homebrew-core when stable.

Create a Tap Repository
1) Create a public repo named `homebrew-bazbom` under your GitHub account
2) Add a `README.md` explaining the tap
3) Create `Formula/bazbom.rb` with a formula (template below)

Formula Template (`Formula/bazbom.rb`)
```
class Bazbom < Formula
  desc "Build-time SBOM, SCA, and dependency graph for JVM projects"
  homepage "https://github.com/cboyd0319/BazBOM"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/cboyd0319/BazBOM/releases/download/v0.1.0/bazbom-aarch64-apple-darwin.tar.gz"
      sha256 "<sha256>"
    else
      url "https://github.com/cboyd0319/BazBOM/releases/download/v0.1.0/bazbom-x86_64-apple-darwin.tar.gz"
      sha256 "<sha256>"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/cboyd0319/BazBOM/releases/download/v0.1.0/bazbom-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "<sha256>"
    else
      url "https://github.com/cboyd0319/BazBOM/releases/download/v0.1.0/bazbom-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "<sha256>"
    end
  end

  def install
    bin.install "bazbom"
    (bash_completion/"bazbom").write Utils.safe_popen_read(bin/"bazbom", "--generate-completions", "bash")
    (zsh_completion/"_bazbom").write Utils.safe_popen_read(bin/"bazbom", "--generate-completions", "zsh")
  end

  test do
    assert_match "bazbom", shell_output("#{bin}/bazbom --version")
  end
end
```

Publish a Release
- Attach macOS (x86_64/arm64) and Linux (x86_64/aarch64) tarballs containing the `bazbom` binary
- Include checksums, cosign signature, and provenance in release assets

Use the Tap
```
brew tap cboyd0319/bazbom
brew install bazbom
```

Upstream to homebrew-core
- After several stable releases, open a PR to homebrew-core with the formula
- Ensure bottles are produced via CI for faster installs

