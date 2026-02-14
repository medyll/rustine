Icons:

- `assets/icons/tray.svg` : source vector for tray icon
- Derive PNG sizes: `tray-16.png`, `tray-32.png`, `tray-48.png` — place them in `assets/icons/`
- `assets/icon.ico` may be used as fallback on Windows

Regeneration:

- Use `inkscape` or `rsvg-convert` / ImageMagick to export PNGs and create `.ico` / `.icns` files from `tray.svg`.

Example commands:

```bash
inkscape -w 48 -h 48 assets/icons/tray.svg -o assets/icons/tray-48.png
inkscape -w 32 -h 32 assets/icons/tray.svg -o assets/icons/tray-32.png
inkscape -w 16 -h 16 assets/icons/tray.svg -o assets/icons/tray-16.png
# create multi-size .ico using ImageMagick
convert assets/icons/tray-16.png assets/icons/tray-32.png assets/icons/tray-48.png assets/icon.ico
```

Notes:

- The runtime will try `assets/icons/tray-48.png` → `tray-32.png` → `tray-16.png` → `assets/icon.ico`.
- If none are found, the tray falls back to no custom icon and logs a message.
