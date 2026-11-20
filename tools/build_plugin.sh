#!/usr/bin/env bash
# Build the plugin and make the install folder (run in bash: PowerShell 5 fails because of cargo's stderr)
#   bash tools/build_plugin.sh
# Output: dist/MHGU-Skill-Simulator/romfs/nativeNX/plugins/
#         skill_simulator.nro
#         skill-simulator/{font.otf, atlas.rgba, names.<language>.bin, NOTO_LICENSE.txt}
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# Paths specific to my PC (devkitARM, game install folders) live in tools/local.sh, which is not committed.
# See tools/local.example.sh. Without it, the install step is skipped.
if [ -f "$ROOT/tools/local.sh" ]; then
  . "$ROOT/tools/local.sh"
fi
if [ -n "${DEVKITARM:-}" ]; then
  export PATH="$DEVKITARM/bin:$PATH"
fi

cd "$ROOT"
# Regenerate the tables only where the game database sits next to this repository (../Database). Without it (a fresh
# clone), the committed plugin/src/generated and plugin/assets are used as they are.
if [ -f "$ROOT/../Database/mhxx_all_tables.json" ]; then
  PYTHONIOENCODING=utf-8 py -3.10 tools/build_plugin_data.py
  PYTHONIOENCODING=utf-8 py -3.10 tools/build_armor_map.py || { echo "방어구 대응표 검증 실패"; exit 1; }
else
  echo "../Database 없음: 저장소에 들어 있는 생성 데이터를 그대로 씁니다"
fi
cd "$ROOT/plugin"
cargo +nightly forge build --release

# There must be no packed saturating ops that Recomp rejects
ELF=target/armv7a-none-eabi/release/libskill_simulator.so
if arm-none-eabi-objdump -d "$ELF" | grep -qE '\b(uqadd8|uqadd16|uqsub8|uqsub16|uqasx|uqsax)\b'; then
  echo "Recomp 미지원 명령어(묶음 포화 연산)가 있다" >&2
  exit 1
fi
# NEON 64-bit saturating add/sub (vqadd/vqsub .u64/.s64) are unknown to the Recomp interpreter too
# (2026-11-18 crash: the compiler fused two u64 saturating_sub into one vqsub.u64). Replace with plain arithmetic.
if arm-none-eabi-objdump -d "$ELF" | grep -E '\bvq(add|sub)\.[us]64\b'; then
  echo "Recomp 미지원 명령어(NEON 64비트 포화 연산)가 있다" >&2
  exit 1
fi
# Must fit comfortably in the Recomp plugin memory area (12 MB) shared by all plugins
py -3.10 - <<'EOF'
import struct
b = open('skill_simulator.nro', 'rb').read()
need = len(b) + struct.unpack_from('<I', b, 0x38)[0] + 0x10000
print(f'Recomp 적재 크기 {need} / 12582912')
assert need < 0x300000, '3MB 를 넘는다: 다른 모드 자리를 남겨야 한다'
EOF

OUT="$ROOT/dist/MHGU-Skill-Simulator/romfs/nativeNX/plugins"
rm -rf "$ROOT/dist/MHGU-Skill-Simulator"
mkdir -p "$OUT/skill-simulator"
cp skill_simulator.nro "$OUT/"
cp assets/font.otf assets/atlas.rgba assets/names.*.bin assets/NOTO_LICENSE.txt "$OUT/skill-simulator/"
echo "설치 폴더: $ROOT/dist/MHGU-Skill-Simulator"

# Installs to MODS_DIR (Recomp) and RYUJINX_MODS_DIR (Ryujinx) from tools/local.sh. NO_INSTALL=1 to skip.
install_to() {
  local dir="$1"
  [ -n "$dir" ] && [ -d "$dir" ] || return 0
  rm -rf "$dir/MHGU-Skill-Simulator"
  cp -r "$ROOT/dist/MHGU-Skill-Simulator" "$dir/"
  local a b
  a=$(sha256sum < "$OUT/skill_simulator.nro" | cut -c1-16)
  b=$(sha256sum < "$dir/MHGU-Skill-Simulator/romfs/nativeNX/plugins/skill_simulator.nro" | cut -c1-16)  # With backslashes in the path, sha256sum prefixes its output with a backslash, so read it from stdin
  [ "$a" = "$b" ] || { echo "설치본 해시가 다르다: $dir" >&2; exit 1; }
  echo "설치: $dir/MHGU-Skill-Simulator (nro $a)"
}
if [ -z "${NO_INSTALL:-}" ]; then
  install_to "${MODS_DIR:-}"
  install_to "${RYUJINX_MODS_DIR:-}"
fi
