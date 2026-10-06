# Maintainer: Voltsparx <voltsparx@gmail.com>
pkgname=nailsnake
pkgver=$(sed -n 's/^version = "\([^"]*\)"$/\1/p' "$startdir/Cargo.toml" | head -n 1)
pkgrel=1
pkgdesc="Cross-platform terminal Snake game written in Rust"
arch=('x86_64' 'aarch64')
url="https://github.com/voltsparx/NailSnake"
license=('MIT')
depends=('gcc-libs')
makedepends=('cargo')
source=()
sha256sums=()

build() {
    cd "$startdir"
    cargo build --release --locked
}

check() {
    cd "$startdir"
    cargo test --locked
}

package() {
    cd "$startdir"
    install -Dm755 "target/release/nailsnake" "$pkgdir/usr/bin/nailsnake"
    install -Dm644 "man/nailsnake.1" "$pkgdir/usr/share/man/man1/nailsnake.1"
    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
}
