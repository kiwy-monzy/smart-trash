// Minor streets, drawn into one canvas with viewport culling.
//
// 76 000 polylines is far too many for Leaflet path layers (each one becomes an
// SVG node) and it is also too many to redraw per frame. So this layer keeps the
// lines as plain arrays, skips everything outside the current view and strokes
// the rest into a single canvas after each move.

import L from 'leaflet';

interface Line {
  points: [number, number][];
  south: number;
  west: number;
  north: number;
  east: number;
}

const MIN_ZOOM = 10;

function toLine(coordinates: [number, number][]): Line {
  let south = 90;
  let west = 180;
  let north = -90;
  let east = -180;
  for (const [lng, lat] of coordinates) {
    if (lat < south) south = lat;
    if (lat > north) north = lat;
    if (lng < west) west = lng;
    if (lng > east) east = lng;
  }
  return { points: coordinates, south, west, north, east };
}

export async function fetchMinorRoads(): Promise<Line[]> {
  const response = await fetch('/geo/minor_roads.geojson');
  if (!response.ok) throw new Error(`minor_roads.geojson -> ${response.status}`);
  const data = await response.json();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const features = (data.features ?? []) as any[];
  return features
    .filter((feature) => feature.geometry?.coordinates?.length > 1)
    .map((feature) => toLine(feature.geometry.coordinates as [number, number][]));
}

/** Leaflet layer that owns one canvas and paints the streets behind the markers. */
export function minorRoadsLayer(lines: Line[]): L.Layer {
  const MinorRoads = L.Layer.extend({
    onAdd(map: L.Map) {
      this._map = map;
      this._canvas = L.DomUtil.create('canvas', 'leaflet-minor-roads') as HTMLCanvasElement;
      this._canvas.style.position = 'absolute';
      this._canvas.style.pointerEvents = 'none';
      map.getPanes().overlayPane.appendChild(this._canvas);
      map.on('moveend zoomend resize', this._reset, this);
      this._reset();
    },

    onRemove(map: L.Map) {
      map.off('moveend zoomend resize', this._reset, this);
      this._canvas?.remove();
    },

    _reset() {
      const map = this._map as L.Map;
      const size = map.getSize();
      const topLeft = map.containerPointToLayerPoint([0, 0]);
      const canvas = this._canvas as HTMLCanvasElement;
      const ratio = window.devicePixelRatio || 1;

      L.DomUtil.setPosition(canvas, topLeft);
      canvas.width = Math.round(size.x * ratio);
      canvas.height = Math.round(size.y * ratio);
      canvas.style.width = `${size.x}px`;
      canvas.style.height = `${size.y}px`;

      const context = canvas.getContext('2d');
      if (!context) return;
      context.setTransform(ratio, 0, 0, ratio, 0, 0);
      context.clearRect(0, 0, size.x, size.y);
      if (map.getZoom() < MIN_ZOOM) return;

      const view = map.getBounds();
      context.strokeStyle = '#c3bcae';
      context.lineWidth = map.getZoom() >= 15 ? 1.4 : 1.1;
      context.lineCap = 'round';
      context.beginPath();

      for (const line of lines) {
        if (
          line.north < view.getSouth() ||
          line.south > view.getNorth() ||
          line.east < view.getWest() ||
          line.west > view.getEast()
        ) {
          continue;
        }
        const points = line.points;
        const first = map.latLngToLayerPoint([points[0][1], points[0][0]]).subtract(topLeft);
        context.moveTo(first.x, first.y);
        for (let i = 1; i < points.length; i++) {
          const next = map.latLngToLayerPoint([points[i][1], points[i][0]]).subtract(topLeft);
          context.lineTo(next.x, next.y);
        }
      }

      context.stroke();
    },
  });

  return new (MinorRoads as unknown as new () => L.Layer)();
}
