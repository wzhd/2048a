# Maintainer: Zihao Wang <2048a@wzhd.org>

pkgname=2048a
pkgver=0.2.0
pkgrel=1
pkgdesc="2048 game with animation in console"
arch=('i686' 'x86_64')
url="https://github.com/wzhd/2048a"
license=('GPL')
depends=()
makedepends=('cargo')
provides=()
conflicts=()

build() {
  cargo build --release
}

package() {
  install -Dm755 "$startdir/target/release/2048a" "$pkgdir/usr/bin/2048a"
}

# vim:set ts=2 sw=2 et:
