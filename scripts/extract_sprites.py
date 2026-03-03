"""Extract individual sprites from an AoE2 sprite sheet with magenta background."""
from PIL import Image
import os

def extract_sprites(input_path, output_dir, magenta_threshold=60):
    img = Image.open(input_path).convert("RGBA")
    pixels = img.load()
    w, h = img.size

    # Replace background color (160, 0, 96) with transparency
    bg = (160, 0, 96)
    for y in range(h):
        for x in range(w):
            r, g, b, a = pixels[x, y]
            if abs(r - bg[0]) < 10 and abs(g - bg[1]) < 10 and abs(b - bg[2]) < 10:
                pixels[x, y] = (0, 0, 0, 0)

    # Find connected non-transparent regions (bounding boxes)
    visited = [[False] * w for _ in range(h)]
    sprites = []

    def flood_fill_bbox(sx, sy):
        stack = [(sx, sy)]
        min_x, min_y = sx, sy
        max_x, max_y = sx, sy
        count = 0
        while stack:
            cx, cy = stack.pop()
            if cx < 0 or cx >= w or cy < 0 or cy >= h:
                continue
            if visited[cy][cx]:
                continue
            _, _, _, a = pixels[cx, cy]
            if a == 0:
                continue
            visited[cy][cx] = True
            count += 1
            min_x = min(min_x, cx)
            min_y = min(min_y, cy)
            max_x = max(max_x, cx)
            max_y = max(max_y, cy)
            for dx, dy in [(-1,0),(1,0),(0,-1),(0,1)]:
                stack.append((cx+dx, cy+dy))
        return (min_x, min_y, max_x+1, max_y+1, count)

    import sys
    sys.setrecursionlimit(100000)

    for y in range(h):
        for x in range(w):
            if visited[y][x]:
                continue
            _, _, _, a = pixels[x, y]
            if a == 0:
                visited[y][x] = True
                continue
            bbox = flood_fill_bbox(x, y)
            min_x, min_y, max_x, max_y, count = bbox
            bw = max_x - min_x
            bh = max_y - min_y
            # Skip tiny regions (text, noise) and very large ones (the whole image)
            if count > 500 and bw < w * 0.8 and bh < h * 0.8:
                sprites.append((min_x, min_y, max_x, max_y, count))

    os.makedirs(output_dir, exist_ok=True)

    # Sort by position (top-to-bottom, left-to-right)
    sprites.sort(key=lambda s: (s[1] // 100, s[0]))

    print(f"Found {len(sprites)} sprites")
    for i, (x1, y1, x2, y2, count) in enumerate(sprites):
        cropped = img.crop((x1, y1, x2, y2))
        name = f"sprite_{i:02d}_{x2-x1}x{y2-y1}.png"
        cropped.save(os.path.join(output_dir, name))
        print(f"  {name}: {x2-x1}x{y2-y1} at ({x1},{y1}) - {count} pixels")

if __name__ == "__main__":
    extract_sprites(
        "assets/mines.png",
        "assets/extracted/mines"
    )
