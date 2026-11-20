"""Copies the button prompt pictures into assets/ui (packed into the atlas by build_plugin_data.py).

Source: Kenney Input Prompts (CC0 - www.kenney.nl), the Default (64x64) folder of each set in HD Icons/kenney Icons.
- Controller: <layout>-<button>.png for Nintendo / Xbox / PlayStation. Face buttons follow the position
  (A right, B bottom, X top, Y left), as Recomp's FaceButtons = Position.
- Keyboard / mouse: kbm-<key>.png. Names follow prompts::key_sprite in the plugin: the [Keybinds] key name in lower case,
  spaces as '-', punctuation as words ('-' -> 'minus').
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
KENNEY = ROOT.parent / 'HD Icons' / 'kenney Icons'
SOURCE = KENNEY / 'Keyboard & Mouse' / 'Default'
OUT = ROOT / 'assets' / 'ui'

# Kenney pictures leave a wide margin (visible part 48 of 64). Crop the same centered 56x56 from every picture so all keep
# their relative sizes and fill the cell like the earlier button pictures (about 86%); nothing in the set reaches past it.
CROP = (4, 4, 60, 60)


def copy(src, dst):
    from PIL import Image
    im = Image.open(src).convert('RGBA').crop(CROP).resize((64, 64), Image.Resampling.LANCZOS)
    im.save(dst)

# Controller pictures: our button name -> Kenney file stem. 'dpad' = up/down hints, 'dpadleft' = pager arrow (flipped for right).
PADS = {
    'Nintendo': ('Nintendo Switch', {
        'a': 'switch_button_a', 'b': 'switch_button_b', 'x': 'switch_button_x', 'y': 'switch_button_y',
        'l': 'switch_button_l', 'r': 'switch_button_r', 'zl': 'switch_button_zl', 'zr': 'switch_button_zr',
        'plus': 'switch_button_plus', 'minus': 'switch_button_minus',
        'dpad': 'switch_buttons', 'dpadleft': 'switch_buttons_left_outline',
        'l3': 'switch_stick_l_press', 'r3': 'switch_stick_r_press',
    }),
    'Xbox': ('Xbox Series', {
        # Only the colored face buttons are in the set
        'a': 'xbox_button_color_b', 'b': 'xbox_button_color_a', 'x': 'xbox_button_color_y', 'y': 'xbox_button_color_x',
        'l': 'xbox_lb', 'r': 'xbox_rb', 'zl': 'xbox_lt', 'zr': 'xbox_rt',
        'plus': 'xbox_button_menu', 'minus': 'xbox_button_view',
        'dpad': 'xbox_dpad', 'dpadleft': 'xbox_dpad_left_outline',
        'l3': 'xbox_ls', 'r3': 'xbox_rs',
    }),
    'PlayStation': ('PlayStation Series', {
        'a': 'playstation_button_color_circle', 'b': 'playstation_button_color_cross',
        'x': 'playstation_button_color_triangle', 'y': 'playstation_button_color_square',
        'l': 'playstation_trigger_l1_alternative', 'r': 'playstation_trigger_r1_alternative',
        'zl': 'playstation_trigger_l2_alternative', 'zr': 'playstation_trigger_r2_alternative',
        'plus': 'playstation4_button_options', 'minus': 'playstation4_button_share',
        'dpad': 'playstation_dpad', 'dpadleft': 'playstation_dpad_left_outline',
        'l3': 'playstation_button_l3', 'r3': 'playstation_button_r3',
    }),
}

# our name -> Kenney file stem
MAP = {}
for c in 'abcdefghijklmnopqrstuvwxyz0123456789':
    MAP[c] = f'keyboard_{c}'
for n in range(1, 13):
    MAP[f'f{n}'] = f'keyboard_f{n}'
MAP.update({
    'up': 'keyboard_arrow_up',
    'down': 'keyboard_arrow_down',
    'left': 'keyboard_arrow_left',
    'right': 'keyboard_arrow_right',
    'space': 'keyboard_space',
    'return': 'keyboard_enter',
    'enter': 'keyboard_enter',
    'keypad-enter': 'keyboard_numpad_enter',
    'keypad-plus': 'keyboard_numpad_plus',
    'backspace': 'keyboard_backspace',
    'tab': 'keyboard_tab',
    'escape': 'keyboard_escape',
    'left-shift': 'keyboard_shift',
    'right-shift': 'keyboard_shift',
    'left-ctrl': 'keyboard_ctrl',
    'right-ctrl': 'keyboard_ctrl',
    'left-alt': 'keyboard_alt',
    'right-alt': 'keyboard_alt',
    'capslock': 'keyboard_capslock',
    'caps-lock': 'keyboard_capslock',
    'page-up': 'keyboard_page_up',
    'page-down': 'keyboard_page_down',
    'home': 'keyboard_home',
    'end': 'keyboard_end',
    'insert': 'keyboard_insert',
    'delete': 'keyboard_delete',
    'minus': 'keyboard_minus',
    'equals': 'keyboard_equals',
    'bracket-open': 'keyboard_bracket_open',
    'bracket-close': 'keyboard_bracket_close',
    'semicolon': 'keyboard_semicolon',
    'apostrophe': 'keyboard_apostrophe',
    'comma': 'keyboard_comma',
    'period': 'keyboard_period',
    'slash': 'keyboard_slash_forward',
    'backslash': 'keyboard_slash_back',
    'grave': 'keyboard_tilde',
    'mouse-left': 'mouse_left',
    'mouse-right': 'mouse_right',
    'mouse-middle': 'mouse_scroll',
    # Side buttons and wheel notches ([Keybinds] names "Mouse 4" / "Mouse 5" / "Wheel Up" / "Wheel Down")
    'mouse-4': 'mouse_side_back',
    'mouse-5': 'mouse_side_forward',
    'wheel-up': 'mouse_scroll_up',
    'wheel-down': 'mouse_scroll_down',
})


def main():
    if not SOURCE.is_dir():
        sys.exit(f'missing {SOURCE}')
    for old in OUT.glob('kbm-*.png'):
        old.unlink()
    for name, stem in MAP.items():
        copy(SOURCE / f'{stem}.png', OUT / f'kbm-{name}.png')
    n = 0
    for layout, (folder, names) in PADS.items():
        for name, stem in names.items():
            copy(KENNEY / folder / 'Default' / f'{stem}.png', OUT / f'{layout}-{name}.png')
            n += 1
    print(f'{len(MAP)} keyboard / mouse pictures, {n} controller pictures')


if __name__ == '__main__':
    main()
