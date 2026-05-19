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
- CPU, GPU, NVMe, fan RPM, and power profile display
- One-second background polling with Tokio
- Rolling 35-minute in-memory temperature graph using `egui_plot`, with
  five-minute clock labels, a `0..105 C` plot range, and labels shown only up to
  `100 C`
- Platform power profile switching
- Acer `acer-wmi` hwmon PWM fan control for the AN515-58
- Thermal desktop notifications
- Single-instance process guard for the GUI
- Feature-gated system tray support
- Desktop entry and setup documentation
- Correctly sized hicolor application icons for desktop launchers
- Fixed `920x600` non-resizable window layout tuned for a `176px` sidebar and
  the AN515-58 dashboard screens

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

For normal fan-control testing, prefer the installed launcher path because the
Polkit action is generated for `~/.local/bin/nitrosense`:

```bash
~/.local/bin/nitrosense
```

Install for the current user:

```bash
scripts/install-local.sh release
```

Create a release archive:

```bash
scripts/package-release.sh 0.1.0
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
pkexec nitrosense --fan-helper set-manual-both 50 50
pkexec nitrosense --fan-helper set-auto
```

The local installer also installs a NitroSense Polkit action for the installed
binary path with `auth_admin_keep`. The GUI never reads or stores your password;
Polkit shows the system authentication prompt, keeps the authorization briefly,
and the helper only accepts validated fan-control commands. Slider changes are
debounced and applied through the batched `set-manual-both` helper command. If
you run the app directly from `target/release/nitrosense`, that installed policy
path will not match, so use the application launcher or `~/.local/bin/nitrosense`
for normal fan control testing.

The GUI is single-instance guarded. A second normal launch exits early instead
of creating another polling/fan-control window. Privileged `--fan-helper`
processes are exempt from that GUI lock.

## UI Behavior

- The main window is fixed at `920x600`, non-resizable, with the maximize button
  disabled.
- The left sidebar is fixed at `176px`.
- Monitoring shows thermal stats and a separate cooling section. Battery
  voltage is intentionally not shown or read.
- Temperature graph and fan-control panels are sized for the remaining fixed
  content area. Fan Control content keeps a 10px left inset and 20px right
  breathing room.
- Temperature graph hover labels appear inside the graph only when a real nearby
  sample timestamp exists. If both CPU and GPU graph series are visible, both
  readings are shown; if one series is hidden, only the visible series is shown.
- In-app notification toasts fade after about two seconds.

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
- `src/single_instance.rs`: per-user runtime lock that prevents multiple GUI instances
- `src/graph.rs`: rolling graph history and rendering
- `docs/setup.md`: Fedora setup and install notes
- `docs/handoff.md`: architecture and continuation notes
- `docs/official-app-analysis.md`: local official-app extraction guide
- `my_laptop.md`: target laptop facts and local hardware findings
- `nitrosense_info.md`: official NitroSense analysis and reverse-engineering notes

## Status

The default build is warning-clean and tested. Tray support is feature-gated
because it requires GTK/AppIndicator development packages on Linux.
