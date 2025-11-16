# GPG Signing for BazBOM Releases

This document describes how to set up GPG signing for BazBOM releases to enhance supply chain security.

## Overview

GPG signing provides cryptographic verification of release binaries, ensuring they haven't been tampered with and come from a trusted source.

## For Maintainers: Signing Releases

### 1. Generate GPG Key (if you don't have one)

```bash
gpg --full-generate-key
```

Choose:
- Key type: RSA and RSA
- Key size: 4096 bits
- Expiration: 2 years (recommended)
- Name and email: Use your GitHub commit email

### 2. Export Public Key

```bash
# Get your key ID
gpg --list-keys

# Export public key
gpg --armor --export YOUR_KEY_ID > BAZBOM_PUBLIC_KEY.asc

# Publish to key server
gpg --send-keys YOUR_KEY_ID
```

### 3. Sign Release Binaries

Add this to `.github/workflows/release.yml`:

```yaml
- name: Sign release binaries
  env:
    GPG_PRIVATE_KEY: ${{ secrets.GPG_PRIVATE_KEY }}
    GPG_PASSPHRASE: ${{ secrets.GPG_PASSPHRASE }}
  run: |
    echo "$GPG_PRIVATE_KEY" | gpg --import
    for file in dist/*; do
      echo "$GPG_PASSPHRASE" | gpg --batch --yes --passphrase-fd 0 \
        --detach-sign --armor "$file"
    done

- name: Upload signatures
  uses: actions/upload-artifact@v4
  with:
    name: signatures
    path: dist/*.asc
```

### 4. Generate Checksums

```bash
cd dist/
for file in *.tar.gz; do
  sha256sum "$file" > "$file.sha256"
done
```

## For Users: Verifying Signatures

### 1. Import BazBOM Public Key

```bash
# Download public key
curl -sSfL https://github.com/cboyd0319/BazBOM/raw/main/BAZBOM_PUBLIC_KEY.asc \
  | gpg --import

# Or from key server
gpg --recv-keys BAZBOM_KEY_ID
```

### 2. Verify Signature

```bash
# Download release, signature, and checksum
wget https://github.com/cboyd0319/BazBOM/releases/download/v6.5.0/bazbom-linux-x86_64.tar.gz
wget https://github.com/cboyd0319/BazBOM/releases/download/v6.5.0/bazbom-linux-x86_64.tar.gz.asc
wget https://github.com/cboyd0319/BazBOM/releases/download/v6.5.0/bazbom-linux-x86_64.tar.gz.sha256

# Verify GPG signature
gpg --verify bazbom-linux-x86_64.tar.gz.asc bazbom-linux-x86_64.tar.gz

# Verify checksum
sha256sum -c bazbom-linux-x86_64.tar.gz.sha256
```

### 3. Use Secure Installation Script

The secure installation script (`install-secure.sh`) automatically verifies both GPG signatures and checksums:

```bash
curl -sSfL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install-secure.sh | bash
```

## Automation

### GitHub Actions Setup

1. Generate GPG key locally
2. Export private key: `gpg --armor --export-secret-keys YOUR_KEY_ID`
3. Add to GitHub Secrets:
   - `GPG_PRIVATE_KEY`: The exported private key
   - `GPG_PASSPHRASE`: The key passphrase
4. Update release workflow to include signing steps

### Key Rotation

Rotate GPG keys every 2 years:

1. Generate new key
2. Sign old key with new key (web of trust)
3. Update GitHub secrets
4. Announce key rotation to users
5. Keep old key for 1 year for verification of old releases

## Trust Model

BazBOM follows the "Web of Trust" model:

1. **Maintainer Keys**: Each maintainer has their own GPG key
2. **Release Key**: A dedicated release key signed by all maintainers
3. **Key Fingerprint**: Published in repository README and security policy

## References

- [GPG Manual](https://www.gnupg.org/documentation/manuals/gnupg/)
- [GitHub GPG Signing](https://docs.github.com/en/authentication/managing-commit-signature-verification)
- [SLSA Provenance](https://slsa.dev/provenance/)
