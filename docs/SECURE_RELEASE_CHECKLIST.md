# MorSteg Secure Release Checklist

## Before building

```bash
cargo check --locked
cargo test --locked
git status
```

The working tree should be clean.

## Version

Update:

```text
Cargo.toml version
PKGBUILD pkgver
flake.nix version
release notes version
```

## Build

```bash
bash scripts/release-local.sh 0.2.0
```

## Verify artifacts

```bash
cd dist
sha256sum -c SHA256SUMS
```

## Sign

The release script tries:

```bash
gpg --armor --detach-sign dist/SHA256SUMS
```

Do not claim "secure release" unless the checksum file is attached.

## Post-quantum-capable wording

Allowed:

```text
Post-quantum-capable through age PQ recipient keys.
```

Avoid:

```text
Quantum-proof
Unbreakable
Fully quantum-safe steghide
```

## GitHub release

Tag:

```bash
git tag -s v0.2.0 -m "MorSteg CLI v0.2.0"
git push origin v0.2.0
```

Then upload all files from:

```text
dist/
```
