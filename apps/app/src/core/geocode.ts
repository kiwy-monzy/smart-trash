// Address search and reverse geocoding with OpenStreetMap's Nominatim.
// Requests are limited to the Tanzania bounding box.

const NOMINATIM = 'https://nominatim.openstreetmap.org';
const VIEWBOX = '29.2,-0.9,40.6,-11.8'; // left, top, right, bottom

export interface Place {
  label: string;
  latitude: number;
  longitude: number;
}

interface SearchRow {
  display_name?: string;
  lat?: string;
  lon?: string;
}

export async function searchAddress(query: string): Promise<Place[]> {
  const url = new URL(`${NOMINATIM}/search`);
  url.searchParams.set('format', 'jsonv2');
  url.searchParams.set('q', query);
  url.searchParams.set('limit', '6');
  url.searchParams.set('countrycodes', 'tz');
  url.searchParams.set('viewbox', VIEWBOX);
  url.searchParams.set('bounded', '1');

  const response = await fetch(url, { headers: { Accept: 'application/json' } });
  if (!response.ok) throw new Error(`address search failed (${response.status})`);

  const rows = (await response.json()) as SearchRow[];
  return rows
    .filter((row) => row.lat && row.lon)
    .map((row) => ({
      label: row.display_name ?? `${row.lat}, ${row.lon}`,
      latitude: Number(row.lat),
      longitude: Number(row.lon),
    }));
}

/** Street-level name for a pair of coordinates, or '' when offline. */
export async function describePoint(latitude: number, longitude: number): Promise<string> {
  const url = new URL(`${NOMINATIM}/reverse`);
  url.searchParams.set('format', 'jsonv2');
  url.searchParams.set('lat', String(latitude));
  url.searchParams.set('lon', String(longitude));
  url.searchParams.set('zoom', '17');

  try {
    const response = await fetch(url, { headers: { Accept: 'application/json' } });
    if (!response.ok) return '';
    const data = (await response.json()) as { display_name?: string };
    return data.display_name ?? '';
  } catch {
    return '';
  }
}
