#!/usr/bin/env bash
# Build for MHXX Recomp without devkitARM (MSYS2 + Rust nightly only), and put the plugin into the port's mods folder.
#   bash tools/build_xx.sh [mods folder of MHXX Recomp]
# Links with rust-lld (plugin/armv7a-none-eabi-lld.json) instead of arm-none-eabi-gcc, and makes the NRO with
# tools/elf_to_nro.py instead of cargo-forge. The assets (font, atlas, names) are the committed ones.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MODS="${1:-/f/MHXX_port/MHXXRecomp/build/mods}"
cd "$ROOT/plugin"
# (no folder of this PC in the plugin: its panic messages carry source paths)
remap() {
  local from="$1" to="$2"
  RUSTFLAGS="$RUSTFLAGS --remap-path-prefix=$from=$to"
  if command -v cygpath >/dev/null 2>&1; then
    RUSTFLAGS="$RUSTFLAGS --remap-path-prefix=$(cygpath -w "$from")=$to --remap-path-prefix=$(cygpath -m "$from")=$to"
  fi
}
RUSTFLAGS="-C link-arg=-Tlink/switch32.ld -C link-arg=--build-id=sha1 -C target-feature=-dsp"
remap "$ROOT" /skillsim
remap "${CARGO_HOME:-$HOME/.cargo}" /cargo
remap "${RUSTUP_HOME:-$HOME/.rustup}" /rustup
if [[ -n "${USERPROFILE:-}" ]] && command -v cygpath >/dev/null 2>&1; then
  remap "$(cygpath -u "$USERPROFILE")/.cargo" /cargo
  remap "$(cygpath -u "$USERPROFILE")/.rustup" /rustup
fi
export RUSTFLAGS
cargo +nightly build --release --target armv7a-none-eabi-lld.json
mkdir -p "$ROOT/dist"
python3 "$ROOT/tools/elf_to_nro.py" target/armv7a-none-eabi-lld/release/libskill_simulator.so "$ROOT/dist/skill_simulator.nro"
ls -la "$ROOT/dist/skill_simulator.nro"
# The release zip (dist/), as the original's layout: refused if the plugin holds a folder of this PC
VERSION="$(grep -m1 '^version' "$ROOT/plugin/Cargo.toml" | cut -d'"' -f2)-mhxx"
python3 - "$ROOT" "$VERSION" <<'PY'
import os, sys, zipfile
from pathlib import Path
root, version = Path(sys.argv[1]), sys.argv[2]
nro = (root / 'dist' / 'skill_simulator.nro').read_bytes()
marks = [b'\\Users\\', b'/Users/', b'/home/', b'C:/Users', str(root).encode(), root.as_posix().encode()]
marks += [os.environ[v].encode() for v in ('USERNAME', 'USER') if os.environ.get(v)]
if any(len(m) > 2 and m in nro for m in marks):
    sys.exit('STOP: the plugin holds paths of this PC: not packaged')
out = root / 'dist' / f'MHGU-Skill-Simulator_v{version}.zip'
base = 'MHGU-Skill-Simulator/romfs/nativeNX/plugins/'
with zipfile.ZipFile(out, 'w', zipfile.ZIP_DEFLATED) as z:
    z.writestr(base + 'skill_simulator.nro', nro)
    for f in sorted((root / 'plugin' / 'assets').iterdir()):
        z.write(f, base + 'skill-simulator/' + f.name)
    z.write(root / 'LICENSE', base + 'skill-simulator/LICENSE.txt')
    z.write(root / 'THIRD_PARTY_NOTICES.md', base + 'skill-simulator/THIRD_PARTY_NOTICES.md')
print(out, out.stat().st_size, 'bytes')
PY
DEST="$MODS/MHGU-Skill-Simulator/romfs/nativeNX/plugins"
if [ -d "$MODS" ]; then
  mkdir -p "$DEST/skill-simulator"
  cp "$ROOT/dist/skill_simulator.nro" "$DEST/"
  for f in "$ROOT"/plugin/assets/*; do cp "$f" "$DEST/skill-simulator/"; done
  cp "$ROOT/LICENSE" "$DEST/skill-simulator/LICENSE.txt"
  cp "$ROOT/THIRD_PARTY_NOTICES.md" "$DEST/skill-simulator/"
  echo "copied into $DEST"
else
  echo "no mods folder at $MODS: the plugin is in dist/"
fi
