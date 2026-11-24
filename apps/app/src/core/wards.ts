// The 102 official Dar es Salaam wards, read from the same files the map draws.
//
// The backend only knows a handful of wards, so the Areas sheet uses this so the
// list and the map always agree.

const BASE = '/geo';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type Collection = any;

export interface WardArea {
  name: string;
  district: string;
  boundary: string;
  lat: number;
  lng: number;
}

let cache: WardArea[] | null = null;

async function load(name: string): Promise<Collection> {
  const response = await fetch(`${BASE}/${name}`);
  if (!response.ok) throw new Error(`${name} -> ${response.status}`);
  return response.json();
}

function flatten(value: unknown, out: number[][]): void {
  if (!Array.isArray(value)) return;
  if (typeof value[0] === 'number') {
    out.push(value as number[]);
    return;
  }
  for (const item of value) flatten(item, out);
}

/** Good enough to centre the map: the average of the outline points. */
function centre(geometry: Collection): { lat: number; lng: number } | null {
  if (!geometry) return null;
  const points: number[][] = [];
  flatten(geometry.coordinates, points);
  if (!points.length) return null;
  const lng = points.reduce((sum, point) => sum + point[0], 0) / points.length;
  const lat = points.reduce((sum, point) => sum + point[1], 0) / points.length;
  return { lat, lng };
}

function boundaryLabel(source: unknown): string {
  if (source === 'osm') return 'OSM boundary';
  if (source === 'codab_2018') return 'OCHA COD-AB 2018';
  return 'ward office only';
}

function toArea(feature: Collection): WardArea | null {
  const props = feature?.properties ?? {};
  const spot = centre(feature?.geometry);
  if (!props.name || !spot) return null;
  return {
    name: String(props.name),
    district: String(props.district ?? 'Dar es Salaam'),
    boundary: boundaryLabel(props.boundary_source),
    lat: spot.lat,
    lng: spot.lng,
  };
}

export async function listAreas(): Promise<WardArea[]> {
  if (cache) return cache;

  const [polygons, anchors] = await Promise.all([
    load('wards.geojson'),
    load('ward_anchors.geojson'),
  ]);

  const areas = [...(polygons.features as Collection[]), ...(anchors.features as Collection[])]
    .map(toArea)
    .filter((area): area is WardArea => area !== null);

  areas.sort((a, b) => a.district.localeCompare(b.district) || a.name.localeCompare(b.name));
  cache = areas;
  return areas;
}
