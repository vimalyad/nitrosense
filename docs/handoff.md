# NitroSense Handoff

## Current Architecture

NitroSense is a native Rust desktop app using `eframe`/`egui`.

- `src/app/mod.rs`: app startup, state, lifecycle, and hardware action handlers.
- `src/app/views.rs`: header, navigation, status, overview, graph, and fan-control screen rendering.
- `src/app/formatting.rs`: pure display-formatting helpers and tests.
- `src/ui/theme.rs`: Nitro-style colors, egui visuals, and reusable panel frame.
- `src/ui/widgets.rs`: reusable egui widgets such as navigation buttons, metrics, fan panels, and sliders.
- `src/hardware/sensors.rs`: read-only `/sys` hwmon discovery and sensor reads.
- `src/hardware/profile.rs`: platform profile reads and privileged writes through `sudo -n tee`.
- `src/hardware/fan_control.rs`: Acer `acer-wmi` hwmon PWM discovery and restricted Polkit helper writes.
- `src/services/polling.rs`: Tokio background polling task that sends `SensorSnapshot` values through a watch channel.
- `src/services/notifications.rs`: thermal alert thresholds, cooldowns, and `notify-rust` delivery.
- `src/services/tray.rs`: feature-gated tray integration with a no-op default backend.
- `src/graph.rs`: RAM-only rolling graph history and `egui_plot` rendering.
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
  exposed by `acer-wmi`. The GUI calls the same binary through
  `pkexec --fan-helper ...`, so Polkit handles authentication and the helper
  performs validated writes.

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
