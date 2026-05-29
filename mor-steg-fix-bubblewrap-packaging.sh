#!/usr/bin/env bash
set -euo pipefail

if [[ ! -f "Cargo.toml" || ! -d "src" ]]; then
  echo "Run this from the root of your mor-steg project."
  echo "Example:"
  echo "  cd ~/Projects/mor-steg"
  echo "  bash ~/Downloads/mor-steg-fix-bubblewrap-packaging.sh"
  exit 1
fi

echo "[Mor-SteG] Organizing Bubblewrap packaging files..."

mkdir -p packaging/arch

# Move loose Bubblewrap files from the project root into packaging/arch if they exist.
if [[ -f "mor-steg-sandboxed" ]]; then
  mv -f "mor-steg-sandboxed" "packaging/arch/mor-steg-sandboxed"
fi

if [[ -f "mor-steg-sandboxed.desktop" ]]; then
  mv -f "mor-steg-sandboxed.desktop" "packaging/arch/mor-steg-sandboxed.desktop"
fi

# Remove the temporary snippet file if it is still hanging around.
rm -f PKGBUILD_BUBBLEWRAP_SNIPPET.txt

# If the sandboxed launcher is missing, create it.
if [[ ! -f "packaging/arch/mor-steg-sandboxed" ]]; then
  cat > "packaging/arch/mor-steg-sandboxed" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

APP="/usr/bin/mor-steg"

if ! command -v bwrap >/dev/null 2>&1; then
  echo "Bubblewrap was not found."
  echo
  echo "Install it on Arch with:"
  echo "  sudo pacman -S bubblewrap"
  echo
  echo "Then run this again."
  exit 1
fi

if [[ ! -x "$APP" ]]; then
  echo "Mor-SteG was not found at:"
  echo "  $APP"
  exit 1
fi

CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/mor-steg"
KEY_DIR="$CONFIG_DIR/keys"
OUTPUT_DIR="$HOME/Mor-SteG-Output"
WORK_DIR="$HOME/Mor-SteG-Work"

mkdir -p "$KEY_DIR" "$OUTPUT_DIR" "$WORK_DIR"

echo "Mor-SteG Sandboxed Mode"
echo
echo "Inside the sandbox, Mor-SteG can use:"
echo "  Work folder:   $WORK_DIR"
echo "  Output folder: $OUTPUT_DIR"
echo "  Key folder:    $KEY_DIR"
echo
echo "Tip:"
echo "  Put cover files and secret files in:"
echo "    $WORK_DIR"
echo
echo "  Save new files in:"
echo "    $OUTPUT_DIR"
echo
echo "Starting sandbox..."
echo

exec bwrap \
  --die-with-parent \
  --new-session \
  --unshare-all \
  --ro-bind /usr /usr \
  --ro-bind /etc /etc \
  --ro-bind /bin /bin \
  --ro-bind-try /sbin /sbin \
  --ro-bind-try /lib /lib \
  --ro-bind-try /lib64 /lib64 \
  --dev /dev \
  --proc /proc \
  --tmpfs /tmp \
  --bind "$CONFIG_DIR" "$CONFIG_DIR" \
  --bind "$OUTPUT_DIR" "$OUTPUT_DIR" \
  --bind "$WORK_DIR" "$WORK_DIR" \
  --setenv HOME "$HOME" \
  --setenv XDG_CONFIG_HOME "${XDG_CONFIG_HOME:-$HOME/.config}" \
  --setenv PATH "/usr/bin:/bin" \
  --chdir "$WORK_DIR" \
  "$APP"
EOF
fi

chmod +x packaging/arch/mor-steg-sandboxed

# Make/overwrite the normal desktop file.
cat > "packaging/arch/mor-steg.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=Mor-SteG
Comment=Noob-friendly steganography helper
Exec=mor-steg
Icon=mor-steg
Terminal=true
Categories=Utility;Security;Cryptography;
StartupNotify=false
EOF

# Make/overwrite the sandboxed desktop file.
cat > "packaging/arch/mor-steg-sandboxed.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=Mor-SteG Sandboxed
Comment=Run Mor-SteG in a Bubblewrap sandbox
Exec=mor-steg-sandboxed
Icon=mor-steg
Terminal=true
Categories=Utility;Security;Cryptography;
StartupNotify=false
EOF

# Overwrite PKGBUILD with the updated package recipe.
cat > "PKGBUILD" <<'EOF'
# Maintainer: Moribund Murdoch <moribundmurdoch@gmail.com>

pkgname=mor-steg
pkgver=0.1.0
pkgrel=1
pkgdesc="A noob-friendly Rust front-end for age and steghide"
arch=('x86_64')
url="https://github.com/MoribundMurdoch/mor-steg"
license=('Unlicense')
makedepends=('cargo')
depends=('age')
optdepends=(
  'steghide: required backend for hiding and extracting files; on Arch, install from the AUR'
  'bubblewrap: optional sandboxed launcher'
)
source=()
sha256sums=()

build() {
  cd "$startdir"

  # Security note:
  # --locked makes Cargo use Cargo.lock exactly.
  # If this errors because Cargo.lock does not exist yet, run:
  #   cargo generate-lockfile
  cargo build --release --locked
}

package() {
  cd "$startdir"

  install -Dm755 "target/release/mor-steg" \
    "$pkgdir/usr/bin/mor-steg"

  install -Dm755 "packaging/arch/mor-steg-sandboxed" \
    "$pkgdir/usr/bin/mor-steg-sandboxed"

  install -Dm644 "packaging/arch/mor-steg.desktop" \
    "$pkgdir/usr/share/applications/mor-steg.desktop"

  install -Dm644 "packaging/arch/mor-steg-sandboxed.desktop" \
    "$pkgdir/usr/share/applications/mor-steg-sandboxed.desktop"

  install -Dm644 "assets/mor-steg.png" \
    "$pkgdir/usr/share/icons/hicolor/256x256/apps/mor-steg.png"

  install -Dm644 "LICENSE" \
    "$pkgdir/usr/share/licenses/$pkgname/LICENSE"

  install -Dm644 "README.md" \
    "$pkgdir/usr/share/doc/$pkgname/README.md"
}
EOF

echo
echo "[Mor-SteG] Done."
echo
echo "Expected layout now:"
echo "  packaging/arch/mor-steg.desktop"
echo "  packaging/arch/mor-steg-sandboxed"
echo "  packaging/arch/mor-steg-sandboxed.desktop"
echo "  PKGBUILD"
echo
echo "Next:"
echo "  cargo generate-lockfile"
echo "  rm -rf pkg"
echo "  rm -f mor-steg-*.pkg.tar.zst mor-steg-debug-*.pkg.tar.zst"
echo "  makepkg -f"
echo "  sudo pacman -U ./mor-steg-0.1.0-1-x86_64.pkg.tar.zst"
