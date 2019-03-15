#!/run/current-system/sw/bin/bash
sudo nix-shell -p cargo dbus pkgconfig libpulseaudio pulseaudio rustc --pure --run 'cargo build --release'
