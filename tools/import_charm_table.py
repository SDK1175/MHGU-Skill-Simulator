"""Copies the charm generation tables from the apmnnn/mhxx-rng notebook into tools/charm_table.json.

  python tools/import_charm_table.py <path to mhxx-rng.ipynb>

The notebook code is not executed; only the assignment statements holding the tables are read as text.
Source: https://github.com/apmnnn/mhxx-rng (MIT License)
"""
import ast
import json
import re
import sys
from pathlib import Path

OUT = Path(__file__).with_name('charm_table.json')
# Notebook function name -> name of the unappraised charm
KINDS = [('set_blue', '風化したお守り'), ('set_red', '古びたお守り'), ('set_yellow', '光るお守り'), ('set_white', 'なぞのお守り')]


def tree_id(index):
    """Notebook skill number -> game skill tree id."""
    return {202: 1, 203: 204, 204: 205}.get(index, index + 2)


def main():
    nb = json.loads(Path(sys.argv[1]).read_text(encoding='utf-8'))
    source = next(s for s in (''.join(c['source']) for c in nb['cells']) if 'def set_blue' in s)
    kinds = []
    for func, label in KINDS:
        body = source[source.index(f'def {func}'):]
        nxt = body.find('\ndef ', 1)
        body = body if nxt < 0 else body[:nxt]

        def value(name):
            m = re.search(rf'^\s*{name}\s*=\s*(\[.*?\]|\d+)\s*$', body, re.S | re.M)
            return ast.literal_eval(m.group(1))

        skill1, sp1, skill2, sp2 = value('skill1'), value('sp1'), value('skill2'), value('sp2')
        slots = value('slotvalue')
        assert len(skill1) == len(sp1) and len(skill2) == len(sp2) and len(slots) == 20, func
        kinds.append({
            'name': label,
            # Threshold deciding whether a second skill is added: added when the roll (0-99) is at or above it.
            'second_threshold': value('th'),
            # First skill: [tree, min, max]
            'first': [[tree_id(s), lo, hi] for s, (lo, hi) in zip(skill1, sp1)],
            # Second skill: [tree, max magnitude when negative, max when positive]
            'second': [[tree_id(s), neg, hi] for s, (neg, hi) in zip(skill2, sp2)],
            # Slot roll thresholds per fill value 1-20: 0, 1, 2 slots when the roll is below a threshold, otherwise 3 slots
            'slot_table': [list(t) for t in slots],
        })
    OUT.write_text(json.dumps({
        'source': 'https://github.com/apmnnn/mhxx-rng',
        'license': 'MIT',
        'kinds': kinds,
    }, ensure_ascii=False, indent=1), encoding='utf-8')
    for k in kinds:
        print(k['name'], 'first', len(k['first']), 'second', len(k['second']), 'threshold', k['second_threshold'])


if __name__ == '__main__':
    main()
