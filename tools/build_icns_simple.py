import os, struct

ROOT = os.path.join(os.path.dirname(__file__), '..')
ICONS_DIR = os.path.join(ROOT, 'assets', 'icons')
PNG_MAP = {
    'tray-16.png': b'icp4',
    'tray-32.png': b'icp5',
    'tray-48.png': b'icp6',
}
OUT = os.path.join(ICONS_DIR, 'tray.icns')

chunks = []
for name, tag in PNG_MAP.items():
    path = os.path.join(ICONS_DIR, name)
    if os.path.exists(path):
        with open(path, 'rb') as f:
            data = f.read()
        # Each chunk: 4-byte type + 4-byte BE length + data
        length = 8 + len(data)
        chunks.append((tag, length, data))

if not chunks:
    print('No PNGs found to build icns')
else:
    # total size = 8 + sum(lengths)
    total = 8 + sum(c[1] for c in chunks)
    with open(OUT, 'wb') as out:
        out.write(b'icns')
        out.write(struct.pack('>I', total))
        for tag, length, data in chunks:
            out.write(tag)
            out.write(struct.pack('>I', length))
            out.write(data)
    print('Wrote', OUT)
