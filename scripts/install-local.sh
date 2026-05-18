#!/usr/bin/env bash
set -euo pipefail

APP_NAME="nitrosense"
BUILD_PROFILE="${1:-release}"
BIN_DIR="${HOME}/.local/bin"
DATA_DIR="${XDG_DATA_HOME:-${HOME}/.local/share}"
APP_DIR="${DATA_DIR}/applications"
ICON_DIR="${DATA_DIR}/icons/hicolor"
POLKIT_ACTION_DIR="/usr/share/polkit-1/actions"
POLKIT_ACTION_ID="io.github.vimalyad.nitrosense.fan-control"
POLKIT_POLICY="${POLKIT_ACTION_DIR}/${POLKIT_ACTION_ID}.policy"
POLKIT_POLICY_TEMPLATE="packaging/io.github.vimalyad.nitrosense.policy.in"

case "${BUILD_PROFILE}" in
  release)
    cargo build --release
    SOURCE_BINARY="target/release/${APP_NAME}"
    ;;
  debug)
    cargo build
    SOURCE_BINARY="target/debug/${APP_NAME}"
    ;;
  *)
    echo "usage: scripts/install-local.sh [release|debug]" >&2
    exit 2
    ;;
esac

install -Dm755 "${SOURCE_BINARY}" "${BIN_DIR}/${APP_NAME}"
cp -R "assets/icons/hicolor/." "${ICON_DIR}/"
test -f "${ICON_DIR}/index.theme" || cp "/usr/share/icons/hicolor/index.theme" "${ICON_DIR}/index.theme"

desktop-file-install \
  --dir="${APP_DIR}" \
  --set-key=Exec \
  --set-value="${BIN_DIR}/${APP_NAME}" \
  --set-icon="${ICON_DIR}/256x256/apps/${APP_NAME}.png" \
  "packaging/${APP_NAME}.desktop"

update-desktop-database "${APP_DIR}"
gtk-update-icon-cache -f -t "${ICON_DIR}" 2>/dev/null || true
xdg-icon-resource forceupdate --theme hicolor --mode user
kbuildsycoca6 --noincremental 2>/dev/null || true

policy_file="$(mktemp)"
trap 'rm -f "${policy_file}"' EXIT
escaped_binary="$(printf '%s' "${BIN_DIR}/${APP_NAME}" | sed 's/[\/&]/\\&/g')"
sed "s/@NITROSENSE_EXEC@/${escaped_binary}/g" \
  "${POLKIT_POLICY_TEMPLATE}" > "${policy_file}"
sudo install -Dm644 "${policy_file}" "${POLKIT_POLICY}"

echo "Installed ${BIN_DIR}/${APP_NAME}"
echo "Installed Polkit action ${POLKIT_POLICY}"
