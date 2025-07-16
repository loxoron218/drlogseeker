# Maintainer: Your Name <your@email.com>
pkgname=drlogseeker
pkgver=$(git ls-remote "${url}.git" HEAD | cut -f1) # Get the latest commit hash
pkgver="${pkgver:0:7}-g${pkgver}"  # Shorten to first 7 chars and add a "g" prefix
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
source=("git+${url}.git#commit=${pkgver}")
sha256sums=('SKIP') # Use SKIP for git sources as content changes with each commit

# Prepare the build environment
prepare() {

  # Ensure we're working from the latest commit
  cd "${srcdir}/${pkgname}"
  git checkout "${pkgver}" # Checkout the latest commit
}

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
  install -D -m644 "${srcdir}/${pkgname}/${pkgname}.desktop" "${pkgdir}/usr/share/applications/${pkgname}.desktop"
  install -D -m644 "${srcdir}/${pkgname}/icon.svg" "${pkgdir}/usr/share/icons/hicolor/scalable/apps/${pkgname}.svg"
}
