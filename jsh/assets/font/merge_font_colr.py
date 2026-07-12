#!/usr/bin/env python3
"""Merges the 3 custom OS-logo COLRv1 glyphs (built by nanoemoji, format
glyf_colr_1) into a JetBrains Mono weight. COLRv1 is used instead of the
raw `SVG ` table because Chrome/Skia only paints OT-SVG glyphs reliably
when the font is recognized as a "real" emoji font; COLRv1 (vector
layers + COLR/CPAL) is universally supported and renders correctly as a
plain merged glyph in any font."""
import sys
from fontTools.ttLib import TTFont

BASE = sys.argv[1]
EMOJI = sys.argv[2]
OUT = sys.argv[3]
NAME_SUFFIX = sys.argv[4] if len(sys.argv) > 4 else "jsh"

base = TTFont(BASE)
emoji = TTFont(EMOJI)

emoji_cmap = emoji.getBestCmap()  # codepoint -> glyph name, in emoji font
emoji_glyph_order = emoji.getGlyphOrder()
emoji_glyf = emoji["glyf"]
emoji_hmtx = emoji["hmtx"]

# The emoji font's glyph set includes the 3 top-level color glyphs (named
# after their codepoint, e.g. "u1F427") plus every layer glyph referenced
# by its COLR table (the actual outlines). We need to bring *all* of them
# across, renamed to avoid clashes with the base font's glyph names.
base_glyph_order = base.getGlyphOrder()
base_glyf = base["glyf"]
base_hmtx = base["hmtx"]
existing = set(base_glyph_order)


def renamed(name: str) -> str:
    new = f"{name}.{NAME_SUFFIX}"
    return new


rename_map = {}
for name in emoji_glyph_order:
    if name in (".notdef", "space"):
        continue
    rename_map[name] = renamed(name)

# Reference width: make the logo occupy a full monospace cell.
cell_width = base_hmtx["A"][0] if "A" in base_hmtx.metrics else 1200

for old_name, new_name in rename_map.items():
    base_glyph_order.append(new_name)
    base_glyf[new_name] = emoji_glyf[old_name]
    base_hmtx[new_name] = (cell_width, emoji_hmtx[old_name][1])

base.setGlyphOrder(base_glyph_order)
base_glyf.glyphOrder = base_glyph_order

# Update cmap: point the 3 top-level codepoints at the renamed glyphs.
# COLRv1 glyphs (the 3 emoji) are all outside the BMP, so only cmap
# subtables that support >0xFFFF codepoints (format != 4) can carry them.
for table in base["cmap"].tables:
    if table.format == 4:
        continue
    for cp, name in emoji_cmap.items():
        if name not in rename_map:
            continue
        table.cmap[cp] = rename_map[name]

# Merge COLR/CPAL: remap every glyph name referenced by the emoji font's
# COLR table to its renamed counterpart, then attach as-is (JetBrains Mono
# has no COLR table of its own, so there's nothing to union with).
emoji_colr = emoji["COLR"]
emoji_cpal = emoji["CPAL"]


def remap_value(value):
    """Recursively rewrites glyph-name strings found anywhere in a COLR
    Paint record tree (paints can nest arbitrarily for gradients/composites)."""
    if isinstance(value, str):
        return rename_map.get(value, value)
    if isinstance(value, list):
        return [remap_value(v) for v in value]
    if isinstance(value, dict):
        return {k: remap_value(v) for k, v in value.items()}
    if hasattr(value, "__dict__"):
        for attr, v in vars(value).items():
            setattr(value, attr, remap_value(v))
        return value
    return value


for rec in emoji_colr.table.BaseGlyphList.BaseGlyphPaintRecord:
    rec.BaseGlyph = rename_map.get(rec.BaseGlyph, rec.BaseGlyph)
    remap_value(rec.Paint)

# PaintColrLayers (Format 1) doesn't hold glyph names directly — it
# indexes into a separate top-level LayerList of Paint records, which
# must be walked and remapped too, or nested glyph refs stay pointed at
# the emoji font's original (now-orphaned) glyph names.
if emoji_colr.table.LayerList is not None:
    for paint in emoji_colr.table.LayerList.Paint:
        remap_value(paint)

base["COLR"] = emoji_colr
base["CPAL"] = emoji_cpal

# Rename the family so it's clearly a distinct font, not vanilla JetBrains Mono.
for rec in base["name"].names:
    text = rec.toUnicode()
    if rec.nameID in (1, 4, 6, 16):
        rec.string = text.replace("JetBrainsMono", "JSHMono").replace(
            "JetBrains Mono", "JSH Mono"
        )

base.save(OUT)
print(f"Wrote {OUT}; added glyphs: {list(rename_map.values())}")
