import os
from icnsutil import icns

ROOT = os.path.join(os.path.dirname(__file__), '..')
ICONS_DIR = os.path.join(ROOT, 'assets', 'icons')
PNG_SIZES = [16,32,48]
PNG_FILES = [os.path.join(ICONS_DIR, f'tray-{s}.png') for s in PNG_SIZES]
OUT_ICNS = os.path.join(ICONS_DIR, 'tray.icns')

# Build icns from available PNGs
iconset = {}
for s, p in zip(PNG_SIZES, PNG_FILES):
    if os.path.exists(p):
        iconset[s] = p

if not iconset:
    print('No PNG sources found for icns generation')
else:
    builder = icns.IconBuilder()
    for size, path in iconset.items():
        builder.add_png(path, size)
    with open(OUT_ICNS, 'wb') as f:
        f.write(builder.build())
    print('Wrote', OUT_ICNS)
