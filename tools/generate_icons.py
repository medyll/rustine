from PIL import Image, ImageDraw
import os

OUT_DIR = os.path.join(os.path.dirname(__file__), '..', 'assets', 'icons')
os.makedirs(OUT_DIR, exist_ok=True)

# Define colors
bg = (14,165,164,255) # teal
fg = (255,255,255,255) # white

# Create SVG-like icon: rounded square bg and inner circle+bar
for size in [16,32,48]:
    img = Image.new('RGBA', (size, size), (0,0,0,0))
    draw = ImageDraw.Draw(img)
    # rounded rect background
    r = size//6
    draw.rounded_rectangle([0,0,size-1,size-1], radius=r, fill=bg)
    # inner circle
    cx = size//2
    cy = size//3
    cr = size//6
    draw.ellipse([cx-cr, cy-cr, cx+cr, cy+cr], fill=fg)
    # bar
    bar_h = max(2, size//12)
    bar_w = size - size//4
    bx0 = (size - bar_w)//2
    by0 = size - size//4 - bar_h//2
    draw.rounded_rectangle([bx0, by0, bx0+bar_w, by0+bar_h], radius=bar_h, fill=fg)
    out_path = os.path.join(OUT_DIR, f'tray-{size}.png')
    img.save(out_path)
    print('Wrote', out_path)

# Create multi-size ICO
ico_path = os.path.join(os.path.dirname(__file__), '..', 'assets', 'icon.ico')
ico_path = os.path.abspath(ico_path)
imgs = []
for size in [16,32,48]:
    imgs.append(Image.open(os.path.join(OUT_DIR, f'tray-{size}.png')))
imgs[0].save(ico_path, format='ICO', sizes=[(16,16),(32,32),(48,48)])
print('Wrote', ico_path)
