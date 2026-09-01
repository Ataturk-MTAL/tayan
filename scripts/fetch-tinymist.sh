#!/usr/bin/env bash
# tinymist dil sunucusunu indirir ve sha256 ile doğrular.
#
# İkili depoya GİRMEZ: platform başına 60 MB, dört platform 240 MB eder ve git
# geçmişini kalıcı olarak şişirir. Bunun yerine derleme öncesi indirilir.
#
# Kullanım: scripts/fetch-tinymist.sh
set -euo pipefail

VERSION="v0.15.2"
DEST="$(cd "$(dirname "$0")/.." && pwd)/apps/tayan-desktop/src-tauri/binaries"

case "$(uname -s)-$(uname -m)" in
  Darwin-arm64)  ASSET="tinymist-aarch64-apple-darwin.tar.gz";        TRIPLE="aarch64-apple-darwin" ;;
  Darwin-x86_64) ASSET="tinymist-x86_64-apple-darwin.tar.gz";         TRIPLE="x86_64-apple-darwin" ;;
  Linux-x86_64)  ASSET="tinymist-x86_64-unknown-linux-gnu.tar.gz";    TRIPLE="x86_64-unknown-linux-gnu" ;;
  Linux-aarch64) ASSET="tinymist-aarch64-unknown-linux-gnu.tar.gz";   TRIPLE="aarch64-unknown-linux-gnu" ;;
  *) echo "Desteklenmeyen platform: $(uname -s)-$(uname -m)" >&2; exit 1 ;;
esac

TARGET="$DEST/tinymist-$TRIPLE"
if [ -x "$TARGET" ]; then
  echo "zaten var: $TARGET"
  exit 0
fi

BASE="https://github.com/Myriad-Dreamin/tinymist/releases/download/$VERSION"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "indiriliyor: $ASSET ($VERSION)"
curl -fsSL -o "$TMP/a.tar.gz" "$BASE/$ASSET"
curl -fsSL -o "$TMP/a.sha256" "$BASE/$ASSET.sha256"

# Doğrulama atlanamaz: dışarıdan indirilen ve kullanıcının makinesinde
# çalıştırılacak bir ikili.
EXPECTED="$(awk '{print $1}' "$TMP/a.sha256")"
if command -v shasum >/dev/null; then
  ACTUAL="$(shasum -a 256 "$TMP/a.tar.gz" | awk '{print $1}')"
else
  ACTUAL="$(sha256sum "$TMP/a.tar.gz" | awk '{print $1}')"
fi
if [ "$EXPECTED" != "$ACTUAL" ]; then
  echo "sha256 UYUŞMADI" >&2
  echo "  beklenen: $EXPECTED" >&2
  echo "  gerçek  : $ACTUAL" >&2
  exit 1
fi
echo "sha256 doğrulandı: $ACTUAL"

tar xzf "$TMP/a.tar.gz" -C "$TMP"
BIN="$(find "$TMP" -type f -name tinymist -perm -u+x | head -1)"
[ -n "$BIN" ] || { echo "arşivde tinymist bulunamadı" >&2; exit 1; }

mkdir -p "$DEST"
cp "$BIN" "$TARGET"
chmod +x "$TARGET"
echo "kuruldu: $TARGET  ($(du -h "$TARGET" | awk '{print $1}'))"
