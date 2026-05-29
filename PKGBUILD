# Maintainer: Moribund Murdoch <moribundmurdoch@gmail.com>

pkgname=mor-steg
pkgver=0.2.0
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
