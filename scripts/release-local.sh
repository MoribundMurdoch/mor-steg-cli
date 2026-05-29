#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-0.2.0}"
ROOT="$(pwd)"
DIST="$ROOT/dist"

echo "[MorSteg] Release build v$VERSION"
echo

if [[ ! -f Cargo.toml || ! -d src ]]; then
  echo "Run this from the root of the mor-steg-cli repo."
  exit 1
fi

mkdir -p "$DIST"
rm -f "$DIST"/*

echo "[1/8] Rust checks"
cargo check --locked

echo "[2/8] Release binary"
cargo build --release --locked

echo "[3/8] Portable tarball"
mkdir -p "$DIST/mor-steg-$VERSION-x86_64-unknown-linux-gnu"
cp target/release/mor-steg "$DIST/mor-steg-$VERSION-x86_64-unknown-linux-gnu/"
cp README.md LICENSE "$DIST/mor-steg-$VERSION-x86_64-unknown-linux-gnu/"
if [[ -f packaging/arch/mor-steg-sandboxed ]]; then
  cp packaging/arch/mor-steg-sandboxed "$DIST/mor-steg-$VERSION-x86_64-unknown-linux-gnu/"
fi
tar -C "$DIST" -I 'zstd -19' -cf "$DIST/mor-steg-$VERSION-x86_64-unknown-linux-gnu.tar.zst" "mor-steg-$VERSION-x86_64-unknown-linux-gnu"
rm -rf "$DIST/mor-steg-$VERSION-x86_64-unknown-linux-gnu"

echo "[4/8] Arch package"
if command -v makepkg >/dev/null 2>&1; then
  rm -rf pkg
  rm -f mor-steg-*.pkg.tar.zst mor-steg-debug-*.pkg.tar.zst
  makepkg -f
  cp mor-steg-"$VERSION"-*.pkg.tar.zst "$DIST/" 2>/dev/null || cp mor-steg-*.pkg.tar.zst "$DIST/" || true
else
  echo "makepkg not found; skipping Arch package."
fi

echo "[5/8] DEB package"
if command -v cargo-deb >/dev/null 2>&1; then
  cargo deb --no-build
  cp target/debian/*.deb "$DIST/" || true
else
  echo "cargo-deb not found; skipping .deb."
  echo "Install with: cargo install cargo-deb"
fi

echo "[6/8] RPM package"
if command -v cargo-generate-rpm >/dev/null 2>&1; then
  cargo generate-rpm
  cp target/generate-rpm/*.rpm "$DIST/" || true
else
  echo "cargo-generate-rpm not found; skipping .rpm."
  echo "Install with: cargo install cargo-generate-rpm"
fi

echo "[7/8] Nix build"
if command -v nix >/dev/null 2>&1 && [[ -f flake.nix ]]; then
  nix build .# --print-build-logs
  if [[ -e result ]]; then
    tar -C result -I 'zstd -19' -cf "$DIST/mor-steg-$VERSION-nix-result.tar.zst" .
    rm -f result
  fi
else
  echo "nix or flake.nix not found; skipping Nix build."
fi

echo "[8/8] Checksums and optional signature"
(
  cd "$DIST"
  sha256sum * > SHA256SUMS
)

if command -v gpg >/dev/null 2>&1; then
  gpg --armor --detach-sign "$DIST/SHA256SUMS" || {
    echo "gpg signing skipped or failed."
  }
else
  echo "gpg not found; SHA256SUMS created but not signed."
fi

echo
echo "[MorSteg] Release artifacts:"
ls -lah "$DIST"
echo
echo "Upload everything in dist/ to the GitHub release."
