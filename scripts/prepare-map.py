"""Turns the raw Dar es Salaam extract into the layers the home map draws.

Reads  var/geo-raw/*.geojson   (see scripts/fetch-map-data.ps1)
Writes apps/app/public/geo/*.geojson + manifest.json

    python scripts/prepare-map.py

The ward source mixes 102 official NBS-2022 wards: OSM polygons where OSM has
a boundary relation, OCHA COD-AB 2018 polygons where it does not, and a ward
office point for the eight wards created after 2018. Polygons and points are
split into two files so the map can style them differently.
"""

import json
import os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
GEO = os.path.join(ROOT, "apps", "app", "public", "geo")
RAW = os.path.join(ROOT, "var", "geo-raw")

PRECISION = 5  # ~1.1 m, plenty for a city basemap and much smaller on disk


def load(name):
    path = os.path.join(RAW, name)
    if not os.path.exists(path):
        raise SystemExit(f"missing {path} - run scripts/fetch-map-data.ps1 first")
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def round_coords(value):
    if isinstance(value, (int, float)):
        return round(value, PRECISION)
    if isinstance(value, list):
        return [round_coords(item) for item in value]
    return value


def collect_rings(geometry):
    if not geometry:
        return 0
    kind = geometry.get("type")
    coords = geometry.get("coordinates")
    if kind == "Polygon":
        return len(coords)
    if kind == "MultiPolygon":
        return sum(len(polygon) for polygon in coords)
    return 0


def flatten(value, out):
    if isinstance(value, (int, float)):
        return
    if isinstance(value, list) and value and isinstance(value[0], (int, float)):
        out.append((value[0], value[1]))
        return
    for item in value:
        flatten(item, out)


def bounds_of(features):
    points = []
    for feature in features:
        if feature.get("geometry"):
            flatten(feature["geometry"]["coordinates"], points)
    if not points:
        return None
    lons = [p[0] for p in points]
    lats = [p[1] for p in points]
    return (min(lons), min(lats), max(lons), max(lats))


def inside(box, feature):
    if not box or not feature.get("geometry"):
        return False
    points = []
    flatten(feature["geometry"]["coordinates"], points)
    if not points:
        return False
    lon = sum(p[0] for p in points) / len(points)
    lat = sum(p[1] for p in points) / len(points)
    return box[0] <= lon <= box[2] and box[1] <= lat <= box[3]


def trim(feature, keep):
    props = feature.get("properties") or {}
    trimmed = {key: props[key] for key in keep if key in props}
    geometry = feature.get("geometry")
    if geometry:
        geometry = dict(geometry, coordinates=round_coords(geometry["coordinates"]))
    return {"type": "Feature", "properties": trimmed, "geometry": geometry}


def write(name, features):
    payload = {"type": "FeatureCollection", "features": features}
    path = os.path.join(GEO, name)
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(payload, handle, ensure_ascii=False, separators=(",", ":"))
    size = os.path.getsize(path) / 1024
    print(f"{name:24} {len(features):4} features {size:8.0f} KB")
    return size


def simplify(points, tolerance):
    """Douglas-Peucker, so a street grid stops carrying a point every 3 metres."""
    points = [[point[0], point[1]] for point in points]  # some ways carry elevation
    if len(points) < 3:
        return points

    keep = [False] * len(points)
    keep[0] = keep[-1] = True
    stack = [(0, len(points) - 1)]

    while stack:
        first, last = stack.pop()
        if last <= first + 1:
            continue
        ax, ay = points[first]
        bx, by = points[last]
        dx, dy = bx - ax, by - ay
        length = dx * dx + dy * dy
        worst = 0.0
        index = -1
        for i in range(first + 1, last):
            px, py = points[i]
            if length == 0:
                dist = (px - ax) ** 2 + (py - ay) ** 2
            else:
                t = max(0.0, min(1.0, ((px - ax) * dx + (py - ay) * dy) / length))
                dist = (px - (ax + t * dx)) ** 2 + (py - (ay + t * dy)) ** 2
            if dist > worst:
                worst = dist
                index = i
        if worst > tolerance * tolerance and index > 0:
            keep[index] = True
            stack.append((first, index))
            stack.append((index, last))

    return [point for point, flag in zip(points, keep) if flag]


def line_features(source, keep_props, tolerance, filter_prop=None, allowed=None):
    out = []
    for feature in source["features"]:
        geometry = feature.get("geometry")
        if not geometry or geometry["type"] != "LineString":
            continue
        props = feature.get("properties") or {}
        if allowed is not None and props.get(filter_prop) not in allowed:
            continue
        line = simplify(geometry["coordinates"], tolerance)
        if len(line) < 2:
            continue
        out.append(
            {
                "type": "Feature",
                "properties": {key: props[key] for key in keep_props if key in props},
                "geometry": {"type": "LineString", "coordinates": round_coords(line)},
            }
        )
    return out


def main():
    region = load("region.geojson")
    districts = load("districts.geojson")
    wards = load("wards_official.geojson")
    sites = load("dumpsites.geojson")
    roads = load("roads_major.geojson")

    write("region.geojson", [trim(f, ["name", "admin_level", "osm_id"]) for f in region["features"]])

    write(
        "districts.geojson",
        [
            trim(f, ["name", "boundary_source", "area_km2", "district_official", "wards_total"])
            for f in districts["features"]
        ],
    )

    ward_polygons = []
    ward_points = []
    for feature in wards["features"]:
        geometry = feature.get("geometry")
        if not geometry:
            continue
        if geometry["type"] == "Point":
            ward_points.append(
                trim(feature, ["name", "district", "boundary_source", "anchor_type", "note"])
            )
        else:
            ward_polygons.append(
                trim(feature, ["name", "district", "boundary_source", "nbs_2022_ward"])
            )

    write("wards.geojson", ward_polygons)
    write("ward_anchors.geojson", ward_points)

    # A handful of "waste" features sit far outside the city; keep only the
    # ones inside the region so the map bounds stay tight.
    region_box = bounds_of(region["features"])
    in_city = [f for f in sites["features"] if inside(region_box, f)]

    # OSM maps most sites as a point and a few as a polygon, so the two are
    # kept apart: a polygon is drawn as an area, a point stays a marker.
    site_polygons = []
    site_points = []
    for feature in in_city:
        if feature["properties"].get("in_region") is False:
            continue
        geometry = feature.get("geometry")
        if not geometry:
            continue
        props = trim(feature, ["kind", "zone_ward", "zone_district", "osm_id", "name"])
        if geometry["type"] == "Point":
            site_points.append(props)
        else:
            site_polygons.append(props)

    write("sites.geojson", site_polygons)
    write("site_points.geojson", site_points)

    write(
        "roads_major.geojson",
        [trim(f, ["highway", "name"]) for f in roads["features"]],
    )

    # Tracks and footways are mostly outside the built-up area and would just
    # add noise (and megabytes) to a street map.
    WANTED = {"residential", "service", "unclassified"}
    minor = line_features(
        load("roads_minor.geojson"),
        ["highway"],
        tolerance=0.00006,
        filter_prop="highway",
        allowed=WANTED,
    )
    write("minor_roads.geojson", minor)

    rivers = []
    if os.path.exists(os.path.join(RAW, "rivers.geojson")):
        rivers = line_features(
            load("rivers.geojson"),
            ["waterway", "name", "intermittent"],
            tolerance=0.00004,
        )
        write("rivers.geojson", rivers)
    else:
        print("rivers.geojson missing - run scripts/fetch-rivers.py")

    # A short manifest so the app can sanity-check what it loaded.
    by_district = {}
    for feature in ward_polygons:
        district = feature["properties"].get("district") or "unknown"
        by_district[district] = by_district.get(district, 0) + 1

    by_kind = {}
    for feature in site_polygons + site_points:
        kind = feature["properties"].get("kind") or "unknown"
        by_kind[kind] = by_kind.get(kind, 0) + 1

    kept = len(site_polygons) + len(site_points)
    outside = len(sites["features"]) - kept

    manifest = {
        "region": region["features"][0]["properties"].get("name"),
        "districts": [f["properties"].get("name") for f in districts["features"]],
        "ward_polygons": len(ward_polygons),
        "ward_anchors": len(ward_points),
        "ward_polygons_by_district": by_district,
        "ward_polygons_osm": sum(
            1 for f in ward_polygons if f["properties"].get("boundary_source") == "osm"
        ),
        "ward_polygons_codab": sum(
            1 for f in ward_polygons if f["properties"].get("boundary_source") == "codab_2018"
        ),
        "waste_sites": kept,
        "waste_site_polygons": len(site_polygons),
        "waste_site_points": len(site_points),
        "waste_sites_outside_region": outside,
        "waste_sites_by_kind": by_kind,
        "road_features": len(roads["features"]),
        "minor_road_features": len(minor),
        "river_features": len(rivers),
        "rings": sum(collect_rings(f["geometry"]) for f in ward_polygons),
        "attribution": "Â© OpenStreetMap contributors (ODbL); ward names NBS 2022",
    }

    with open(os.path.join(GEO, "manifest.json"), "w", encoding="utf-8") as handle:
        json.dump(manifest, handle, indent=2)

    print("\nmanifest")
    for key in ("region", "districts", "ward_polygons", "ward_anchors", "waste_sites", "road_features"):
        print(f"  {key}: {manifest[key]}")
    print(f"  wards by district: {by_district}")
    print(f"  sites by kind: {by_kind}")


if __name__ == "__main__":
    main()
