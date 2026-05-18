#!/usr/bin/env bash
set -euo pipefail

APP_NAME="nitrosense"
VERSION="${1:-0.1.0-alpha}"
TARGET_TRIPLE="$(rustc -vV | awk '/host:/ { print $2 }')"
ARCHIVE_NAME="${APP_NAME}-${VERSION}-${TARGET_TRIPLE}"
DIST_DIR="dist"
STAGE_DIR="${DIST_DIR}/${ARCHIVE_NAME}"

cargo build --release

rm -rf "${STAGE_DIR}"
mkdir -p \
  "${STAGE_DIR}/bin" \
  "${STAGE_DIR}/share/applications" \
  "${STAGE_DIR}/share/icons/hicolor" \
  "${STAGE_DIR}/docs"

install -Dm755 "target/release/${APP_NAME}" "${STAGE_DIR}/bin/${APP_NAME}"
install -Dm644 "packaging/${APP_NAME}.desktop" "${STAGE_DIR}/share/applications/${APP_NAME}.desktop"
cp -R "assets/icons/hicolor/." "${STAGE_DIR}/share/icons/hicolor/"
install -Dm644 "README.md" "${STAGE_DIR}/README.md"
install -Dm644 "my_laptop.md" "${STAGE_DIR}/my_laptop.md"
install -Dm644 "nitrosense_info.md" "${STAGE_DIR}/nitrosense_info.md"
install -Dm644 "docs/setup.md" "${STAGE_DIR}/docs/setup.md"
install -Dm644 "docs/official-app-analysis.md" "${STAGE_DIR}/docs/official-app-analysis.md"
install -Dm644 "docs/handoff.md" "${STAGE_DIR}/docs/handoff.md"

cat > "${STAGE_DIR}/INSTALL.md" <<'EOF'
# NitroSense Linux Install

This archive is intended for Acer Nitro AN515-58 systems with matching Linux
`acer-wmi` hwmon support.

## Install For Current User

From this extracted archive:

```bash
install -Dm755 bin/nitrosense ~/.local/bin/nitrosense
cp -R share/icons/hicolor/. ~/.local/share/icons/hicolor/
install -Dm644 share/applications/nitrosense.desktop ~/.local/share/applications/nitrosense.desktop
update-desktop-database ~/.local/share/applications
gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor 2>/dev/null || true
```

Then launch `NitroSense` from your application menu or run:

```bash
nitrosense
```

## Runtime Notes

Fan writes use Polkit through `pkexec nitrosense --fan-helper ...`; profile
writes still use sudo. See `docs/setup.md`.

This is alpha hardware-control software. Use only if your laptop exposes the
expected Acer hwmon PWM controls.
EOF

tar -C "${DIST_DIR}" -czf "${DIST_DIR}/${ARCHIVE_NAME}.tar.gz" "${ARCHIVE_NAME}"
sha256sum "${DIST_DIR}/${ARCHIVE_NAME}.tar.gz" > "${DIST_DIR}/${ARCHIVE_NAME}.tar.gz.sha256"

echo "Created ${DIST_DIR}/${ARCHIVE_NAME}.tar.gz"
echo "Created ${DIST_DIR}/${ARCHIVE_NAME}.tar.gz.sha256"
