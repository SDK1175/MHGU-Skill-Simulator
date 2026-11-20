"""Cuts the part icons and button pictures from the atlas Wishlist already extracted from HD Layout.

Output: assets/ui/part-0..4.png, charm.png, weapon-7..21.png, jewel.png, chest.png
"""
import json
import re
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT.parent / 'MHGU-Wishlist'
OUT = ROOT / 'assets' / 'ui'

meta = (SOURCE / 'src/icons_generated.rs').read_text(encoding='utf-8')
rects = [tuple(map(int, r)) for r in re.findall(r'Sprite\{x:(\d+),y:(\d+),w:(\d+),h:(\d+)\}', meta)]
height = int(re.search(r'ATLAS_HEIGHT:u16=(\d+)', meta)[1])
equip = [int(v) for v in re.search(r'EQUIP:\[usize;\d+\]=\[([^\]]+)\]', meta)[1].split(',')]
atlas = Image.frombytes('RGBA', (2048, height), (SOURCE / 'assets/wishlist-atlas.rgba').read_bytes())
OUT.mkdir(parents=True, exist_ok=True)


def crop(index, name):
    x, y, w, h = rects[index]
    atlas.crop((x, y, x + w, y + h)).save(OUT / name)


# Wishlist equipment kinds 1-5 = head, torso, arms, waist, legs. 6 = charm, 25 = decoration.
for part in range(5):
    crop(equip[part + 1], f'part-{part}.png')
crop(equip[6], 'charm.png')
# Weapon kinds 7-21 (the equipped weapon in the decoration placement)
for kind in range(7, 22):
    crop(equip[kind], f'weapon-{kind}.png')
crop(14, 'jewel.png')
# Item box (owned quantity) icon: Wishlist's CHEST picture (from the game item sheet)
crop(int(re.search(r'CHEST:usize=(\d+)', meta)[1]), 'chest.png')
# Button pictures (<layout>-<button>.png, kbm-<key>.png) come from Kenney Input Prompts: tools/import_kenney_icons.py
# Material icons: Wishlist's ITEMS table (item id -> icon index, color index, rarity)
items = [tuple(int(v) for v in t) for t in re.findall(r'\((\d+),(\d+),(\d+),\d+\)',
                                                      meta[meta.index('pub static ITEMS'):])]
palette = [int(v) for v in re.search(r'PALETTE:\[u32;\d+\]=\[([^\]]+)\]', meta)[1].split(',')]
for sprite in sorted({t[0] for t in items}):
    crop(sprite, f'item-{sprite}.png')
rgb = lambda v: f'#{v & 255:02x}{v >> 8 & 255:02x}{v >> 16 & 255:02x}'   # AABBGGRR -> #rrggbb
(OUT / 'items.json').write_text(json.dumps([[sprite, rgb(palette[color]), rare] for sprite, color, rare in items]),
                                encoding='utf-8')
(OUT / 'source.json').write_text(json.dumps({
    'source': 'MHGU-Wishlist/assets/wishlist-atlas.rgba (HD Layout 원본)',
    'sprites': 'MHGU-Wishlist/src/icons_generated.rs',
}, ensure_ascii=False, indent=1), encoding='utf-8')
print(f'{len(list(OUT.glob("*.png")))} images exported')
