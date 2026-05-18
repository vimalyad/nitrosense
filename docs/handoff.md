# NitroSense Handoff

## Current Architecture

NitroSense is a native Rust desktop app using `eframe`/`egui`.

- `src/app.rs`: app state, UI, profile actions, fan actions, graph display, notifications, tray integration glue.
- `src/sensors.rs`: read-only `/sys` hwmon discovery and sensor reads.
- `src/polling.rs`: Tokio background polling task that sends `SensorSnapshot` values through a watch channel.
- `src/profile.rs`: platform profile reads and privileged writes through `sudo -n tee`.
- `src/fan_control.rs`: Acer `acer-wmi` hwmon PWM discovery and write helpers.
- `src/graph.rs`: RAM-only rolling graph history and `egui_plot` rendering.
- `src/notifications.rs`: thermal alert thresholds, cooldowns, and `notify-rust` delivery.
- `src/tray.rs`: feature-gated tray integration with a no-op default backend.
- `docs/setup.md`: Fedora dependencies and local installation/setup commands.

## Important Build Commands

Default build:

```bash
cargo check
cargo test
cargo build --release
```

Tray feature build:

```bash
cargo check --features tray
```

The tray feature requires GTK/AppIndicator development packages. On Fedora,
start with:

```bash
sudo dnf install gtk3-devel libappindicator-gtk3-devel
```

## Known Caveats

- The default build intentionally keeps tray support disabled so the app can
  build without GTK/AppIndicator development packages.
- `cargo check --features tray` was not verified in this environment because
  required `pkg-config` files for GTK stack libraries were missing.
- Runtime tray behavior still needs verification on Fedora KDE Plasma Wayland
  after installing tray dependencies.
- Power profile writes require passwordless sudo for:
  `tee /sys/firmware/acpi/platform_profile`.
- Manual fan control is AN515-58-specific and uses the Acer hwmon PWM files
  exposed by `acer-wmi`. Writes require passwordless sudo for the current
  `pwm1`, `pwm2`, `pwm1_enable`, and `pwm2_enable` paths.

## Branch Flow

Each project phase was implemented on a phase branch:

- `phase-1-foundation`
- `phase-2-sensors`
- `phase-3-ui`
- `phase-4-polling`
- `phase-5-graphs`
- `phase-6-power-profiles`
- `phase-7-fan-control`
- `phase-8-notifications`
- `phase-9-tray`
- `phase-10-packaging`
- `phase-11-hardening`
