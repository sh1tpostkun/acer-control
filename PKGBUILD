pkgname=acercontrol-git
pkgver=1.0.0
pkgrel=1
pkgdesc="Native Linux Control Center for Acer Nitro laptops"
arch=('x86_64')
url="https://github.com/sh1tpostkun/acer-control"
license=('GPL3')
depends=('qt6-base' 'qt6-declarative' 'qt6-svg')
makedepends=('cargo' 'cmake' 'ninja' 'qt6-tools')
provides=('acercontrol')
conflicts=('acercontrol')
source=("git+https://github.com/sh1tpostkun/acer-control.git")
md5sums=('SKIP')

build() {
  cd "$srcdir/acer-control"
  
  # Build Rust components
  cargo build --release -p acercontrol-daemon -p acercontrol-cli
  
  # Build Qt GUI
  cmake -B build -S . \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_INSTALL_PREFIX=/usr
  cmake --build build
}

package() {
  cd "$srcdir/acer-control"
  
  # Install binaries
  install -Dm755 target/release/acercontrol-daemon "$pkgdir/usr/bin/acercontrol-daemon"
  install -Dm755 target/release/acercontrol-cli "$pkgdir/usr/bin/acercontrol"
  install -Dm755 build/gui/acercontrol-gui "$pkgdir/usr/bin/acercontrol-gui"
  install -Dm755 acercontrol-toggle.sh "$pkgdir/usr/bin/acercontrol-toggle"
  
  # Install systemd service
  install -Dm644 systemd/acercontrol.service "$pkgdir/usr/lib/systemd/system/acercontrol.service"
  
  # Install desktop entry
  install -Dm644 gui/res/app_icon.png "$pkgdir/usr/share/pixmaps/acercontrol.png"
  install -Dm644 acercontrol.desktop "$pkgdir/usr/share/applications/acercontrol.desktop"
}
