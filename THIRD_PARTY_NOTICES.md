# Third-party notices

This project is licensed under the MIT License (see `LICENSE`). It also contains or is built from the following.

## forge-rs

`vendor/forge-rs/` is a vendored copy of forge-rs (see `vendor/forge-rs/VENDORED.md`).
MIT License, Copyright (c) 2026 Fexty. The full text is in `vendor/forge-rs/LICENSE`.

## Noto Sans CJK

`plugin/assets/font.otf` is a subset of Noto Sans CJK KR, licensed under the SIL Open Font License 1.1.
The full text is in `plugin/assets/NOTO_LICENSE.txt` and is shipped next to the font.

## mhxx-rng (charm generation tables)

`tools/charm_table.json` holds the charm generation tables copied from https://github.com/apmnnn/mhxx-rng
(by `tools/import_charm_table.py`).

```
MIT License

Copyright (c) 2026 apmnnn

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Kenney Input Prompts

The keyboard, mouse and controller button pictures in `assets/ui/` (`kbm-*.png`, `Nintendo-*.png`, `Xbox-*.png`,
`PlayStation-*.png`) are from Kenney Input Prompts (https://kenney.nl), released under CC0.

## Game data and game art

Monster Hunter Generations Ultimate is © CAPCOM. Equipment, skill, item and quest names and the numbers in `data/` and
`plugin/src/generated/` come from the game and remain the property of their owner.

The part, weapon, charm, decoration and item icons in `assets/ui/` are the game's UI art as found in the "HD Layout" UI
pack (cut from the atlas MHGU-Wishlist extracted from it, see `assets/ui/source.json`). The author and redistribution
terms of that pack have not been confirmed yet.

These are included only so the mod can show the game's own names and icons; no license is granted for them here, and
this project is not affiliated with or endorsed by CAPCOM.
