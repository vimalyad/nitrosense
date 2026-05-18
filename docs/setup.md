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

The app writes `pwm1`, `pwm2`, `pwm1_enable`, and `pwm2_enable` through a
restricted helper mode in the same binary. The normal GUI calls that helper with
`pkexec`, so KDE/GNOME/Polkit shows the system password prompt and the app never
reads or stores your password.

Manual helper examples:

```bash
pkexec nitrosense --fan-helper authorize
pkexec nitrosense --fan-helper set-manual cpu 50
pkexec nitrosense --fan-helper set-manual gpu 50
pkexec nitrosense --fan-helper set-auto
```

Install `polkit`/`pkexec` if your distribution does not include it:

```bash
sudo dnf install polkit
```

## Desktop Entry

Install the binary, launcher, and icons:

```bash
scripts/install-local.sh release
```

The local installer writes absolute `Exec=` and `Icon=` paths into the installed
desktop entry. This avoids KDE/Plasma showing a generic file icon when theme
icon-name lookup caches are stale.
