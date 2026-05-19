# Contributing to NitroSense Linux

NitroSense Linux is currently custom-targeted at the Acer Nitro AN515-58. Please
keep changes conservative, easy to review, and scoped to the subsystem they
touch.

## Local Setup

Install the default Fedora build dependencies:

```bash
sudo dnf install gcc pkg-config
```

Optional tray builds require GTK/AppIndicator development packages:

```bash
sudo dnf install gtk3-devel libappindicator-gtk3-devel
```

Build and test:

```bash
cargo fmt -- --check
cargo check
cargo test
```

Install locally for GUI and fan-control testing:

```bash
scripts/install-local.sh release
```

Use the installed launcher or `~/.local/bin/nitrosense` when testing fan
control. The Polkit policy is generated for that installed path.

## Code Organization

- `src/app/mod.rs`: application lifecycle, state, polling integration, and
  hardware action handlers.
- `src/app/views/`: small view modules. Keep tab rendering and UI-only helpers
  near the tab they belong to.
- `src/ui/`: shared theme and reusable widgets.
- `src/graph/`: graph history, rendering, axis formatting, hover labels, and
  graph tests.
- `src/hardware/`: hardware discovery and writes. Treat this area as
  high-risk.
- `src/services/`: background polling, notifications, and tray integration.

## Design Rules

- Preserve the fixed `920x600` non-resizable window unless the layout is
  intentionally redesigned.
- Keep the sidebar at `176px`.
- Avoid adding generic laptop support without verified hardware behavior.
- Do not reintroduce unlabeled Intel GPU temperatures or battery display unless
  the data source is reliable and intentionally supported.
- Keep UI panels compact and readable inside the fixed window.

## Hardware Safety

- Do not add raw EC writes unless the register map is model-specific, reviewed,
  and documented.
- Prefer Linux kernel interfaces such as `acer-wmi` hwmon over direct firmware
  access.
- Fan writes must stay validated and routed through the restricted helper path.
- If a fan update fails, the app should continue attempting to restore automatic
  fan control.

## Pull Request Checklist

- Explain what changed and why.
- Update documentation for user-visible behavior, setup steps, or architecture
  changes.
- Run:

```bash
cargo fmt -- --check
cargo check
cargo test
```

- For UI changes, check the fixed `920x600` window manually.
- For fan-control changes, mention the exact hardware and Linux hwmon paths used
  during testing.

## Release Checklist

Create a release archive:

```bash
scripts/package-release.sh 0.1.0
```

Upload both generated files from `dist/`:

- `nitrosense-<version>-<target>.tar.gz`
- `nitrosense-<version>-<target>.tar.gz.sha256`
