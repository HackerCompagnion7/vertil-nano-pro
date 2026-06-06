# Termux package metadata for Vertil Nano Pro
# This file follows Termux package format

TERMUX_PKG_HOMEPAGE=https://github.com/ishmaelvertil/vertil-nano-pro
TERMUX_PKG_DESCRIPTION="Modern terminal code editor — simple like Nano, powerful for 2026"
TERMUX_PKG_LICENSE="MIT"
TERMUX_PKG_MAINTAINER="Ishmael Vertil"
TERMUX_PKG_VERSION=1.0.0
TERMUX_PKG_REVISION=1
TERMUX_PKG_SRCURL=https://github.com/ishmaelvertil/vertil-nano-pro/archive/refs/tags/v${TERMUX_PKG_VERSION}.tar.gz
TERMUX_PKG_SHA256=SKIP_CHECKSUM
TERMUX_PKG_BUILD_IN_SRC=true
TERMUX_PKG_DEPENDS="libgit2"
TERMUX_PKG_BUILD_DEPENDS="cmake"

termux_step_make() {
    cargo build --release --target aarch64-linux-android
}

termux_step_make_install() {
    install -Dm700 target/release/nanopro $TERMUX_PREFIX/bin/nanopro
    install -Dm600 themes/dark.toml $TERMUX_PREFIX/etc/nanopro/themes/dark.toml
    install -Dm600 themes/light.toml $TERMUX_PREFIX/etc/nanopro/themes/light.toml
}
