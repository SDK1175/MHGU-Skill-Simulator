<p align="center">
  <strong>English</strong> | <a href="README_ko.md">한국어</a>
</p>

# MHGU Skill Simulator

A Forge mod that adds an in-game armor skill simulator to Monster Hunter Generations Ultimate. 

Pick the skills you want, and it searches every armor set, charm and decoration combination that activates them — using the armor, charms and decorations you actually own. 

If your charms cannot make the set, it shows the charm you would need and estimates how likely one appraisal is to give it.

## Requirements

- MHGU global version 1.4.0
- MHGU Recomp with Forge plugin support, or an emulator with Forge 1.5.1 installed

## Features

- Searches with up to 8 target skills, reading your equipment box, charms and current equipment from the game
- Searches with unowned charms when your own charms cannot make the set, with the estimated chance per appraisal
- Results show recommended sets, free slots, defense, resistances, decoration placement, active skills and the parts you still need to craft, with crafting materials and cost
- Alternative armor for each part that keeps the same skills, comparison with your current equipment, and the extra skills a set can still activate
- When nothing matches, finds which single condition to change to get a result
- Nine languages: English, 简体中文, 繁體中文, 日本語, 한국어, Deutsch, Français, Italiano, Español
- Nintendo, Xbox and PlayStation button icons; mouse and keyboard on Recomp
- Works with Better Skillinfo (skill details for the selected item) and Wishlist (register sets and parts) when they are installed

## Installation

1. Close the game.
2. Extract `MHGU-Skill-Simulator_vX.X.X.zip`.
3. Place the extracted `MHGU-Skill-Simulator` folder in the `mods` folder next to the Recomp executable.
4. Launch the game. If the mod has been disabled in the mod manager, enable it.

```text
mods/
└── MHGU-Skill-Simulator/
    └── romfs/
        └── nativeNX/
            └── plugins/
                ├── skill_simulator.nro
                └── skill-simulator/
                    ├── NOTO_LICENSE.txt
                    ├── atlas.rgba
                    ├── font.otf
                    └── names.en.bin … names.es.bin
```

## Usage

### Opening the simulator

Load a character, open the in-game menu with the + button and go to **Hunter's Notes**. Choose **Skill Simulator** at the bottom of the list.

The **Open** button in the Forge settings opens it too.

## Settings

Press Insert to open the Forge settings, then go to `Plugin UI > Skill Simulator`.

| Setting | Options |
| --- | --- |
| Language | Choose from nine supported languages (English by default) |
| Button layout | Nintendo / Xbox / PlayStation |
| Enable logging | Writes search conditions and frame timing to `log.txt` for bug reports (off by default) |

## Save Data

Settings and per-character data are stored in separate INI files within the game's current save directory.

| File | Contents |
| --- | --- |
| `skill_simulator.ini` | Language, button layout and logging |
| `skill_simulator_character_1.ini` | Last conditions, saved sets, excluded armor and recent skills of the first character slot |
| `skill_simulator_character_2.ini` | The same for the second character slot |
| `skill_simulator_character_3.ini` | The same for the third character slot |

The default Windows Recomp path is `%APPDATA%\MHGURecomp\save\save_0\`. If your environment specifies a custom save path, that path is used instead. These files are not replaced when updating the mod.

## Notes

- Charm chances are estimates from the charm generation tables, not exact game odds.
- Settings and saved data are per save folder; the window opens only after a character is loaded.

## Building from Source

Required tools: Rust nightly with `rust-src`, the Forge Cargo build tool (`cargo forge`), devkitARM, Python 3.10, and Git Bash.

1. Copy `tools/local.example.sh` to `tools/local.sh` and set your devkitARM path (and, optionally, your game mods folders to install to).
2. Run the build:

```bash
bash tools/build_plugin.sh
```

The build produces `plugin/skill_simulator.nro` and the install folder `dist/MHGU-Skill-Simulator`. The generated tables in `plugin/src/generated` and the files in `plugin/assets` are committed, so the build does not need the game data. They are regenerated only when the game database (`../Database/mhxx_all_tables.json`) is next to the repository; that also needs the Python packages Pillow and fontTools.

Tests:

```bash
cd engine && cargo test --release                 # engine, including brute-force comparison
cd plugin-hosttest && cargo run --release         # plugin logic on the PC
```

## Repository Structure

| Folder | Contents |
| --- | --- |
| `engine` | Search engine (`no_std` Rust), shared by the plugin and the tests |
| `plugin` | The Forge plugin: screens, input, game data reading and integrations |
| `plugin-hosttest` | Runs the plugin's logic on the PC |
| `data` | Simulator data (`simdata.bin`) built from the game database |
| `assets/ui` | Pictures packed into the plugin's icon atlas |
| `tools` | Build scripts, data generators and the screen strings in all languages (`tools/strings.py`) |
| `vendor/forge-rs` | Vendored Forge SDK |

## License

MIT License. See `LICENSE`, and `THIRD_PARTY_NOTICES.md` for third-party components and game data.
