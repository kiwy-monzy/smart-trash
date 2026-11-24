// Map layer: Dar es Salaam basemap, H3 hexes per vehicle, pickup pins, route.

import L from 'leaflet';
import 'leaflet/dist/leaflet.css';
import { cellToBoundary } from 'h3-js';

import type { Vehicle } from '../generated/fleet';
import { VehicleState } from '../generated/fleet';
import type { User } from '../generated/identity';
import type { Pickup } from '../generated/pickup';
import type { Route } from '../generated/routing';
import { buildBasemap } from './basemap';
import { fetchMinorRoads, minorRoadsLayer } from './minorRoads';

const DAR_ES_SALAAM: [number, number] = [-6.7924, 39.2083];

// Roughly how much of the window the bottom sheet hides when it is half open.
const SHEET_HEIGHT = 300;

// The map never leaves Tanzania.
const TANZANIA: [[number, number], [number, number]] = [
  [-11.8, 29.2],
  [-0.9, 40.6],
];

/** Green when on a trip, blue when available, grey when the driver is off. */
function vehicleColour(vehicle: Vehicle): string {
  if (!vehicle.activated) return '#8a9a94';
  if (vehicle.state === VehicleState.ON_ROUTE) return '#0f6e5a';
  if (vehicle.state === VehicleState.AVAILABLE || vehicle.state === VehicleState.IDLE) return '#2563eb';
  if (vehicle.state === VehicleState.MAINTENANCE) return '#b45309';
  return '#b42318';
}

export interface MapData {
  vehicles: Vehicle[];
  pickups: Pickup[];
  users: User[];
  route: Route | null;
}

export class MapView {
  private map: L.Map;
  private vehicleLayer = L.layerGroup();
  private pickupLayer = L.layerGroup();
  private hexLayer = L.layerGroup();
  private userLayer = L.layerGroup();
  private routeLayer = L.layerGroup();
  private baseLayer = L.layerGroup();
  private streetLayer = L.layerGroup();
  private fitted = false;

  constructor(element: HTMLElement) {
    this.map = L.map(element, {
      zoomControl: false,
      attributionControl: false,
      // SVG rather than canvas: the canvas renderer caches its pixel size and
      // comes out the wrong size inside the Tauri webview.
      preferCanvas: false,
      zoomSnap: 0.25,
      minZoom: 6,
      maxZoom: 18,
      maxBounds: TANZANIA,
      maxBoundsViscosity: 0.9,
    }).setView(DAR_ES_SALAAM, 11);


    // The base map is the local Dar es Salaam extract, not a tile server.
    this.baseLayer.addTo(this.map);
    void this.loadBasemap();

    // Streets sit between the base and the markers.
    this.streetLayer.addTo(this.map);
    void this.loadStreets();

    this.vehicleLayer.addTo(this.map);
    this.hexLayer.addTo(this.map);
    this.userLayer.addTo(this.map);
    this.pickupLayer.addTo(this.map);
    this.routeLayer.addTo(this.map);
  }

  private async loadBasemap(): Promise<void> {
    try {
      const basemap = await buildBasemap(this.map);
      basemap.group.addTo(this.baseLayer);
      if (basemap.cityBounds.isValid()) this.fitCity(basemap.cityBounds);
    } catch (error) {
      console.warn('basemap failed to load', error);
    }
  }

  /**
   * Minor streets are a bigger download than the rest of the map, so they are
   * added once the city is already on screen instead of blocking first paint.
   */
  private async loadStreets(): Promise<void> {
    try {
      const lines = await fetchMinorRoads();
      this.streetLayer.addLayer(minorRoadsLayer(lines));
    } catch (error) {
      console.warn('minor roads failed to load', error);
    }
  }

  /** The canvas renderer keeps whatever size it saw first, so resync on load. */
  private fitCity(bounds: L.LatLngBounds): void {
    this.map.invalidateSize();
    this.map.fitBounds(bounds, {
      paddingTopLeft: [8, 8],
      paddingBottomRight: [8, SHEET_HEIGHT],
      maxZoom: 11,
      animate: false,
    });
  }

  render(data: MapData): void {
    this.vehicleLayer.clearLayers();
    this.hexLayer.clearLayers();
    this.userLayer.clearLayers();
    this.pickupLayer.clearLayers();
    this.routeLayer.clearLayers();

    for (const vehicle of data.vehicles) {
      this.drawHex(vehicle);
      this.drawVehicle(vehicle);
    }
    for (const user of data.users) {
      this.drawUser(user);
    }
    for (const pickup of data.pickups) {
      this.drawPickup(pickup);
    }
    if (data.route) {
      this.drawRoute(data.route);
    }

    if (!this.fitted && (data.vehicles.length || data.pickups.length)) {
      this.fitted = true;
    }
  }

  private drawHex(vehicle: Vehicle): void {
    if (!vehicle.h3Cell) return;
    const colour = vehicleColour(vehicle);

    for (const cell of [vehicle.h3Cell]) {
      let boundary: [number, number][];
      try {
        boundary = cellToBoundary(cell).map(([lat, lng]) => [lat, lng] as [number, number]);
      } catch {
        continue;
      }

      L.polygon(boundary, {
        color: colour,
        weight: 1.5,
        opacity: 0.9,
        fillColor: colour,
        fillOpacity: 0.14,
        interactive: false,
      }).addTo(this.hexLayer);
    }
  }

  private drawVehicle(vehicle: Vehicle): void {
    const colour = vehicleColour(vehicle);
    const icon = L.divIcon({
      className: 'map-vehicle',
      html: `<span class="dot" style="background:${colour}"></span><span class="plate">${vehicle.registrationPlate}</span>`,
      iconSize: [92, 26],
      iconAnchor: [13, 13],
    });

    L.marker([vehicle.latitude, vehicle.longitude], { icon, title: vehicle.vehicleId })
      .bindTooltip(
        `${vehicle.vehicleId} ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â· ${vehicle.activated ? 'active' : 'inactive'} ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â· ${vehicle.h3Cell.slice(0, 7)}`,
        { direction: 'top', offset: [0, -10] },
      )
      .addTo(this.vehicleLayer);
  }

  private drawUser(user: User): void {
    if (!user.h3Cell || !user.latitude) return;

    try {
      const boundary = cellToBoundary(user.h3Cell).map(
        ([lat, lng]) => [lat, lng] as [number, number],
      );
      L.polygon(boundary, {
        color: '#2563eb',
        weight: 1,
        opacity: 0.5,
        fillColor: '#2563eb',
        fillOpacity: 0.06,
        dashArray: '4 4',
        interactive: false,
      }).addTo(this.userLayer);
    } catch {
      return;
    }

    L.circleMarker([user.latitude, user.longitude], {
      radius: 4,
      color: '#ffffff',
      weight: 1.5,
      fillColor: '#2563eb',
      fillOpacity: 0.9,
    })
      .bindTooltip(`${user.displayName || user.userId} ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â· ${user.ward}`, {
        direction: 'top',
        offset: [0, -6],
      })
      .addTo(this.userLayer);
  }

  private drawPickup(pickup: Pickup): void {
    const icon = L.divIcon({
      className: 'map-pickup',
      html: `<span class="pin ${pickup.status === 1 ? 'pending' : ''}"></span>`,
      iconSize: [14, 14],
      iconAnchor: [7, 7],
    });

    L.marker([pickup.latitude, pickup.longitude], { icon, title: pickup.pickupId })
      .bindTooltip(`${pickup.pickupId} ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â· ${pickup.h3Cell.slice(0, 7)}`, {
        direction: 'top',
        offset: [0, -8],
      })
      .addTo(this.pickupLayer);
  }

  private drawRoute(route: Route): void {
    const points: [number, number][] = [
      [route.stops[0]?.latitude ?? DAR_ES_SALAAM[0], route.stops[0]?.longitude ?? DAR_ES_SALAAM[1]],
      ...route.stops.map((stop) => [stop.latitude, stop.longitude] as [number, number]),
    ];

    L.polyline(points, { color: '#ffffff', weight: 7, opacity: 0.9 }).addTo(this.routeLayer);
    L.polyline(points, { color: '#0f6e5a', weight: 4, dashArray: '8 8' }).addTo(this.routeLayer);

    route.stops.forEach((stop, index) => {
      L.marker([stop.latitude, stop.longitude], {
        icon: L.divIcon({
          className: 'map-stop',
          html: `<span>${index + 1}</span>`,
          iconSize: [22, 22],
          iconAnchor: [11, 11],
        }),
      })
        .bindTooltip(`${index + 1}. ${stop.pickupId}`, { direction: 'top', offset: [0, -10] })
        .addTo(this.routeLayer);
    });

    this.map.fitBounds(L.latLngBounds(points).pad(0.25), { animate: true });
  }

  focus(latitude: number, longitude: number, zoom = 15): void {
    this.map.setView([latitude, longitude], zoom, { animate: true });
  }

  refresh(): void {
    this.map.invalidateSize();
  }
}

let view: MapView | null = null;

export function mountMap(element: HTMLElement): void {
  view = new MapView(element);
}

export function renderMap(data: MapData): void {
  view?.render(data);
}

export function focusMap(latitude: number, longitude: number, zoom = 15): void {
  view?.focus(latitude, longitude, zoom);
}
