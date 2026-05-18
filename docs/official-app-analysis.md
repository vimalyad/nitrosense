# Official NitroSense Local Analysis

This guide shows how to reproduce the official Acer NitroSense extraction and
analysis locally without this repository redistributing Acer's proprietary
installer, binaries, images, fonts, or decompiled output.

The commands assume you are at the repository root.

## 1. Prepare Local Folder

`official_app/` is ignored by Git except for `official_app/README.md`.

```bash
mkdir -p official_app
```

Download the official Acer NitroSense package for your laptop model from Acer's
support site and place the ZIP in `official_app/`.

For the AN515-58 analysis in this repo, the local file used was:

```text
official_app/Nitro Sense_Acer_3.01.3056_W11x64_A.zip
```

Its observed SHA-256 was:

```text
f9185452fd6baf23ac33f417a35f98686e202e10f1bd1745908e5bfcab971622
```

Verify your local file:

```bash
sha256sum "official_app/Nitro Sense_Acer_3.01.3056_W11x64_A.zip"
```

## 2. Install Analysis Tools

On Fedora:

```bash
sudo dnf install unzip p7zip p7zip-plugins file binutils dotnet-sdk-9.0
```

Install ILSpy command-line decompiler into the ignored local folder:

```bash
mkdir -p official_app/tools
dotnet tool install ilspycmd --tool-path official_app/tools
```

If `dotnet-sdk-9.0` is unavailable on your distribution, install the current
.NET SDK package from your distro or from Microsoft's official .NET packages.

## 3. Extract the Outer ZIP

```bash
mkdir -p official_app/extracted
unzip -q "official_app/Nitro Sense_Acer_3.01.3056_W11x64_A.zip" -d official_app/extracted
find official_app/extracted -maxdepth 3 -type f | sort
```

For the package analyzed here, the important root folder was:

```text
official_app/extracted/NitroSense_V3.01.3056_MSFT_SIGNED_20260424/
```

## 4. Inspect Model Plug-in Files

The AN515-58 plug-in files are plain text INI files:

```bash
find official_app/extracted -path "*Plugs*Nitro AN515-58*" -type f | sort
```

Typical files to inspect:

```bash
sed -n '1,220p' "official_app/extracted/NitroSense_V3.01.3056_MSFT_SIGNED_20260424/Plugs/Nitro AN515-58/Feature.ini"
sed -n '1,220p' "official_app/extracted/NitroSense_V3.01.3056_MSFT_SIGNED_20260424/Plugs/Nitro AN515-58/HW_Support.ini"
sed -n '1,260p' "official_app/extracted/NitroSense_V3.01.3056_MSFT_SIGNED_20260424/Plugs/Nitro AN515-58/NitroSense.ini"
```

Record useful facts in `nitrosense_info.md` or `my_laptop.md`; do not commit the
official files themselves.

## 5. Extract the MSI

The package contains `NitroSense.msi`. Extract it into an ignored folder:

```bash
mkdir -p official_app/msi_extracted
7z x -y \
  "official_app/extracted/NitroSense_V3.01.3056_MSFT_SIGNED_20260424/NitroSense.msi" \
  -oofficial_app/msi_extracted
find official_app/msi_extracted -maxdepth 1 -type f | sort
```

Useful checks:

```bash
file official_app/msi_extracted/*
strings -a official_app/msi_extracted/FILE_APP_SVC_EXE | rg -i "wmi|pipe|fan|gaming|profile|acer"
strings -a official_app/msi_extracted/FILE_APP_SDK_WRAPPER_DLL | rg -i "fan|profile|monitor|wmi|acer"
```

## 6. Extract the AppX Bundle

The ZIP contains an AppX bundle under `NitroSenseV3.1/`.

```bash
mkdir -p official_app/appxbundle_extracted
7z x -y \
  "official_app/extracted/NitroSense_V3.01.3056_MSFT_SIGNED_20260424/NitroSenseV3.1/df1e52f05c3d4982bd3ea1b7e3f591b5.appxbundle" \
  -oofficial_app/appxbundle_extracted
find official_app/appxbundle_extracted -maxdepth 2 -type f | sort
```

Extract the x64 AppX:

```bash
mkdir -p official_app/appx_x64_extracted
7z x -y \
  "official_app/appxbundle_extracted/CentenialConvert_3.1.3056.0_x64.appx" \
  -oofficial_app/appx_x64_extracted
find official_app/appx_x64_extracted -maxdepth 3 -type f | sort
```

Useful files usually include:

```text
official_app/appx_x64_extracted/AppxManifest.xml
official_app/appx_x64_extracted/Win32/NitroSense.exe
official_app/appx_x64_extracted/Win32/TsDotNetLib.dll
official_app/appx_x64_extracted/Win32/Utilities.dll
```

## 7. Decompile Managed Assemblies

Create output folders:

```bash
mkdir -p official_app/decompiled
```

Run ILSpy:

```bash
official_app/tools/ilspycmd \
  -p \
  -o official_app/decompiled/NitroSense \
  official_app/appx_x64_extracted/Win32/NitroSense.exe

official_app/tools/ilspycmd \
  -p \
  -o official_app/decompiled/TsDotNetLib \
  official_app/appx_x64_extracted/Win32/TsDotNetLib.dll

official_app/tools/ilspycmd \
  -p \
  -o official_app/decompiled/Utilities \
  official_app/appx_x64_extracted/Win32/Utilities.dll
```

Useful files to inspect after decompilation:

```bash
sed -n '1,240p' official_app/decompiled/TsDotNetLib/TsDotNetLib/ServiceCommand.cs
sed -n '1,260p' official_app/decompiled/TsDotNetLib/TsDotNetLib/IPCMethods.cs
sed -n '1,260p' official_app/decompiled/NitroSense/NitroSense/CommonFunction.cs
sed -n '1,260p' official_app/decompiled/NitroSense/NitroSense/WMIFunction.cs
sed -n '1,260p' official_app/decompiled/NitroSense/NitroSense/Nitro_FanControlPage.cs
```

Search for fan, thermal, WMI, and pipe behavior:

```bash
rg -n "Fan|fan|CoolBoost|WMI|NamedPipe|Gaming|Temperature|RPM|OperationMode|AcerGaming" official_app/decompiled
```

## 8. Inspect UI Resources Locally

The official UI resources may include images, fonts, and BAML/XAML-derived
files. Keep them local only.

```bash
find official_app/decompiled/NitroSense -iname "*.png" -o -iname "*.ico" -o -iname "*.baml" | sort
file official_app/decompiled/NitroSense/images/150/img_bg_nitro.png
file official_app/decompiled/NitroSense/images/150/img_fan.png
```

Use these only as local visual reference. Do not copy Acer image/font assets into
this repository.

## 9. Record Findings

Portable findings should be summarized in tracked Markdown:

```text
nitrosense_info.md
my_laptop.md
```

Good findings to record:

- package version and checksum
- model plug-in support flags
- fan group constants
- WMI/service command numbers
- sensor readback indices
- Linux kernel interfaces that match the official behavior
- implementation decisions and open questions

Do not commit:

- Acer ZIP/MSI/AppX packages
- extracted binaries
- decompiled source output
- official images/icons/fonts
- generated tool directories
