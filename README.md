<p align="center">
  <strong>English</strong> | <a href="README_ko.md">한국어</a>
</p>

# MHGU Skill Simulator (fork for MHXX Recomp)

An unofficial fork of [ZellD-0371's MHGU Skill Simulator](https://github.com/ZellD-0371/MHGU-Skill-Simulator) (MIT) for
**[MHXX Recomp](https://github.com/SDK1175/MHXXRecomp)**, a PC port of Monster Hunter XX (Switch, Japanese) that runs
MHGU 1.4.0 Forge plugins. The simulator and its search engine are the original's.

- For MHGU (the English game) or MHGU Recomp, use the [original](https://github.com/ZellD-0371/MHGU-Skill-Simulator/releases).
- Report problems with this fork [here](https://github.com/SDK1175/mhgu-skill-simulator/issues), not to the original's author.

## Needs

- MHXX Recomp 0.10.9 or later (plugin memory, mouse and key input, the Korean patch's text, GU address translation).

## What differs from the original

- The village / gathering hall menu of MHXX is recognized: "Skill Simulator" is on page 3 of the + menu.
- With the MHXX Korean patch, the menu entry's name and description are in the patch's codes (shown in Korean).
- A character loaded in MHXX counts as playing (the window opened only in MHGU).
- `tools/build_xx.sh` builds without devkitARM.

On MHGU and emulators it behaves as the original.

## Install

Unzip `MHGU-Skill-Simulator_vX.X.X-mhxx.zip` and put the `MHGU-Skill-Simulator` folder into MHXX Recomp's `mods`
folder (remove the original's first). Open it from page 3 of the + menu in the village or hall, or with the Open button
in Forge's menu (Insert) > Plugin UI > Skill Simulator.

## Build

MSYS2 (UCRT64), Rust nightly with `rust-src`, Python 3:

```bash
bash tools/build_xx.sh [MHXX Recomp's mods folder]
```

The plugin and the release zip land in `dist/`.

## License

MIT (original: Copyright (c) 2026 ZellD-0371). See `LICENSE` and `THIRD_PARTY_NOTICES.md`. `tools/elf_to_nro.py` is from
[MHGU Utility](https://github.com/Rikiar73574/mhgu-utility) (MIT, Riccardo Testa). Monster Hunter is a trademark of
CAPCOM; this project is not affiliated with CAPCOM.
