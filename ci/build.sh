#!/bin/sh
#
# Stolen from: https://github.com/gtk-rs/examples/blob/master/build.sh#L14

set -x
set -e

if [ "$GTK" = latest -o "$GTK" = "3.18" ]; then
	BUNDLE="gtk-3.18.1-2"
	FEATURES=gtk_3_18
fi

if [ -n "$BUNDLE" ]; then
	cd "$HOME"
	curl -LO "https://github.com/gkoz/gtk-bootstrap/releases/download/$BUNDLE/deps.txz"
	tar xf deps.txz
	export PKG_CONFIG_PATH="$HOME/local/lib/pkgconfig"
fi

cargo test
