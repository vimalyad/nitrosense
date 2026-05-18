# NitroSense Linux

NitroSense Linux is a Rust desktop app for monitoring temperatures and controlling
fan/power behavior on Linux. I created this project for myself to build a GUI app
for controlling my laptop fans on Linux, and also as a learning project for anyone
who wants to study or extend it.

The app is inspired by Acer NitroSense on Windows and is currently targeted at
the Acer Nitro AN515-58 on Fedora KDE Plasma Wayland.

## Features

- Native `egui`/`eframe` desktop UI
- Direct `/sys` sensor reads without shelling out to `sensors`
- Dynamic hwmon discovery for changing `/sys/class/hwmon/hwmon*` indexes
- CPU, GPU, NVMe, fan RPM, battery voltage, and power profile display
- One-second background polling with Tokio
- Rolling in-memory graph using `egui_plot`
- Platform power profile switching
- NBFC-based manual fan control
- Thermal desktop notifications
- Feature-gated system tray support
- Desktop entry and setup documentation

## Build

Default build:

```bash
cargo check
cargo test
cargo build --release
```

Run:

```bash
./target/release/nitrosense
```

## Fedora Setup

Default build dependencies:

```bash
sudo dnf install gcc pkg-config
```

Tray feature dependencies:

```bash
sudo dnf install gtk3-devel libappindicator-gtk3-devel
```

More setup details are in [docs/setup.md](docs/setup.md).

## Hardware Control Notes

Power profile switching writes through:

```bash
sudo -n tee /sys/firmware/acpi/platform_profile
```

Manual fan control uses NBFC:

```bash
nbfc set -f 0 -s <percent>
nbfc set -f 1 -s <percent>
nbfc set --auto
```

Use these controls carefully. Fan control and power profile changes are hardware
touchpoints, so validate setup on your own machine before relying on the app.

## Project Structure

- `src/app.rs`: main UI and app state
- `src/sensors.rs`: `/sys` sensor discovery and reads
- `src/polling.rs`: background sensor polling
- `src/profile.rs`: platform profile reads/writes
- `src/fan_control.rs`: NBFC command wrappers
- `src/graph.rs`: rolling graph history and rendering
- `src/notifications.rs`: thermal alerts and desktop notifications
- `src/tray.rs`: feature-gated tray integration
- `docs/setup.md`: Fedora setup and install notes
- `docs/handoff.md`: architecture and continuation notes

## Status

The default build is warning-clean and tested. Tray support is feature-gated
because it requires GTK/AppIndicator development packages on Linux.
