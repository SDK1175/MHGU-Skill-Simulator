"""Builds the table from game equipment box armor (part, series number) -> simulator DB armor id (Amr_ID).

The game stores armor as series number + part (equipment box +6, save editor docs/07). The DB has a separate number per armor.
Compares the game table armorSeriesData (resident.arc, 127 B per series: 6-10 parts present, 13/14 male/female, 15/16 blademaster/gunner,
from 28, 15 B per part = 5 x (u16 tree, i8 points), slots at 108+part-1) with the DB.
 - Tree number: game = DB - 1 (confirmed with Leather Head: game 134, DB 135)
 - DB BorG 0 (shared by blademaster and gunner) may be split into a blademaster and a gunner series in the game -> several map to one DB armor
 - If several share the same features (part, skills, slots, blademaster/gunner, male/female), narrow down by crafting materials (recipes.csv); if still several, count as a failure
Output: plugin/src/generated/armor_map.rs  (ARMOR_MAP: [(part 1-5, series, Amr_ID)] sorted)
"""
import csv
import json
import struct
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DB = ROOT / 'Database' / 'mhxx_all_tables.json'
SAVE = ROOT / 'tools' / 'MHGU-Save-editor'
RESIDENT = ROOT / 'UnpakingGU' / 'backup' / 'romfs' / 'nativeNX' / 'loc' / 'arc' / 'resident.arc'
OUT = Path(__file__).resolve().parents[1] / 'plugin' / 'src' / 'generated' / 'armor_map.rs'


def game_tree(t):
    """Game tree number -> DB tree number - 1. Mostly game = DB - 1 (DB tree 1, torso up, does not exist in the game),
    but secret skill and talisman boost are game 204/205 = DB 204/205 (confirmed with Nesette Solar)"""
    return t - 1 if t in (204, 205) else t


def both(a, b):
    """(allowed1, allowed2) flags -> DB notation 0 both, 1 first only, 2 second only"""
    return 0 if a and b else 1 if a else 2


def main():
    sys.path.insert(0, str(SAVE / 'tools'))
    from build_assets import arc
    table = arc(RESIDENT.read_bytes())['table' + chr(92) + 'armorSeriesData']
    count = struct.unpack_from('<I', table, 4)[0]
    assert len(table) == 8 + count * 127
    db = json.loads(DB.read_text(encoding='utf-8'))
    game_item = {r['Itm_ID']: int(r['Hex'], 16) for r in db['DB_Itm']}
    db_skills = defaultdict(list)
    for r in db['DB_SklTreetoAmr']:
        db_skills[r['Amr_ID']].append((r['SklTree_ID'] - 1, r['Pt']))
    db_mats = defaultdict(list)
    for r in db['DB_ItmtoAmr']:
        db_mats[r['Amr_ID']].append((game_item.get(r['Itm_ID'], -1), r['Qty']))
    g_mats = {}
    for r in csv.DictReader(open(SAVE / 'data' / 'recipes.csv', encoding='utf-8')):
        if r['kind'].startswith('armor:') and r['level'] == '0':
            g_mats[(int(r['kind'][6:]), int(r['id']))] = tuple(sorted((int(r[f'item{i}']), int(r[f'count{i}'])) for i in range(1, 5) if int(r[f'item{i}'])))

    # DB armor features -> candidates
    cands = defaultdict(list)
    for r in db['DB_Amr']:
        a = r['Amr_ID']
        # The DB writes torso up as tree 1 (game 0) points, but the game table has none
        key = (r['Part'], r['Slot'], tuple(sorted((t, p) for t, p in db_skills[a] if p and t != 0)))
        cands[key].append(r)

    pairs, missing, ambiguous = [], [], []
    for k in range(1, count):  # Index 0 is an empty series
        rec = table[8 + 127 * k:8 + 127 * (k + 1)]
        borg = both(rec[15], rec[16])
        morf = both(rec[13], rec[14])
        for part in range(1, 6):
            if not rec[5 + part]:
                continue
            skills = []
            for j in range(5):
                tree, pts = struct.unpack_from('<Hb', rec, 28 + 15 * (part - 1) + 3 * j)
                if tree and pts:
                    skills.append((game_tree(tree), pts))
            key = (part, rec[107 + part], tuple(sorted(skills)))
            # Blademaster/gunner and male/female: the DB's shared (0) matches either
            fit = [r for r in cands.get(key, []) if r['BorG'] in (0, borg) and r['MorF'] in (0, morf)]
            if len(fit) > 1 and (part, k) in g_mats:
                narrowed = [r for r in fit if tuple(sorted(db_mats[r['Amr_ID']])) == g_mats[(part, k)]]
                if narrowed:
                    fit = narrowed
            if len(fit) > 1:
                exact = [r for r in fit if r['BorG'] == borg and r['MorF'] == morf]
                if len(exact) == 1:
                    fit = exact
            if not fit:
                missing.append((part, k, skills))
            elif len(fit) > 1:
                ambiguous.append((part, k, [r['Amr_ID'] for r in fit]))
            else:
                pairs.append((part, k, fit[0]['Amr_ID']))
    # Ambiguous ones (e.g. the DB writes both the blademaster and gunner heads as shared): pick the candidate closest to the DB numbers
    # matched by the other parts of the same game series (the DB lists a set's parts together). Only when exactly one is closest.
    near = defaultdict(list)
    for part, k, a in pairs:
        near[k].append(a)
    still = []
    for part, k, ids in ambiguous:
        if not near[k]:
            still.append((part, k, ids))
            continue
        dist = sorted((min(abs(a - b) for b in near[k]), a) for a in ids)
        if dist[0][0] < dist[1][0] and dist[0][0] <= 10:
            pairs.append((part, k, dist[0][1]))
        else:
            still.append((part, k, ids))
    # Head-only series (Skull Head/Face etc.): a blademaster and a gunner series sit next to each other and the DB also lists two together.
    # The blademaster one (earlier series) gets the earlier number. Both candidates have identical stats, so search results are unaffected.
    rest = []
    for part, k, ids in ambiguous:
        rec = table[8 + 127 * k:8 + 127 * (k + 1)]
        if len(ids) == 2 and rec[15] != rec[16]:
            pairs.append((part, k, min(ids) if rec[15] else max(ids)))
        else:
            rest.append((part, k, ids))
    ambiguous = rest
    covered = {a for _, _, a in pairs}
    names = {r['Amr_ID']: r.get('Amr_Name_4') for r in db['ID_Amr_Name']}
    uncovered = [r['Amr_ID'] for r in db['DB_Amr'] if r['Amr_ID'] not in covered]
    print(f'게임 시리즈 {count}, 짝 {len(pairs)}, 게임 쪽 후보 없음 {len(missing)}, 모호 {len(ambiguous)}, '
          f'짝 없는 DB 방어구 {len(uncovered)} / {len(db["DB_Amr"])}')
    if ambiguous:
        print('모호 예:', [(p, s, [(a, names.get(a)) for a in v]) for p, s, v in ambiguous[:8]])
    if uncovered:
        print('짝 없는 DB 예:', [(a, names.get(a)) for a in uncovered[:12]])
    if '--missing' in sys.argv:
        print('후보 없는 게임 방어구:', missing[:40])
    # Verification: pairs with crafting materials must have the same materials too (possible DB typos are judged by count)
    with_recipe = [(p, k, a) for p, k, a in pairs if (p, k) in g_mats]
    differ = [(p, k, a, names.get(a)) for p, k, a in with_recipe if tuple(sorted(db_mats[a])) != g_mats[(p, k)]]
    print(f'소재 대조: 생산법 있는 짝 {len(with_recipe)}, 소재 다름 {len(differ)}', differ[:6])
    pairs.sort()
    OUT.parent.mkdir(parents=True, exist_ok=True)
    lines = ['//! Generated by tools/build_armor_map.py. Do not edit.',
             '/// (part 1-5, game armor series number, DB armor id), sorted by part and series',
             f'pub static ARMOR_MAP: [(u8, u16, u16); {len(pairs)}] = [']
    for i in range(0, len(pairs), 8):
        lines.append('    ' + ' '.join(f'({p}, {s}, {a}),' for p, s, a in pairs[i:i + 8]))
    lines.append('];')
    OUT.write_text('\n'.join(lines) + '\n', encoding='utf-8')
    # 3964 'no equipment X' is a DB entry for empty slots
    return 0 if not ambiguous and set(uncovered) <= {3964} else 1


if __name__ == '__main__':
    sys.exit(main())
