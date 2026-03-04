"""Extract individual isometric ground tiles from the seasons tile sheet."""
from PIL import Image
import os

INPUT = os.path.join(
    os.path.dirname(__file__),
    "..",
    ".cursor-assets",
    "seasons_tiles.png",
)
OUTPUT_DIR = os.path.join(os.path.dirname(__file__), "..", "assets", "textures", "ground")

TILE_W = 128
TILE_H = 64
COLS = 8
ROWS = 12

# Seasons span 4 rows each
SEASONS = [
    ("summer", 0),
    ("autumn", 4),
    ("winter", 8),
]

# Column categories (based on visual inspection of the sheet)
# Cols 0-4: pure terrain, Col 5: stump, Cols 6-7: rocks
COLUMN_TAGS = [
    "grass_a", "grass_b", "grass_c", "dirt_a", "dirt_b",
    "stump", "rock_sm", "rock_lg",
]


def extract():
    # Try multiple possible source locations
    candidates = [
        INPUT,
        os.path.join(os.path.dirname(__file__), "..", "assets", "seasons_tiles.png"),
    ]
    # Also look for the cursor-saved version
    import glob
    cursor_assets = glob.glob(
        "/Users/jon/.cursor/projects/Users-jon-rust-age-of-rust/assets/seasons_tiles*.png"
    )
    candidates.extend(cursor_assets)

    img = None
    for path in candidates:
        if os.path.exists(path):
            img = Image.open(path).convert("RGBA")
            print(f"Loaded: {path}")
            break

    if img is None:
        print("ERROR: Could not find seasons tile sheet. Tried:")
        for p in candidates:
            print(f"  {p}")
        return

    assert img.size == (TILE_W * COLS, TILE_H * ROWS), f"Unexpected size: {img.size}"

    os.makedirs(OUTPUT_DIR, exist_ok=True)
    count = 0

    for season_name, start_row in SEASONS:
        for row_offset in range(4):
            row = start_row + row_offset
            for col in range(COLS):
                x = col * TILE_W
                y = row * TILE_H
                tile = img.crop((x, y, x + TILE_W, y + TILE_H))

                # Skip fully transparent tiles
                if tile.getextrema()[3][1] == 0:
                    continue

                tag = COLUMN_TAGS[col]
                name = f"{season_name}_{tag}_r{row_offset}.png"
                tile.save(os.path.join(OUTPUT_DIR, name))
                count += 1

    print(f"Extracted {count} tiles to {OUTPUT_DIR}")


if __name__ == "__main__":
    extract()
