# NitroSense Linux Implementation Plan

This file is the durable project roadmap. Update each checkbox as work lands.
Each phase should be developed on its own branch. Each completed subtask should
be committed with a clear human commit message. After a phase is completed,
pause for review before pushing the branch for GitHub merge.

## Branching and Commit Workflow

- Phase branches use `phase-N-short-name`, for example `phase-1-foundation`.
- Commit after each completed subtask.
- Keep commits focused and explain the user-visible or architectural reason.
- Do not push a phase branch until the user has reviewed it.
- Keep `CHANGES.md` updated locally for every change; it is intentionally ignored.

## Phase 0: Project Planning and Tracking

Branch: `phase-0-project-planning`

- [x] Add a local-only change log.
- [x] Ignore `CHANGES.md` in Git.
- [x] Create this implementation roadmap.
- [ ] Commit the planning and tracking setup.
- [ ] Ask for review before pushing.

## Phase 1: Rust Project Foundation

Branch: `phase-1-foundation`

- [ ] Rename package to `nitrosense`.
- [ ] Align Rust edition and core dependency versions with the project spec.
- [ ] Create the source module skeleton:
  - `app.rs`
  - `sensors.rs`
  - `profile.rs`
  - `fan_control.rs`
  - `graph.rs`
  - `tray.rs`
  - `notifications.rs`
  - `config.rs`
- [ ] Replace the default `main.rs` with an application entry point.
- [ ] Add minimal compile-safe stubs for each module.
- [ ] Run `cargo check`.
- [ ] Commit each completed subtask.
- [ ] Ask for review before pushing.

## Phase 2: Read-Only Sensor Core

Branch: `phase-2-sensors`

- [ ] Implement hwmon discovery by reading `/sys/class/hwmon/hwmon*/name`.
- [ ] Define `SensorData` and per-sensor optional values.
- [ ] Implement direct `/sys` reads for CPU, GPU, fan, NVMe, battery, and profile data.
- [ ] Make missing sensors non-fatal and visible to the caller.
- [ ] Add unit tests for parsing and path resolution logic where practical.
- [ ] Run `cargo check` and relevant tests.
- [ ] Commit each completed subtask.
- [ ] Ask for review before pushing.

## Phase 3: Minimal Desktop UI

Branch: `phase-3-ui`

- [ ] Wire `eframe` and `egui` startup.
- [ ] Build the main application state.
- [ ] Add the top header and power profile display area.
- [ ] Add stats cards for CPU, GPU, fans, NVMe, battery, and active profile.
- [ ] Add tabs for Overview, Graph, and Fan Control.
- [ ] Show graceful unavailable states for missing sensors.
- [ ] Run `cargo check`.
- [ ] Commit each completed subtask.
- [ ] Ask for review before pushing.

## Phase 4: Async Polling Architecture

Branch: `phase-4-polling`

- [ ] Add Tokio runtime integration.
- [ ] Create a one-second sensor polling task.
- [ ] Send latest sensor data to the UI without blocking rendering.
- [ ] Ensure UI code never performs direct blocking sensor I/O.
- [ ] Add basic error reporting for sensor poll failures.
- [ ] Run `cargo check`.
- [ ] Commit each completed subtask.
- [ ] Ask for review before pushing.

## Phase 5: Live Graphs

Branch: `phase-5-graphs`

- [ ] Implement fixed-size ring buffers for the rolling 30-minute window.
- [ ] Add graph series for CPU temp, GPU temp, CPU fan RPM, and GPU fan RPM.
- [ ] Add series visibility toggles.
- [ ] Add `egui_plot` rendering.
- [ ] Keep graph data in RAM only.
- [ ] Run `cargo check`.
- [ ] Commit each completed subtask.
- [ ] Ask for review before pushing.

## Phase 6: Power Profile Control

Branch: `phase-6-power-profiles`

- [ ] Read profile choices from `/sys/firmware/acpi/platform_profile_choices`.
- [ ] Read the active profile from `/sys/firmware/acpi/platform_profile`.
- [ ] Add UI actions for selecting a profile.
- [ ] Implement profile writes through the agreed privileged path.
- [ ] Surface permission/setup errors clearly in the UI.
- [ ] Run `cargo check`.
- [ ] Commit each completed subtask.
- [ ] Ask for review before pushing.

## Phase 7: NBFC Fan Control

Branch: `phase-7-fan-control`

- [ ] Detect whether `nbfc` is available.
- [ ] Detect whether `nbfc_service` appears usable.
- [ ] Add CPU and GPU fan sliders.
- [ ] Implement `nbfc set -f 0 -s <pct>` and `nbfc set -f 1 -s <pct>`.
- [ ] Implement automatic fan restore with `nbfc set --auto`.
- [ ] Surface command failures clearly in the UI.
- [ ] Run `cargo check`.
- [ ] Commit each completed subtask.
- [ ] Ask for review before pushing.

## Phase 8: Notifications

Branch: `phase-8-notifications`

- [ ] Implement thermal threshold evaluation.
- [ ] Add cooldown tracking for each alert type.
- [ ] Send desktop notifications with `notify-rust`.
- [ ] Avoid repeated notifications while temperatures remain above a threshold.
- [ ] Run `cargo check`.
- [ ] Commit each completed subtask.
- [ ] Ask for review before pushing.

## Phase 9: Tray Integration

Branch: `phase-9-tray`

- [ ] Add system tray setup.
- [ ] Add tray menu actions for show, quit, and quick profile switching.
- [ ] Add temperature-aware tray icon color logic.
- [ ] Implement minimize-to-tray behavior.
- [ ] Verify behavior on Fedora KDE Plasma Wayland.
- [ ] Run `cargo check`.
- [ ] Commit each completed subtask.
- [ ] Ask for review before pushing.

## Phase 10: Packaging and Desktop Integration

Branch: `phase-10-packaging`

- [ ] Add app icon assets.
- [ ] Add `build.rs` if needed for icon embedding.
- [ ] Add desktop entry template.
- [ ] Document Fedora package dependencies.
- [ ] Document setup for sudoers and NBFC.
- [ ] Run release build.
- [ ] Commit each completed subtask.
- [ ] Ask for review before pushing.

## Phase 11: Hardening and Polish

Branch: `phase-11-hardening`

- [ ] Audit error handling across sensor, profile, fan, tray, and notification paths.
- [ ] Add focused tests for pure logic.
- [ ] Verify UI layout at practical desktop sizes.
- [ ] Review config persistence needs.
- [ ] Update project documentation for handoff.
- [ ] Run final checks.
- [ ] Commit each completed subtask.
- [ ] Ask for review before pushing.
