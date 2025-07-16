# Maintainer: Your Name <your@email.com>
pkgname=drlogseeker
pkgver=0.1.0
pkgrel=1
pkgdesc="An application for analyzing DR values in log files"
arch=('x86_64') # Adjust if you support other architectures
url="https://github.com/loxoron218/drlogseeker"
license=('GPL3')

# System dependencies for building and running GTK-rs applications
depends=(
    'gtk4'
    'glib2'
    'libadwaita'
    'cairo'
    'pango'
    'gdk-pixbuf2'
    'gobject-introspection'
)
makedepends=(
    'cargo'
    'rust'
    'git'
    'pkgconf' # Provides pkg-config
)

# For git source, we need to get the commit hash for reproducible builds
# This assumes your repository is at the root of the source directory after cloning
_commit=$(git ls-remote "${url}.git" HEAD | cut -f1)
source=("git+${url}.git#commit=${_commit}")
sha256sums=('SKIP') # Use SKIP for git sources as content changes with each commit

build() {
  cd "${srcdir}/${pkgname}"
  cargo build --release --locked
}

package() {
  # Create the destination directory for the binary
  install -D -m755 "${srcdir}/${pkgname}/target/release/${pkgname}" "${pkgdir}/usr/bin/${pkgname}"

  # Install the license file
  install -D -m644 "${srcdir}/${pkgname}/LICENSE" "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"

  # --- IMPORTANT: Desktop Entry and Icon ---
  # If drlogseeker is a GUI application, you should create a .desktop file
  # and an icon for proper integration with desktop environments.
  install -D -m644 "${srcdir}/${pkgname}/${pkgname}.desktop" "${pkgdir}/usr/share/applications/${pkgname}.desktop"
  install -D -m644 "${srcdir}/${pkgname}/icon.svg" "${pkgdir}/usr/share/icons/hicolor/scalable/apps/${pkgname}.svg"
}
