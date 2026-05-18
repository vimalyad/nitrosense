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

## Acer WMI Fan Control

On the target AN515-58, Linux exposes Acer fan control through the `acer-wmi`
hwmon adapter. Verify it is present:

```bash
cat /sys/class/hwmon/hwmon*/name
ls /sys/class/hwmon/hwmon*/pwm*
```

The app writes `pwm1`, `pwm2`, `pwm1_enable`, and `pwm2_enable` through
`sudo -n tee`. Because the exact `hwmonN` index can change across boots, create
a sudoers rule after replacing `hwmon5` with the current Acer hwmon directory:

```bash
echo "vimal2907 ALL=(ALL) NOPASSWD: /usr/bin/tee /sys/class/hwmon/hwmon5/pwm1, /usr/bin/tee /sys/class/hwmon/hwmon5/pwm1_enable, /usr/bin/tee /sys/class/hwmon/hwmon5/pwm2, /usr/bin/tee /sys/class/hwmon/hwmon5/pwm2_enable" \
  | sudo tee /etc/sudoers.d/nitrosense-fans
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
