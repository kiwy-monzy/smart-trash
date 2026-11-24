// Dar es Salaam basemap drawn from the local GeoJSON extract, so the home map
// needs no tile server and works offline once bundled.
//
// Layers live in apps/app/public/geo and come from scripts/prepare-map.py:
//
//   region.geojson        the region outline (OSM relation 7202037)
//   districts.geojson     5 municipalities, unions of their ward polygons
//   wards.geojson         94 ward polygons (71 OSM + 23 OCHA COD-AB 2018)
//   ward_anchors.geojson  8 wards that only exist as a ward-office point
//   roads_major.geojson   1 973 motorway / trunk / primary / secondary lines
//   minor_roads.geojson   76 729 residential / service / unclassified streets
//   rivers.geojson        2 026 waterway lines
//   sites.geojson         the 11 waste sites mapped as a polygon
//   site_points.geojson    the 35 waste sites mapped as a point
//
// Ward outlines drawn from COD-AB are dashed because OSM has no boundary for
// them; the anchors are circles because there is no polygon anywhere.

import L from 'leaflet';

const BASE = '/geo';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type Collection = any;

// Earth tones only, same palette as the FYP map prototype.
const COLOURS = {
  districtStroke: ['#b45309', '#0f766e', '#86198f', '#4d7c0f', '#be123c'],
  districtFill: ['#f6e7d6', '#d8ece9', '#ecdcee', '#e3ecd3', '#f5dee2'],
  ward: '#c9c4b6',
  anchor: '#b45309',
  roadMotorway: '#b45309',
  roadPrimary: '#a16207',
  roadTertiary: '#b3ac9c',
  region: '#8a8577',
  landfill: '#b91c1c',
  disposal: '#d97706',
  recycling: '#15803d',
  transfer: '#7c2d12',
  river: '#5b9bb5',
  stream: '#8ab8c9',
};

const LABEL_ZOOM = 12;

async function layer(name: string): Promise<Collection> {
  const response = await fetch(`${BASE}/${name}`);
  if (!response.ok) throw new Error(`${name} -> ${response.status}`);
  return response.json();
}

function siteColour(kind: unknown): string {
  if (kind === 'landfill') return COLOURS.landfill;
  if (kind === 'recycling') return COLOURS.recycling;
  if (kind === 'waste_transfer_station') return COLOURS.transfer;
  return COLOURS.disposal;
}

function roadStyle(feature?: { properties?: Record<string, unknown> }) {
  const highway = String(feature?.properties?.highway ?? '');
  if (highway.startsWith('motorway')) return { color: COLOURS.roadMotorway, weight: 2.4, opacity: 0.9 };
  if (/^(trunk|primary)/.test(highway)) return { color: COLOURS.roadPrimary, weight: 1.8, opacity: 0.85 };
  return { color: COLOURS.roadTertiary, weight: 1.1, opacity: 0.8 };
}

function card(title: string, rows: [string, unknown][]): string {
  const body = rows
    .filter(([, value]) => value !== undefined && value !== null && value !== '')
    .map(([label, value]) => `<div><b>${label}</b> ${String(value)}</div>`)
    .join('');
  return `<strong>${title}</strong>${body}`;
}

function boundaryName(source: unknown): string {
  if (source === 'osm') return 'OSM polygon';
  if (source === 'codab_2018') return 'OCHA COD-AB 2018';
  return 'anchor point';
}

export interface Basemap {
  group: L.LayerGroup;
  /** The five municipalities only, so the view ignores stray points. */
  cityBounds: L.LatLngBounds;
}

/**
 * Builds the static city layers once. The map keeps them at the bottom, so the
 * H3 grid, pickups and trucks always draw on top.
 */
export async function buildBasemap(map: L.Map): Promise<Basemap> {
  const [region, districts, wards, anchors, roads, rivers, sites, sitePoints] = await Promise.all([
    layer('region.geojson'),
    layer('districts.geojson'),
    layer('wards.geojson'),
    layer('ward_anchors.geojson'),
    layer('roads_major.geojson'),
    layer('rivers.geojson'),
    layer('sites.geojson'),
    layer('site_points.geojson'),
  ]);

  const group = L.layerGroup();

  const add = (geo: L.GeoJSON) => {
    geo.addTo(group);
  };

  const names = (districts.features as { properties: { name?: string } }[]).map((f) => f.properties.name);

  const districtLayer = L.geoJSON(districts, {
      style: (feature) => {
        const index = Math.max(0, names.indexOf(feature?.properties?.name));
        return {
          color: COLOURS.districtStroke[index % 5],
          weight: 2.2,
          opacity: 0.9,
          fillColor: COLOURS.districtFill[index % 5],
          fillOpacity: 0.85,
        };
      },
      onEachFeature: (feature, featureLayer) => {
        const props = feature.properties ?? {};
        featureLayer.bindTooltip(String(props.name ?? ''), {
          permanent: true,
          direction: 'center',
          className: 'district-label',
        });
        featureLayer.bindPopup(
          card(String(props.name ?? 'District'), [['area', props.area_km2 ? `${props.area_km2} kmÃ‚Â²` : '']]),
        );
      },
  });
  add(districtLayer);
  const cityBounds = districtLayer.getBounds();

  const wardLayer = L.geoJSON(wards, {
    style: (feature) => ({
      color: COLOURS.ward,
      weight: feature?.properties?.boundary_source === 'codab_2018' ? 1 : 1.2,
      opacity: 0.9,
      dashArray: feature?.properties?.boundary_source === 'codab_2018' ? '4 3' : undefined,
      fill: false,
    }),
    onEachFeature: (feature, featureLayer) => {
      const props = feature.properties ?? {};
      featureLayer.bindPopup(
        card(String(props.name ?? 'Ward'), [
          ['municipality', props.district],
          ['boundary', boundaryName(props.boundary_source)],
        ]),
      );
    },
  });
  add(wardLayer);

  add(
    L.geoJSON(anchors, {
      pointToLayer: (_feature, latlng) =>
        L.circleMarker(latlng, {
          radius: 3.5,
          color: COLOURS.anchor,
          weight: 1.2,
          fillColor: '#ffffff',
          fillOpacity: 0.9,
        }),
      onEachFeature: (feature, featureLayer) => {
        const props = feature.properties ?? {};
        featureLayer.bindTooltip(`${props.name ?? ''} Ã‚Â· anchor only`, { direction: 'top' });
        featureLayer.bindPopup(
          card(String(props.name ?? 'Ward'), [
            ['municipality', props.district],
            ['boundary', 'no published polygon'],
            ['anchor', props.anchor_type],
          ]),
        );
      },
    }),
  );

  // Rivers sit under the roads: they are the first thing you look for when
  // checking a ward boundary, and they must not cover a street label.
  add(
    L.geoJSON(rivers, {
      style: (feature) => {
        const kind = String(feature?.properties?.waterway ?? '');
        const big = kind === 'river';
        return {
          color: big ? COLOURS.river : COLOURS.stream,
          weight: big ? 2.2 : 1,
          opacity: 0.85,
          dashArray: feature?.properties?.intermittent === 'yes' ? '5 4' : undefined,
        };
      },
      onEachFeature: (feature, featureLayer) => {
        const props = feature.properties ?? {};
        if (!props.name) return;
        featureLayer.bindTooltip(String(props.name), { sticky: true, direction: 'top' });
        featureLayer.bindPopup(
          card(String(props.name), [
            ['type', props.waterway],
            ['osm', props.osm_id],
          ]),
        );
      },
    }),
  );

  add(
    L.geoJSON(roads, {
      style: (feature) => roadStyle(feature as { properties?: Record<string, unknown> } | undefined),
      onEachFeature: (feature, featureLayer) => {
        const name = feature.properties?.name;
        if (name) featureLayer.bindTooltip(String(name), { sticky: true, direction: 'top' });
      },
    }),
  );

  // Only 11 of the 46 sites are mapped as a polygon; those are drawn to their
  // outline. The other 35 exist only as an OSM node, so they stay a marker —
  // inventing a polygon for them would be made-up geometry.
  add(
    L.geoJSON(sites, {
      style: (feature) => ({
        color: '#ffffff',
        weight: 1.4,
        opacity: 1,
        fillColor: siteColour(feature?.properties?.kind),
        fillOpacity: 0.75,
      }),
      onEachFeature: (feature, featureLayer) => {
        const props = feature.properties ?? {};
        featureLayer.bindTooltip(String(props.kind ?? 'waste site'), { direction: 'top' });
        featureLayer.bindPopup(
          card(String(props.name ?? props.kind ?? 'Waste site'), [
            ['type', props.kind],
            ['ward', props.zone_ward],
            ['municipality', props.zone_district],
            ['osm', props.osm_id],
          ]),
        );
      },
    }),
  );

  add(
    L.geoJSON(sitePoints, {
      pointToLayer: (feature, latlng) =>
        L.circleMarker(latlng, {
          radius: 5,
          color: '#ffffff',
          weight: 1.6,
          fillColor: siteColour(feature.properties?.kind),
          fillOpacity: 0.95,
        }),
      onEachFeature: (feature, featureLayer) => {
        const props = feature.properties ?? {};
        featureLayer.bindPopup(
          card(String(props.kind ?? 'Waste site'), [
            ['type', props.kind],
            ['ward', props.zone_ward],
            ['municipality', props.zone_district],
            ['mapped as', 'OSM node only'],
          ]),
        );
      },
    }),
  );

  // Region outline last so the city edge stays crisp over the fills.
  add(
    L.geoJSON(region, {
      style: { color: COLOURS.region, weight: 2.4, opacity: 0.95, dashArray: '8 5', fill: false },
    }),
  );

  const labels = L.layerGroup();
  wardLayer.eachLayer((item) => {
    const featureLayer = item as L.GeoJSON & { feature?: { properties?: Record<string, unknown> } };
    const name = featureLayer.feature?.properties?.name;
    const centre = featureLayer.getBounds().getCenter();
    if (!name || !centre) return;
    L.marker(centre, {
      icon: L.divIcon({ className: 'ward-label', html: String(name), iconSize: [0, 0] }),
      interactive: false,
    }).addTo(labels);
  });

  const syncLabels = () => {
    const want = map.getZoom() >= LABEL_ZOOM;
    if (want && !group.hasLayer(labels)) labels.addTo(group);
    if (!want && group.hasLayer(labels)) group.removeLayer(labels);
  };
  map.on('zoomend', syncLabels);
  syncLabels();

  return { group, cityBounds };
}
