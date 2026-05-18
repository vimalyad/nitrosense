# NitroSense Setup

## Fedora Build Dependencies

Default build:

```bash
sudo dnf install gcc pkg-config
```

Tray feature build:

```bash
sudo dnf install gtk3-devel libappindicator-gtk3-devel
```

If `cargo check --features tray` reports missing `*.pc` files, install the
corresponding Fedora development package. Common requirements include GLib, GDK,
GDK Pixbuf, Cairo, Pango, and ATK, which are normally pulled in by `gtk3-devel`.

## Power Profile Switching

The app writes platform profiles through:

```bash
sudo -n tee /sys/firmware/acpi/platform_profile
```

Set up passwordless sudo for that exact command:

```bash
echo "vimal2907 ALL=(ALL) NOPASSWD: /usr/bin/tee /sys/firmware/acpi/platform_profile" \
  | sudo tee /etc/sudoers.d/nitrosense
```

## NBFC Fan Control

Manual fan control requires NBFC and its service:

```bash
sudo systemctl enable --now nbfc_service
sudo nbfc config --set "Acer Nitro AN515-58"
```

## Desktop Entry

Install the binary and icon:

```bash
cargo build --release
install -Dm755 target/release/nitrosense ~/.local/bin/nitrosense
install -Dm644 assets/icon.png ~/.local/share/icons/hicolor/256x256/apps/nitrosense.png
install -Dm644 packaging/nitrosense.desktop ~/.local/share/applications/nitrosense.desktop
update-desktop-database ~/.local/share/applications
```
