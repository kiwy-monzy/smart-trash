"""Puts every waste-site polygon on satellite imagery so the shape can be checked.

    python scripts/check-sites-satellite.py

Writes map-sites-satellite.png: one panel per sit, cropped from Esri World
Imagery, with the OSM ring drawn in red and the ward/area printed underneath.
A polygon that does not sit on a visible tip/mound is a mapping error.
"""

import io
import json
import math
import os
import sys

try:
    import requests
    from PIL import Image, ImageDraw
except ImportError:
    sys.exit("needs requests and pillow - pip install requests pillow")

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
GEO = os.path.join(ROOT, "apps", "app", "public", "geo")
OUT = os.path.join(ROOT, "map-sites-satellite.png")

TILE = (
    "https://services.arcgisonline.com/ArcGIS/rest/services/"
    "World_Imagery/MapServer/tile/{z}/{y}/{x}"
)
HEADERS = {"User-Agent": "smart-trash-map/1.0 (FYP site check)"}
PANEL = 340
ZOOM = 17


def project(lon, lat, zoom):
    """Web-mercator world pixel coordinates at a zoom level."""
    size = 256 * 2**zoom
    x = (lon + 180) / 360 * size
    siny = math.sin(math.radians(lat))
    y = (0.5 - math.log((1 + siny) / (1 - siny)) / (4 * math.pi)) * size
    return x, y


def tile_image(zoom, x, y):
    response = requests.get(TILE.format(z=zoom, x=x, y=y), headers=HEADERS, timeout=60)
    response.raise_for_status()
    return Image.open(io.BytesIO(response.content)).convert("RGB")


def crop(lon, lat, zoom, size):
    """Satellite crop centred on a point, from as few tiles as possible."""
    px, py = project(lon, lat, zoom)
    left = int(px - size / 2)
    top = int(py - size / 2)

    canvas = Image.new("RGB", (size, size))
    first = left // 256
    last = (left + size) // 256
    top_first = top // 256
    top_last = (top + size) // 256

    for tx in range(first, last + 1):
        for ty in range(top_first, top_last + 1):
            img = tile_image(zoom, tx, ty)
            canvas.paste(img, (tx * 256 - left, ty * 256 - top))

    return canvas, left, top


def ring_points(geometry):
    if geometry["type"] == "Polygon":
        return geometry["coordinates"][0]
    return geometry["coordinates"][0][0]


def main():
    with open(os.path.join(GEO, "dumpsites.geojson"), encoding="utf-8") as handle:
        data = json.load(handle)

    sites = [f for f in data["features"] if f["geometry"]["type"] == "Polygon"]
    print(f"{len(sites)} site polygons")

    cols = 3
    rows = max(1, math.ceil(len(sites) / cols))
    sheet = Image.new("RGB", (cols * PANEL, rows * (PANEL + 26)), "white")
    draw = ImageDraw.Draw(sheet)

    for index, feature in enumerate(sites):
        ring = ring_points(feature["geometry"])
        lon = sum(point[0] for point in ring) / len(ring)
        lat = sum(point[1] for point in ring) / len(ring)

        frame, left, top = crop(lon, lat, ZOOM, PANEL)
        marks = ImageDraw.Draw(frame)
        pixels = [
            (project(lng, lt, ZOOM)[0] - left, project(lng, lt, ZOOM)[1] - top)
            for lng, lt in ring
        ]
        marks.line(pixels + [pixels[0]], fill=(255, 40, 40), width=3)

        x0 = (index % cols) * PANEL
        y0 = (index // cols) * (PANEL + 26)
        sheet.paste(frame, (x0, y0))

        props = feature["properties"]
        label = f"{props.get('kind', '?')} · {props.get('zone_ward') or '?'}"
        draw.text((x0 + 6, y0 + PANEL + 6), label, fill=(20, 20, 20))

    sheet.save(OUT)
    print(f"wrote {OUT} ({os.path.getsize(OUT) / 1024:.0f} KB)")


if __name__ == "__main__":
    main()
