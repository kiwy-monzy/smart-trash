"""Fetches the rivers of Dar es Salaam from OpenStreetMap (Overpass API).

    python scripts/fetch-rivers.py

Writes var/geo-raw/rivers.geojson - every waterway way inside the
region bbox, with its geometry inlined, so no node lookup is needed. The 674 MB
Tanzania PBF on disk could be used instead, but a full node scan in pyosmium
callbacks takes many minutes for a few thousand rivers.

Overpass needs a User-Agent and rate-limits; run it once and keep the file.
"""

import json
import os
import sys

try:
    import requests
except ImportError:
    sys.exit("requests is missing - pip install requests")

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RAW = os.path.join(ROOT, "var", "geo-raw")
OUT = os.path.join(RAW, "rivers.geojson")

# Overpass boxes are (south, west, north, east) - latitude first. Getting this
# backwards silently returns rivers from another continent.
BBOX = "-7.30,38.90,-6.40,39.80"
KINDS = "river|stream|canal"

QUERY = f"""
[out:json][timeout:300];
way["waterway"~"^({KINDS})$"]({BBOX});
out geom;
"""

HOSTS = [
    "https://overpass-api.de/api/interpreter",
    "https://overpass.kumi.systems/api/interpreter",
    "https://overpass.private.coffee/api/interpreter",
]

HEADERS = {"User-Agent": "smart-trash-map/1.0 (FYP Dar es Salaam extract)"}


def fetch():
    for host in HOSTS:
        try:
            print(f"querying {host}")
            response = requests.post(host, data={"data": QUERY}, headers=HEADERS, timeout=300)
            if response.status_code == 200:
                return response.json()
            print(f"  {response.status_code}")
        except Exception as error:  # noqa: BLE001 - any network failure moves on
            print(f"  {type(error).__name__}: {str(error)[:120]}")
    sys.exit("every Overpass host failed")


def to_feature(element):
    line = [[point["lon"], point["lat"]] for point in element.get("geometry") or []]
    if len(line) < 2:
        return None
    tags = element.get("tags") or {}
    properties = {
        "osm_id": element["id"],
        "waterway": tags.get("waterway"),
        "name": tags.get("name", ""),
    }
    if tags.get("intermittent"):
        properties["intermittent"] = tags["intermittent"]
    return {
        "type": "Feature",
        "properties": properties,
        "geometry": {"type": "LineString", "coordinates": line},
    }


def main():
    payload = fetch()
    features = [f for f in (to_feature(e) for e in payload.get("elements", [])) if f]

    os.makedirs(RAW, exist_ok=True)
    with open(OUT, "w", encoding="utf-8") as handle:
        json.dump({"type": "FeatureCollection", "features": features}, handle, ensure_ascii=False)

    kinds = {}
    for feature in features:
        kind = feature["properties"]["waterway"]
        kinds[kind] = kinds.get(kind, 0) + 1

    print(f"rivers: {len(features)} ways, {os.path.getsize(OUT) / 1024:.0f} KB")
    print("by kind:", kinds)


if __name__ == "__main__":
    main()
