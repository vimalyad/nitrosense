# My Laptop

This file tracks the exact laptop details used to keep NitroSense custom to this
machine instead of making it a generic Acer utility.

## Identity

- Vendor: `Acer`
- Product name: `Nitro AN515-58`
- Product family: `Nitro 5`
- BIOS version observed: `V2.14`
- Target app label: `Acer Nitro AN515-58`
- Operating system target: Fedora KDE Plasma Wayland

## Official Acer NitroSense Package

- Official package kept locally under ignored `official_app/`.
- Package file:
  `official_app/Nitro Sense_Acer_3.01.3056_W11x64_A.zip`
- SHA-256:
  `f9185452fd6baf23ac33f417a35f98686e202e10f1bd1745908e5bfcab971622`
- ZIP root folder:
  `NitroSense_V3.01.3056_MSFT_SIGNED_20260424/`
- Official app package version: `3.1.3056.0`
- Official package date visible in archive names: `2026-04-24`

## Official AN515-58 Plug-in Facts

From `Plugs/Nitro AN515-58/`:

- `Feature.ini`
  - `MachineType.Type=2`
  - `YearType.Year=2022`
  - `Planet9Support.Planet9=1`
- `HW_Support.ini`
  - CPU fan supported: `CPU=1`
  - GPU fan supported: `GPU=1`
  - System fan unsupported: `System=0`
  - Lighting type: `Type:1`
  - Per-key RGB disabled: `PerKey=0`
  - Default keyboard zones are red: `#FF0000`
- `NitroSense.ini`
  - CPU overclock entries: `OC_CPU.Count=0`
  - GPU overclock entries: `OC_GPU.Count=11`
  - GPU performance values use `Performance=100`
  - GPU RAM performance values use `RAM_Performance=200`

## Official Fan and Sensor Mapping

The official Windows app talks to an Acer service, and that service calls Acer
WMI methods under `ROOT\WMI`.

- Fan behavior command: `SetAcerGamingFanGroupBehavior`
- Fan speed command: `SetAcerGamingFanGroupSpeed`
- CPU fan group flag: `1`
- GPU fan group flag: `4`
- Official fan modes:
  - `0`: Auto
  - `1`: Max
  - `2`: Custom
- Official behavior payloads:
  - Auto: `0x410009`
  - Max: `0x820009`
  - Custom: `0xC30009`
- Official custom speed payload:
  - CPU: `1 | (percentage << 8)`
  - GPU: `4 | (percentage << 8)`
- Official UI multiplies the slider value by `10` before sending it to WMI.

Official system-health readback indices:

- `1`: CPU temperature
- `2`: CPU fan speed
- `6`: GPU fan speed
- `10`: GPU1 temperature

## Linux Kernel Capability Scan

Relevant modules observed:

- `acer_wmi`
- `wmi_bmof`
- `acpi_ec`
- `platform_profile`
- `nvidia_wmi_ec_backlight`

Platform profile support:

- Observed active profile during scan: `performance`
- Choices:
  - `low-power`
  - `quiet`
  - `balanced`
  - `balanced-performance`
  - `performance`

`acer_wmi` parameters observed through `modinfo`:

- `ec_raw_mode=N`
- `cycle_gaming_thermal_profile=Y`
- `predator_v4=N`
- `force_series=0`
- `force_caps=-1`

## Acer Hwmon Adapter

Linux exposes an Acer hwmon adapter through `acer-wmi`.

- Observed concrete path during scan:
  `/sys/devices/platform/acer-wmi/hwmon/hwmon5`
- The `/sys/class/hwmon/hwmonN` number can change after reboot.
- App detection must read hwmon `name` and select the adapter where
  `name=acer`.

Observed files:

- `fan1_input`: CPU fan RPM
- `fan2_input`: GPU fan RPM
- `temp1_input`
- `temp2_input`
- `temp3_input`
- `pwm1_enable`
- `pwm1`
- `pwm2_enable`
- `pwm2`

Observed values during scan:

- `fan1_input`: around `6521` to `7500`
- `fan2_input`: around `6382` to `7500`
- `temp1_input`: around `91000`
- `temp2_input`: around `48000`
- `temp3_input`: around `49000` to `50000`
- `pwm1_enable`: `1`
- `pwm1`: `255`
- `pwm2_enable`: `1`
- `pwm2`: `255`

## GPU Temperature Availability on Linux

PCI devices observed:

- Intel Alder Lake-P GT1 UHD Graphics
- NVIDIA GA104 RTX 3070 Ti Laptop GPU

Linux scan result:

- No `/sys/class/hwmon` adapter named `nvidia` was exposed during the scan.
- No `/sys/class/hwmon` adapter named `i915` was exposed during the scan.
- `nvidia-smi` was installed but could not communicate with the NVIDIA driver.
- `lspci -nnk` shows the Intel GPU uses the `i915` kernel driver.
- `/sys/class/drm/card1/device/hwmon` does not exist for the Intel GPU.
- `/sys/class/drm/card1/device/power_state` reported `D0`, so the Intel GPU was
  awake during the check.
- `lm_sensors` did not report an Intel GPU temperature sensor.
- Generic thermal zones were visible (`SEN2`, `SEN3`, `TCPU`, `TCPU_PCI`,
  `x86_pkg_temp`), but none are labeled as Intel GPU temperature.
- `/sys/kernel/debug/dri` may contain i915 debug data, but it requires root
  access and is not a stable user-facing app data source.

Implementation decision:

- Use native `nvidia` hwmon temperature first if the NVIDIA driver exposes it in
  the future.
- Fall back to Acer firmware hwmon `temp3_input` for the discrete NVIDIA GPU
  temperature on this AN515-58.
- Fall back to Acer firmware hwmon `temp2_input` only if `temp3_input` is
  missing.
- Do not show Intel GPU temperature in the app unless the Linux kernel exposes a
  real Intel GPU temperature source such as an `i915` hwmon adapter.

## Current Linux Implementation Decision

- Use `acer-wmi` hwmon PWM as the first fan-control backend.
- Keep fan control custom to AN515-58.
- Keep raw EC writes out of the app unless the kernel/WMI path proves
  insufficient and a safe model-specific register map is verified.
- Map UI fan percentage `0..100` to Linux PWM `0..255`.
- Manual mode writes:
  - `pwmN_enable=1`
  - `pwmN=<mapped pwm>`
- Auto mode currently tries the standard hwmon value:
  - `pwmN_enable=2`

## App Features Relevant to This Laptop

- Read CPU/GPU temperatures from hwmon.
- Read CPU/GPU fan RPM from Acer hwmon.
- Read NVMe temperature. Battery voltage is not shown or read in the current UI.
- Switch Linux platform profiles.
- Control CPU/GPU fan PWM through Acer hwmon.
- Keep temperature graphs plotted in a fixed `0..105 C` range while only labeling
  the Y-axis up to `100 C`; values above `100 C` should remain visible without
  expanding the graph range.
- Keep the GUI fixed at `920x600` with a `176px` sidebar, matching the custom
  AN515-58 dashboard layout.
- Keep the GUI single-instance guarded so two hardware-control windows do not
  run at the same time.
