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
- Correctly sized hicolor application icons for desktop launchers

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
- The Intel integrated GPU temperature is intentionally not shown. On the tested
  Fedora install, the Intel GPU uses `i915` but exposes no `i915` hwmon node, no
  DRM `device/hwmon`, and no `lm_sensors` temperature; generic thermal zones are
  not mapped blindly because they are not labeled as iGPU sensors.
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

Create a release archive:

```bash
scripts/package-release.sh 0.1.0-alpha
```

The generated `.tar.gz` and checksum are written to `dist/`.

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

Manual fan control writes through a restricted Polkit helper mode in the same
binary:

```bash
pkexec nitrosense --fan-helper set-manual cpu 50
pkexec nitrosense --fan-helper set-manual gpu 50
pkexec nitrosense --fan-helper set-auto
```

The GUI never reads or stores your password. Polkit shows the system
authentication prompt and the helper only accepts validated fan-control commands.

Launcher icons are installed from `assets/icons/hicolor/`, which contains the
fixed app sizes declared by Fedora's hicolor theme, including `22x22`, `36x36`,
`72x72`, `96x96`, `192x192`, and HiDPI `@2` variants. Do not install the large
source PNG directly into a fixed-size hicolor directory.

Use these controls carefully. Fan control and power profile changes are hardware
touchpoints, so validate setup on your own machine before relying on the app.

## Project Structure

- `src/app/`: app lifecycle, state, actions, screen rendering, and formatting
- `src/ui/`: Nitro-style theme and reusable egui widgets
- `src/hardware/`: `/sys` sensor discovery, platform profiles, and Acer hwmon fan control
- `src/services/`: background polling, thermal notifications, and tray integration
- `src/graph.rs`: rolling graph history and rendering
- `docs/setup.md`: Fedora setup and install notes
- `docs/handoff.md`: architecture and continuation notes
- `docs/official-app-analysis.md`: local official-app extraction guide
- `my_laptop.md`: target laptop facts and local hardware findings
- `nitrosense_info.md`: official NitroSense analysis and reverse-engineering notes

## Status

The default build is warning-clean and tested. Tray support is feature-gated
because it requires GTK/AppIndicator development packages on Linux.
