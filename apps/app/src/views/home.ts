import type { Actions, AppState } from '../app';
import { WASTE_OPTIONS, coordinate, statusClass, statusLabel, volume, wasteLabel } from '../core/format';
import { esc } from '../core/html';
import type { WardArea } from '../core/wards';
import { Role } from '../generated/identity';
import { VehicleState } from '../generated/fleet';
import type { Vehicle } from '../generated/fleet';

function card(emoji: string, title: string, sub: string, action: string): string {
  return `
    <button class="quick" data-open="${action}">
      <span class="quick-emoji">${emoji}</span>
      <strong>${esc(title)}</strong>
      <small>${esc(sub)}</small>
    </button>`;
}

function cards(state: AppState): string {
  const role = state.session!.role;
  const items = [];

  if (role === Role.HOUSEHOLD || role === Role.BUSINESS) {
    items.push(card('🗑️', 'Request pickup', 'Bins at your door', 'request'));
  }
  items.push(card('🚛', 'Fleet', `${state.vehicles.length} trucks`, 'fleet'));
  items.push(card('🗺️', 'Areas', 'Dar es Salaam wards', 'search'));

  if (role === Role.COUNCIL) {
    items.push(card('🧭', 'Route', 'Plan today', 'route'));
    items.push(card('📅', 'Schedule', `${state.windows.length} windows`, 'jobs'));
  }

  return `<div class="quick-grid">${items.join('')}</div>`;
}

function recent(state: AppState): string {
  const rows = state.pickups.slice(0, 3);
  if (!rows.length) return '<p class="empty">No pickups yet.</p>';

  return `<div class="rows">${rows
    .map(
      (p) => `
      <button class="row" data-pickup="${esc(p.pickupId)}">
        <span class="row-icon">🕘</span>
        <span class="row-main">
          <strong>${esc(p.pickupId)}</strong>
          <small>${esc(wasteLabel(p.wasteType))} · ${esc(volume(p.estimatedVolume))}</small>
        </span>
        <span class="pill ${statusClass(p.status)}">${esc(statusLabel(p.status))}</span>
      </button>`,
    )
    .join('')}</div>`;
}

function searchBody(state: AppState): string {
  const region = state.regions.find((r) => r.code === state.regionCode);

  return `
    <div class="sheet-head">
      <button class="back" data-sheet="menu">←</button>
      <strong>Where to?</strong>
      <span class="chip">${esc(region ? region.name : 'Dar es Salaam')}</span>
    </div>

    <form id="address-form" class="address-form">
      <input name="q" value="${esc(state.query)}" placeholder="Search an address, street or place…"
             autocomplete="off" spellcheck="false" />
      <button type="submit" ${state.geoBusy ? 'disabled' : ''}>Search</button>
    </form>

    ${state.address ? `<p class="picked">📍 ${esc(state.address)}</p>` : ''}

    ${state.places
      .map(
        (place, index) => `
      <button class="row" data-place="${index}">
        <span class="row-icon">📌</span>
        <span class="row-main">
          <strong>${esc(place.label.split(',')[0])}</strong>
          <small>${esc(place.label.split(',').slice(1, 4).join(',').trim())}</small>
        </span>
      </button>`,
      )
      .join('')}

    <h3 class="sheet-title">Wards in ${esc(region ? region.name : 'this region')}</h3>
    ${
      state.areas.length
        ? areasBody(state.areas)
        : state.wards.length
          ? `<div class="rows">${state.wards
              .map(
                (w) => `
          <button class="row" data-ward="${esc(w.code)}" data-lat="${w.centerLatitude}" data-lng="${w.centerLongitude}">
            <span class="row-icon">📍</span>
            <span class="row-main"><strong>${esc(w.name)}</strong><small>${esc(w.regionCode)}</small></span>
          </button>`,
              )
              .join('')}</div>`
          : `<p class="empty">${esc(region ? region.name : 'This region')} has no ward data yet. Only Dar es Salaam does.</p>`
    }

    <h3 class="sheet-title">Regions</h3>
    <div class="rows">
      ${state.regions
        .map(
          (r) => `
        <button class="row ${r.code === state.regionCode ? 'on' : ''}" data-region="${esc(r.code)}">
          <span class="row-icon">${r.hasWards ? '✅' : '•'}</span>
          <span class="row-main"><strong>${esc(r.name)}</strong>
            <small>${esc(r.code)}${r.hasWards ? ' · wards available' : ''}</small></span>
        </button>`,
        )
        .join('')}
    </div>`;
}

function areasBody(areas: WardArea[]): string {
  const groups = new Map<string, WardArea[]>();
  for (const area of areas) {
    const list = groups.get(area.district) ?? [];
    list.push(area);
    groups.set(area.district, list);
  }

  return [...groups.entries()]
    .map(
      ([district, list]) => `
      <h4 class="area-group">${esc(district)} · ${list.length}</h4>
      <div class="rows">
        ${list
          .map(
            (area) => `
        <button class="row" data-ward="${esc(area.name)}" data-lat="${area.lat}" data-lng="${area.lng}">
          <span class="row-icon">📍</span>
          <span class="row-main"><strong>${esc(area.name)}</strong><small>${esc(area.boundary)}</small></span>
        </button>`,
          )
          .join('')}
      </div>`,
    )
    .join('');
}

function fleetBody(state: AppState): string {
  const main = (v: Vehicle) => `
    <button class="row" data-vehicle="${esc(v.vehicleId)}">
      <span class="dot" style="background:${v.activated ? '#0f6e5a' : '#8a9a94'}"></span>
      <span class="row-main">
        <strong>${esc(v.registrationPlate)}</strong>
        <small>${esc(v.vehicleId)} · ${v.activated ? 'on duty' : 'off duty'}${
          v.state === VehicleState.ON_ROUTE ? ' · on route' : ''
        }</small>
        <small class="mono">${esc(v.h3Cell)}</small>
      </span>
    </button>`;

  const driver = state.session!.role === Role.DRIVER;
  const mine = state.vehicles.find((v) => v.driverId === state.session!.userId);

  return `
    <div class="sheet-head">
      <button class="back" data-sheet="menu">←</button>
      <strong>Fleet</strong>
    </div>
    <div class="rows">${state.vehicles.map(main).join('') || '<p class="empty">No vehicles.</p>'}</div>
    ${driver && mine ? `<button class="block ${mine.activated ? 'ghost' : ''}" id="toggle-vehicle">${
      mine.activated ? 'Go off duty' : 'Go on duty'
    }</button>` : ''}`;
}

function routeBody(state: AppState): string {
  const route = state.route;

  return `
    <div class="sheet-head">
      <button class="back" data-sheet="menu">←</button>
      <strong>Route</strong>
      <button class="ghost small" id="optimize">${route ? 'Re-plan' : 'Plan'}</button>
    </div>

    ${
      route
        ? `<p class="meta">${esc(route.routeId)} · ${route.stops.length} stops · ${route.totalDistanceKm.toFixed(1)} km · ${route.totalDurationMinutes} min</p>
           <ol class="stops">${route.stops
             .map(
               (s) => `<li><span>${s.sequence}</span><div><strong>${esc(s.pickupId)}</strong>
                 <small class="mono">${esc(coordinate(s.latitude))}, ${esc(coordinate(s.longitude))}</small></div></li>`,
             )
             .join('')}</ol>`
        : '<p class="empty">Plan the pending pickups for today.</p>'
    }`;
}

function jobsBody(state: AppState): string {
  return `
    <div class="sheet-head">
      <button class="back" data-sheet="menu">←</button>
      <strong>Collection windows</strong>
    </div>

    ${
      state.windows.length
        ? `<div class="rows">${state.windows
            .map(
              (w) => `
          <div class="row static">
            <span class="row-icon">📅</span>
            <span class="row-main">
              <strong>${esc(w.zone)}</strong>
              <small>${esc(new Date(Number(w.startUnix) * 1000).toLocaleString([], { weekday: 'short', hour: '2-digit', minute: '2-digit' }))} – ${esc(new Date(Number(w.endUnix) * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }))}</small>
              <small class="mono">${esc(w.windowId)}</small>
            </span>
          </div>`,
            )
            .join('')}</div>`
        : '<p class="empty">No windows yet.</p>'
    }

    <button class="block" id="new-window">New window (tomorrow 08–12)</button>`;
}

function requestBody(state: AppState): string {
  return `
    <div class="sheet-head">
      <button class="back" data-sheet="menu">←</button>
      <strong>Request pickup</strong>
    </div>

    <form id="create-form" class="form">
      <label class="field"><span>Waste</span>
        <select name="waste_type">
          ${WASTE_OPTIONS.map((o) => `<option value="${esc(o.value)}">${esc(o.label)}</option>`).join('')}
        </select>
      </label>
      <label class="field"><span>Volume (m³)</span>
        <input name="estimated_volume" type="number" step="0.1" min="0.1" value="2.5" />
      </label>
      <div class="row-2">
        <label class="field"><span>Latitude</span>
          <input name="latitude" type="number" step="any" value="-6.7565" />
        </label>
        <label class="field"><span>Longitude</span>
          <input name="longitude" type="number" step="any" value="39.2049" />
        </label>
      </div>
      <button type="button" class="ghost small" id="use-location" ${state.geoBusy ? 'disabled' : ''}>
        ${state.geoBusy ? 'Locating…' : 'Use my current location'}
      </button>
      ${state.address ? `<p class="picked">📍 ${esc(state.address)}</p>` : ''}
      <button type="submit" ${state.busy ? 'disabled' : ''}>Send request</button>
    </form>`;
}

function sheetBody(state: AppState): string {
  switch (state.sheet) {
    case 'search':
      return searchBody(state);
    case 'fleet':
      return fleetBody(state);
    case 'route':
      return routeBody(state);
    case 'jobs':
      return jobsBody(state);
    case 'request':
      return requestBody(state);
    default:
      return `
        <button class="search" data-sheet="search">
          <span class="search-glass">🔍</span>
          <span>Where to?</span>
          <span class="chip">Later</span>
        </button>
        <h3 class="sheet-title">Let's get you on your way.</h3>
        ${cards(state)}
        <h3 class="sheet-title">Recent</h3>
        ${recent(state)}`;
  }
}

export function homeView(state: AppState): string {
  // A sub-page (fleet, route, request…) always opens full height.
  const snap = state.sheet === 'menu' ? state.snap : 'full';

  return `
    <div class="page home">
      <section class="sheet glass ${snap}">
        <button class="handle" id="toggle-sheet" aria-label="Resize sheet"></button>
        <div class="sheet-scroll">${sheetBody(state)}</div>
      </section>
    </div>`;
}

export function bindHome(root: HTMLElement, actions: Actions): void {
  root.querySelectorAll<HTMLButtonElement>('[data-sheet]').forEach((button) => {
    button.addEventListener('click', () => actions.setSheet(button.dataset.sheet as never));
  });
  root.querySelectorAll<HTMLButtonElement>('[data-open]').forEach((button) => {
    button.addEventListener('click', () => actions.setSheet(button.dataset.open as never));
  });
  root.querySelectorAll<HTMLButtonElement>('[data-pickup]').forEach((button) => {
    button.addEventListener('click', () => actions.focusPickup(button.dataset.pickup ?? ''));
  });
  root.querySelectorAll<HTMLButtonElement>('[data-vehicle]').forEach((button) => {
    button.addEventListener('click', () => actions.focusVehicle(button.dataset.vehicle ?? ''));
  });
  root.querySelectorAll<HTMLButtonElement>('[data-region]').forEach((button) => {
    button.addEventListener('click', () => actions.pickRegion(button.dataset.region ?? ''));
  });
  root.querySelectorAll<HTMLButtonElement>('[data-ward]').forEach((button) => {
    button.addEventListener('click', () =>
      actions.pickWard(Number(button.dataset.lat), Number(button.dataset.lng)),
    );
  });
  root.querySelectorAll<HTMLButtonElement>('[data-place]').forEach((button) => {
    button.addEventListener('click', () => actions.pickPlace(Number(button.dataset.place)));
  });

  root.querySelector<HTMLFormElement>('#address-form')?.addEventListener('submit', (event) => {
    event.preventDefault();
    actions.submitSearch(event.currentTarget as HTMLFormElement);
  });

  root.querySelector<HTMLButtonElement>('#use-location')?.addEventListener('click', () =>
    actions.locate(),
  );

  root.querySelector<HTMLButtonElement>('#toggle-sheet')?.addEventListener('click', () =>
    actions.toggleSheet(),
  );
  root.querySelector<HTMLButtonElement>('#toggle-vehicle')?.addEventListener('click', () =>
    actions.toggleVehicle(),
  );
  root.querySelector<HTMLButtonElement>('#optimize')?.addEventListener('click', () => actions.optimize());
  root.querySelector<HTMLButtonElement>('#new-window')?.addEventListener('click', () =>
    actions.createWindow(),
  );

  root.querySelector<HTMLFormElement>('#create-form')?.addEventListener('submit', (event) => {
    event.preventDefault();
    actions.create(event.currentTarget as HTMLFormElement);
  });
}
