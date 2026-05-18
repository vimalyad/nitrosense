# NitroSense Official App Analysis

This file tracks findings from the local `official_app/` folder. The folder is intentionally ignored by Git because it contains Acer-provided application files, while this document is tracked so the project can keep a portable summary of what was learned.

## Analysis Log

- Added `official_app/` to `.gitignore` so the official Acer app files stay local and do not enter this repository history.

## Source Package

- Local package: `official_app/Nitro Sense_Acer_3.01.3056_W11x64_A.zip`
- SHA-256: `f9185452fd6baf23ac33f417a35f98686e202e10f1bd1745908e5bfcab971622`
- ZIP root folder: `NitroSense_V3.01.3056_MSFT_SIGNED_20260424/`
- Package date visible in archive names: `2026-04-24`
- Major contents:
  - `NitroSense.msi`
  - `Setup.exe`
  - `Setup.exe.config`
  - `NitroSenseV3.1/df1e52f05c3d4982bd3ea1b7e3f591b5.appxbundle`
  - `NitroSenseV3.1/*Microsoft.NET.Native*` dependencies
  - `NitroSenseV3.1/*Microsoft.VCLibs*` dependencies
  - `Plugs/` with model-specific configuration folders

## Current Confirmed Facts

- The official Acer package uses per-model plug-in folders under `Plugs/`.
- The package includes a folder for `Plugs/Nitro AN515-58/`, matching the model we have been targeting in the app UI.
- The package also includes configuration for many related Nitro models, including `AN515-44`, `AN515-46`, `AN515-57`, `AN517-55`, and others.
- The official package files are kept under ignored `official_app/`; only this findings document is tracked.

## AN515-58 Plug-in Configuration

Files inspected:

- `Plugs/Nitro AN515-58/Feature.ini`
- `Plugs/Nitro AN515-58/HW_Support.ini`
- `Plugs/Nitro AN515-58/NitroSense.ini`

Confirmed settings:

- `Feature.ini`
  - `MachineType.Type=2`
  - `LauchTask.DelayTime=7`
  - `YearType.Year=2022`
  - `Planet9Support.Planet9=1`
  - `Planet9Support.Planet9_Link=https://p9.gg/NitroSense`
- `HW_Support.ini`
  - Lighting type is `Type:1`
  - Per-key RGB is disabled: `PerKey=0`
  - Default zone colors are all red: `Zone1` through `Zone4` are `#FF0000`
  - Advanced settings count is 3: `Animation`, `LCD`, `Temperature`
  - Keyboard settings count is 3: `Backlight`, `Sticky_Key`, `Windowskey1`
  - Fan support:
    - `CPU=1`
    - `GPU=1`
    - `System=0`
  - Fan detail type is `Type:1`
- `NitroSense.ini`
  - `OC_CPU.Count=0`, so this model plug-in does not define CPU overclock entries.
  - `OC_GPU.Count=11`, so this model plug-in defines GPU overclock behavior for 11 NVIDIA PCI IDs.
  - Each listed GPU entry uses:
    - `Quiet=0`
    - `Default=0`
    - `Performance=100`
    - `RAM_Quiet=0`
    - `RAM_Default=0`
    - `RAM_Performance=200`

## MSI Payload

The MSI extracts these notable files:

- `FILE_APP_SVC_EXE`: native x64 console service binary.
- `FILE_APP_ADMINAGENT_EXE`: native x64 admin agent binary.
- `FILE_APP_AGENT_EXE`: native x64 agent binary.
- `FILE_APP_SDK_WRAPPER_DLL`: x64 mixed/native .NET wrapper DLL.
- `FILE_APP_PROFILE_HELPER_DLL`: .NET profile helper DLL.
- `FILE_APP_INTEL_OC_SDK_DLL`: .NET Intel overclocking SDK DLL.
- `FILE_APP_LAUNCHER_EXE`, `FILE_APP_CREATE_DEFAULT_PROFILE_EXE`, `FILE_APP_TOAST_CREATE_EXE`, `FILE_APP_UPGRADETOOL_EXE`.

Important service/import findings:

- `FILE_APP_SVC_EXE` imports `SdkWrapper.dll` and calls these exported C++ methods:
  - `Initialize`
  - `ApplyProfile`
  - `GetMonitorValue`
  - `GetMaxTurboBoostCPUSpeed`
  - `IsSupportOC`
- `FILE_APP_SVC_EXE` uses Windows service APIs:
  - `CreateServiceW`
  - `OpenServiceW`
  - `ControlService`
  - `DeleteService`
  - `StartServiceCtrlDispatcherW`
  - `SetServiceStatus`
- `FILE_APP_SVC_EXE` uses Windows named pipe APIs:
  - `CreateNamedPipeW`
  - `ConnectNamedPipe`
  - `DisconnectNamedPipe`
  - `WaitNamedPipeW`
  - `GetNamedPipeClientProcessId`
  - `ImpersonateNamedPipeClient`
- `FILE_APP_SVC_EXE` uses COM/WMI APIs:
  - `CoInitializeEx`
  - `CoCreateInstance`
  - `CoSetProxyBlanket`
  - WMI namespace strings include `ROOT\WMI`.
- `FILE_APP_SVC_EXE` imports SetupAPI functions:
  - `SetupDiGetClassDevsW`
  - `SetupDiEnumDeviceInfo`
  - `SetupDiGetDeviceRegistryPropertyW`

## Appx Package

- Package identity: `AcerIncorporated.NitroSenseV31`
- Version: `3.1.3056.0`
- Publisher display name: `Acer Incorporated`
- Application ID: `App`
- Executable: `Win32\NitroSense.exe`
- Entry point: `Windows.FullTrustApplication`
- Capability: `runFullTrust`
- Target framework metadata: `.NETCore,Version=v5.0`
- Optimizing toolset: `ilc.exe`, which means the packaged UI is .NET Native compiled.

## Service and WMI Architecture

The official app appears to use this architecture:

1. Packaged UI process: `Win32\NitroSense.exe`
2. User/admin agent processes.
3. Windows service process: `PSSvc` / `NitroSense Service`.
4. Named pipe communication between UI/agents and service.
5. Service calls Acer WMI methods under `ROOT\WMI`.
6. Acer firmware/driver layer performs the actual hardware interaction.

Confirmed named-pipe/service strings:

- `predatorsense_service_namedpipe`
- `PredatorSense_service_namedpipe`
- `PredatorSense_admin_agent_`
- `NitroSense Service`
- `PSService in OnStart`
- `PSService in OnStop`

Confirmed Acer WMI classes and methods:

- WMI classes:
  - `AcerGamingFunction`
  - `APGeAction`
  - `APGeEvent`
- WMI queries:
  - `SELECT * FROM AcerGamingFunction`
  - `SELECT * FROM APGeAction`
  - `SELECT * FROM APGeEvent`
- WMI methods:
  - `SetGamingProfile`
  - `GetGamingProfile`
  - `SetGamingProfileSetting`
  - `SetGamingFanBehavior`
  - `SetGamingFanSpeed`
  - `SetGamingFanTable`
  - `SetGamingMiscSettingFunction`
  - `GetGamingMiscSettingFunction`
  - `SetGamingLEDBehaviorFunction`
  - `SetGamingRgbKbSettingFunction`
  - `SetGamingKbbacklightSettingFunction`

Fan-related service/UI command strings:

- `kSvcCmdGamingFanGroupBehaviorSetFunction`
- `kSvcCmdGamingFanGroupSpeedSetFunction`
- `SetAcerGamingFanGroupBehavior`
- `SetAcerGamingFanGroupSpeed`
- `SetFanCoolBoost`
- `CoolBoostMode`
- `CPUFanPercentage`
- `GPU1FanPercentage`
- `CurrentFanMode`
- `CPUFanCustomAuto`
- `GPU1FanCustomAuto`

Temperature/monitoring strings:

- `CPU_Templature` and `GPU_Templature` are present in the official UI binary. The misspelling appears in the compiled strings.
- `CPU_RPM`, `GPU_RPM`, `sCPU_Fan_Speed`, `sGPU_Fan_Speed`, `sCPU_Temperature`, and `sGPU1_Temperature` are present.
- `get_fan_speed_info_data` and `get_temperature_frequency_usage_info_data` are present.

Important inference:

- The official Windows app does not appear to write raw EC registers directly from the UI. It sends named-pipe commands to a service, and the service invokes Acer WMI methods. The EC details are probably hidden behind Acer's Windows WMI/firmware driver layer.

## Decompiled Managed Code Findings

`ilspycmd` version `9.1.0.7988` was installed into ignored `official_app/tools/` and used to decompile:

- `Win32\NitroSense.exe`
- `Win32\TsDotNetLib.dll`
- `Win32\Utilities.dll`

Useful recovered files:

- `official_app/decompiled/TsDotNetLib/TsDotNetLib/ServiceCommand.cs`
- `official_app/decompiled/TsDotNetLib/TsDotNetLib/ConstDef.cs`
- `official_app/decompiled/TsDotNetLib/TsDotNetLib/IPCMethods.cs`
- `official_app/decompiled/NitroSense/NitroSense/WMIFunction.cs`
- `official_app/decompiled/NitroSense/NitroSense/CommonFunction.cs`
- `official_app/decompiled/NitroSense/NitroSense/Nitro_FanControlPage.cs`
- `official_app/decompiled/NitroSense/NitroSense/Nitro_MainWindow.cs`

### Named Pipe Protocol

`TsDotNetLib.IPCMethods.SendCommandByNamedPipe` encodes pipe messages like this:

- Offset `0`: command code as 2 bytes.
- Byte `2`: number of arguments as 1 byte.
- For each argument:
  - 4-byte argument size.
  - argument bytes.
- Strings are UTF-16 with a trailing null.
- Non-string values are marshalled using `Marshal.StructureToPtr`.

Official pipe names:

- Service pipe: `PredatorSense_service_namedpipe`
- Admin agent pipe prefix: `PredatorSense_admin_agent_`

### Service Command Numbers

`ServiceCommand` is a zero-based enum. Important values:

- `9`: `kSvcCmdGamingPorfileWMISetFunction`
- `10`: `kSvcCmdGamingPorfileWMIGetFunction`
- `11`: `kSvcCmdGamingLEDGroupColorSetFunction`
- `12`: `kSvcCmdGamingLEDGroupColorGetFunction`
- `13`: `kSvcCmdGetGamingSysinfoFunction`
- `14`: `kSvcCmdGetGPUUsageLoading`
- `15`: `kSvcCmdGamingFanGroupBehaviorSetFunction`
- `16`: `kSvcCmdGamingFanGroupSpeedSetFunction`
- `17`: `kSvcCmdWMISetFunction`
- `20`: `kSvcCmdWMIGetFunction`
- `27`: `kSvcCmdWMISetGamingKBBacklight`
- `28`: `kSvcCmdWMISetGamingRgbKbSetting`
- `29`: `kSvcCmdWMISetGamingLEDBehavior`
- `30`: `kSvcCmdSetOperationMode`
- `31`: `kSvcCmdGPUGetCount`
- `32`: `kSvcCmdGPUGetFrequency`
- `33`: `kSvcCmdWMISetGamingMISCSetting`
- `34`: `kSvcCmdWMIGetGamingMISCSetting`

### Fan Modes

The official UI has three fan modes:

- `0`: Auto
- `1`: Max
- `2`: Custom

It persists the selected mode in:

- Registry path: `SOFTWARE\OEM\NitroSense\FanControl`
- Value: `CurrentFanMode`

The actual hardware call is `CommonFunction.set_all_fan_mode`, which calls `WMIFunction.SetAcerGamingFanGroupBehavior`.

Exact payloads:

- Auto: `0x410009`
- Max: `0x820009`
- Custom: `0xC30009`

These are built from base `9` plus mode bits:

- Auto adds `0x410000`
- Max adds `0x820000`
- Custom adds `0xC30000`

### Custom Fan Speed

Fan groups:

- CPU fan: `1`
- GPU fan: `4`

`CommonFunction.set_single_custom_fan_speed(percentage, fan_group_type)` builds:

- CPU: `1 | (percentage << 8)`
- GPU: `4 | (percentage << 8)`

Important UI scaling detail:

- The slider value is multiplied by `10` before sending.
- Example: UI slider `50` becomes payload percentage `500`.
- Therefore the official WMI payload does not appear to use a simple `0..100` percentage scale at the final WMI layer.

Registry values used for custom fan UI state:

- `CPUFanPercentage`
- `GPU1FanPercentage`
- `CPUFanCustomAuto`
- `GPU1FanCustomAuto`

### Custom Fan Auto State

`CommonFunction.set_single_custom_fan_state(auto, percentage, fan_group_type)` uses:

- `num2 = auto ? 1 : 3`
- CPU behavior payload: `1 | (num2 << 16)`
- GPU behavior payload: `8 | (num2 << 22)`
- Then sends fan speed:
  - CPU: `1 | (percentage << 8)`
  - GPU: `4 | (percentage << 8)`

`CommonFunction.set_all_custom_fan_state(auto_list, percentage_list)` uses:

- `num2 = auto[0] ? 1 : 3`
- `num3 = auto[1] ? 1 : 3`
- Behavior payload: `9 | (num2 << 16) | (num3 << 22)`
- Then sends CPU and GPU speed payloads separately.

### CoolBoost

CoolBoost UI state is stored in:

- Registry path: `SOFTWARE\OEM\NitroSense\FanControl`
- Value: `CoolBoostMode`

Hardware call:

- `CommonFunction.set_coolboost_state(state)`
- Sends `WMIFunction.WMISetFunction(7 | ((state ? 1 : 0) << 16))`

Readback:

- `CommonFunction.get_coolboost_state()`
- Calls `WMIFunction.WMIGetFunction(519)`
- Treats result as enabled if:
  - low byte is `0`
  - byte at `result >> 8` is `1`

### Temperature and RPM Readback

The official UI reads system health using:

- `CommonFunction.get_wmi_system_health_info(ref info_data, index)`
- This calls `WMIFunction.GetAcerGamingSystemInformation(1 | (index << 8))`.
- If the low byte of the result is `0`, the data value is `(result >> 8) & 0xFFFF`.

Recovered `System_Health_Information_Index` enum:

- `1`: `sCPU_Temperature`
- `2`: `sCPU_Fan_Speed`
- `3`: `sSystem_Temperature`
- `4`: `sSystem_Fan_Speed`
- `5`: `sFrostCore`
- `6`: `sGPU_Fan_Speed`
- `7`: `sSystem2_Temperature`
- `8`: `sSystem2_Fan_Speed`
- `9`: `sGPU2_Fan_Speed`
- `10`: `sGPU1_Temperature`
- `11`: `sGPU2_Temperature`

The official main window reads:

- CPU temperature with index `1`
- GPU1 temperature with index `10`
- CPU fan speed with index `2`
- GPU fan speed with index `6`

### Operation Modes

Recovered operation mode enum:

- `0`: Quiet
- `1`: Default
- `2`: Render
- `3`: Edit
- `4`: Extreme
- `5`: Turbo

`MonitorAndOCClass.set_operation_mode` sends command `30` over `PredatorSense_service_namedpipe` with the selected operation mode as `uint`.

## Practical Linux Implication

The best Linux target is not "copy Windows EC registers"; the official app shows that Acer exposes a higher-level WMI gaming interface on Windows:

- fan behavior
- fan speed
- profile/mode
- system health
- keyboard lighting
- misc settings

On Linux, the closest robust equivalent would be one of:

1. Find whether the same WMI methods are exposed through Linux `wmi`/`acer-wmi`/ACPI interfaces.
2. Add a Linux backend that mirrors these payload encodings through an available Acer WMI bridge if one exists.
3. Only fall back to direct EC writes if WMI/ACPI methods are unavailable and model support is verified.

## Local AN515-58 Linux Capability Scan

Target machine observed locally:

- Vendor: `Acer`
- Product: `Nitro AN515-58`
- Family: `Nitro 5`
- BIOS version: `V2.14`

Loaded relevant kernel modules:

- `acer_wmi`
- `wmi_bmof`
- `acpi_ec`
- `platform_profile`
- `nvidia_wmi_ec_backlight`

Platform profile support:

- Current profile during scan: `performance`
- Available choices: `low-power quiet balanced balanced-performance performance`

`acer_wmi` module parameters visible through `modinfo`:

- `ec_raw_mode=N`
- `cycle_gaming_thermal_profile=Y`
- `predator_v4=N`
- `force_series=0`
- `force_caps=-1`

Important finding:

- Linux already exposes an Acer hwmon adapter for this laptop under
  `/sys/devices/platform/acer-wmi/hwmon/hwmon5` during the scan.
- The dynamic `/sys/class/hwmon/hwmon*` number can change between boots, so the
  app must discover the adapter by reading `name=acer`.

Observed Acer hwmon files and values:

- `name`: `acer`
- `fan1_input`: around `6521` to `7500`
- `fan2_input`: around `6382` to `7500`
- `temp1_input`: around `91000`
- `temp2_input`: around `48000`
- `temp3_input`: around `49000` to `50000`
- `pwm1_enable`: `1`
- `pwm1`: `255`
- `pwm2_enable`: `1`
- `pwm2`: `255`

Practical implementation decision:

- Use the Linux `acer-wmi` hwmon PWM files as the first native fan-control
  backend for this custom AN515-58 app.
- Map UI percentage `0..100` to Linux PWM `0..255`.
- Set manual mode by writing `pwmN_enable=1`, then writing `pwmN`.
- Try restoring automatic mode through the standard hwmon convention
  `pwmN_enable=2`; if the driver rejects it, the app should report that error
  and the user can use platform profiles while we investigate the Acer WMI method
  bridge further.
- Avoid raw EC writes for now because the official Windows app also uses a
  higher-level Acer WMI/driver interface rather than direct UI-side EC access.
