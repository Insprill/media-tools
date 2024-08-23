# Maintainer: Pierce Thompson <pierce at insprill dot net>

pkgname=media-tools-git
pkgver=v0.1.0
pkgrel=1
pkgdesc=""
arch=("x86_64")
url="https://github.com/Insprill/media-tools"
license=('Apache-2.0')
makedepends=('git' 'cargo')
provides=("${pkgname%-git}")
conflicts=("${pkgname%-git}")
source=('git+https://github.com/Insprill/media-tools.git')
sha256sums=('SKIP')

# We don't have any releases yet
# pkgver() {
#     cd "${pkgname%-git}"
#     git describe --long --tags --abbrev=6 | sed 's/\([^-]*-g\)/r\1/;s/-/./g'
# }

prepare() {
    cd "${pkgname%-git}"
    cargo fetch
}

build() {
    cd "${pkgname%-git}"
    cargo build --release
}

package() {
    cd "${pkgname%-git}"
    install -Dm755 "target/release/${pkgname%-git}" -t "$pkgdir/usr/bin/"
    install -Dm644 "LICENSE" -t "$pkgdir/usr/share/licenses/${pkgname%-git}"
}
