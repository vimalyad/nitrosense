# NitroSense Linux

NitroSense Linux is a Rust desktop app for monitoring temperatures and controlling
fan/power behavior on Linux. I created this project for myself to build a GUI app
for controlling my laptop fans on Linux, and also as a learning project for anyone
who wants to study or extend it.

The app is inspired by Acer NitroSense on Windows and is currently targeted at
the Acer Nitro AN515-58 on Fedora KDE Plasma Wayland.

This repository also documents the exploration behind the implementation. The
tracked notes summarize what was learned from Acer's official NitroSense package,
the AN515-58 plug-in files, decompiled managed code, and the Linux `acer-wmi`
interfaces available on the target laptop. The official Acer package itself is
not tracked.

## Features

- Native `egui`/`eframe` desktop UI
- Direct `/sys` sensor reads without shelling out to `sensors`
- Dynamic hwmon discovery for changing `/sys/class/hwmon/hwmon*` indexes
- CPU, GPU, NVMe, fan RPM, battery voltage, and power profile display
- One-second background polling with Tokio
- Rolling in-memory graph using `egui_plot`
- Platform power profile switching
- Acer `acer-wmi` hwmon PWM fan control for the AN515-58
- Thermal desktop notifications
- Feature-gated system tray support
- Desktop entry and setup documentation

## Reverse Engineering Notes

The project keeps portable findings in tracked Markdown files so someone reading
the repo can understand why the Linux implementation works the way it does:

- [my_laptop.md](my_laptop.md): exact AN515-58 identity, official plug-in facts,
  Linux kernel capability scan, and current hardware-control decisions.
- [nitrosense_info.md](nitrosense_info.md): detailed official NitroSense package
  analysis, named-pipe/service architecture, Acer WMI commands, fan payloads,
  sensor indices, CoolBoost notes, and Linux implications.
- [docs/official-app-analysis.md](docs/official-app-analysis.md): local-only
  commands for users who want to download Acer's package themselves and reproduce
  the extraction/decompilation workflow.

Important conclusions so far:

- Acer's Windows UI does not directly write raw EC registers; it talks to a
  service, and that service calls Acer WMI methods under `ROOT\WMI`.
- The AN515-58 official plug-in supports CPU and GPU fans, but not a separate
  system fan.
- On Linux, this laptop exposes fan RPM and PWM control through the kernel
  `acer-wmi` hwmon adapter, so this app uses that native interface first.
- Raw EC writes are intentionally avoided unless a safe model-specific path is
  verified later.

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

Manual fan control writes through the Acer hwmon adapter exposed by `acer-wmi`:

```bash
sudo -n tee /sys/class/hwmon/hwmon*/pwm1_enable
sudo -n tee /sys/class/hwmon/hwmon*/pwm1
sudo -n tee /sys/class/hwmon/hwmon*/pwm2_enable
sudo -n tee /sys/class/hwmon/hwmon*/pwm2
```

Use these controls carefully. Fan control and power profile changes are hardware
touchpoints, so validate setup on your own machine before relying on the app.

## Project Structure

- `src/app.rs`: main UI and app state
- `src/sensors.rs`: `/sys` sensor discovery and reads
- `src/polling.rs`: background sensor polling
- `src/profile.rs`: platform profile reads/writes
- `src/fan_control.rs`: Acer hwmon PWM fan-control backend
- `src/graph.rs`: rolling graph history and rendering
- `src/notifications.rs`: thermal alerts and desktop notifications
- `src/tray.rs`: feature-gated tray integration
- `docs/setup.md`: Fedora setup and install notes
- `docs/handoff.md`: architecture and continuation notes
- `docs/official-app-analysis.md`: local official-app extraction guide
- `my_laptop.md`: target laptop facts and local hardware findings
- `nitrosense_info.md`: official NitroSense analysis and reverse-engineering notes

## Status

The default build is warning-clean and tested. Tray support is feature-gated
because it requires GTK/AppIndicator development packages on Linux.
