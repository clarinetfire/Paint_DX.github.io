# Windows XP Paint (mspaint 5.1) Specification Baseline

Reference documentation for the classic behavior to reproduce in Paint DX.

## 1. Window and Layout
- Single document window: no tabs, no MDI.
- Menu bar: File, Edit, View, Image, Colors, Help.
- Tool box on the left with 16 tools and an options area below it.
- Color box at the bottom: 28 palette swatches plus foreground/background indicator.
- Status bar: cursor coordinates, selection size, contextual help.
- Canvas with three resize handles (right edge, bottom edge, bottom-right corner) and scroll bars.
- Thumbnail window available when zoomed in.

## 2. The 16 Tools

| Tool | Behaviour in XP |
| --- | --- |
| Free-form select | Lasso selection of an irregular area |
| Select | Rectangular selection |
| Eraser / Color eraser | Eraser in 4 sizes paints the background color; right-drag replaces only the foreground color with the background color |
| Fill with color | Flood fill of a contiguous area of one exact color, no tolerance |
| Pick color | Eyedropper; left click sets foreground, right click sets background |
| Magnifier | Presets 1x, 2x, 4x, 6x, 8x, or zoom to a chosen area |
| Pencil | 1-pixel freehand line, no anti-aliasing |
| Brush | 12 tips (round, square, forward and backward slash), 3 sizes each |
| Airbrush | Spray paint in 3 sizes |
| Text | Text box with font, size, bold, italic, underline; opaque or transparent background; font toolbar can be hidden |
| Line | Straight line in 5 widths |
| Curve | Bezier curve with 1 or 2 control points, 5 widths |
| Rectangle | Outline, outline + fill, or fill only |
| Polygon | Multi-sided shape, same 3 fill styles |
| Ellipse | Same 3 fill styles |
| Rounded rectangle | Same 3 fill styles |

Left button draws with the foreground color, right button with the background color. Shift constrains lines to 45 degrees, rectangles to squares and ellipses to circles.

## 3. Selection Behaviour
- Background mode: opaque, or transparent (the background color becomes see-through while moving).
- Drag to move; Ctrl + drag copies; Shift + drag leaves a trail (stamp effect).
- Resize through the handles.
- Stretch and skew through the Image menu.

## 4. File Menu
- New, Open, Save, Save As.
- From scanner or camera.
- Print Preview, Page Setup, Print.
- Send (by e-mail).
- Set as Wallpaper: tiled or centered.
- Recent files list, Exit.

## 5. File Formats
- BMP: monochrome, 16 colors, 256 colors, 24-bit.
- JPEG.
- GIF (no animation, no transparency on save).
- TIFF.
- PNG (saved without alpha channel, so no transparency).

## 6. Edit Menu
- Undo and Redo, limited to 3 levels in XP (extended in Paint DX).
- Repeat (F4).
- Cut, Copy, Paste.
- Clear Selection (Del), Select All.
- Copy To file, Paste From file.

## 7. View Menu
- Show or hide: tool box, color box, status bar, text toolbar.
- Zoom: Normal (100%), Large (400%), Custom (100, 200, 400, 600, 800%).
- Show Grid (from 400% zoom).
- Show Thumbnail.
- View Bitmap: full-screen view of the image (Ctrl+F).

## 8. Image Menu
- Flip/Rotate: flip horizontal, flip vertical, rotate 90, 180, 270 degrees.
- Stretch/Skew: stretch 1 to 500% each axis, skew -89 to 89 degrees each axis.
- Invert Colors.
- Attributes: width, height, units (inches, cm, pixels), color or black and white, default button.
- Clear Image.
- Draw Opaque (toggles transparent selection mode).

## 9. Colors Menu
- Edit Colors: basic palette of 48 colors and 16 custom slots.
- Define Custom Colors: hue, saturation, luminance, red, green, blue.
- Add to Custom Colors.
- Double-click a palette swatch to open the editor.

## 10. Classic Shortcuts
| Action | Shortcut |
| --- | --- |
| New, Open, Save, Print | Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+P |
| Undo, Redo, Repeat | Ctrl+Z, Ctrl+Y, F4 |
| Cut, Copy, Paste, Select All | Ctrl+X, Ctrl+C, Ctrl+V, Ctrl+A |
| Invert colors | Ctrl+I |
| Attributes, Stretch/Skew | Ctrl+E, Ctrl+W |
| Clear image | Ctrl+Shift+N |
| Grid, Text toolbar, View Bitmap | Ctrl+G, Ctrl+T, Ctrl+F |
| Brush and line size | Ctrl + numeric keypad + / - |
| Cancel current operation | Esc |
