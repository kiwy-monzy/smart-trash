// State, actions and rendering.

import { RpcError } from '@protobuf-ts/runtime-rpc';

import { captureFrame, startCamera, stopCamera } from './core/camera';
import { rpcUrl, saveRpcUrl } from './core/config';
import { listVehicles, setVehicleState } from './core/fleet';
import { WASTE_OPTIONS } from './core/format';
import { currentFix } from './core/geo';
import { describePoint, searchAddress } from './core/geocode';
import type { Place } from './core/geocode';
import { tap } from './core/haptics';
import { focusMap, renderMap } from './core/map';
import { listNotifications } from './core/notifications';
import { createPickup, listPickups } from './core/pickups';
import { attachPullToRefresh } from './core/pull';
import { DEFAULT_REGION, listRegions, listWards } from './core/regions';
import { listAreas } from './core/wards';
import type { WardArea } from './core/wards';
import { createWindow, listWindows } from './core/schedule';
import { optimizeRoute } from './core/route';
import { isExpired, login, logout, register, restore, session } from './core/session';
import type { Session } from './core/session';
import { resetTransport } from './core/transport';
import { listUsers } from './core/users';
import type { Vehicle } from './generated/fleet';
import { VehicleState } from './generated/fleet';
import type { Region, Ward } from './generated/geo';
import { Role } from './generated/identity';
import type { User } from './generated/identity';
import type { Notification } from './generated/notification';
import type { CreatePickupRequest, Pickup } from './generated/pickup';
import { PickupStatus } from './generated/pickup';
import type { Route } from './generated/routing';
import type { CollectionWindow } from './generated/scheduling';
import { bindActivity, activityView } from './views/activity';
import { bindAccount, accountView } from './views/account';
import { bindHome, homeView } from './views/home';
import { bindLogin, loginView } from './views/login';
import { renderTabBar } from './views/tabbar';

export type TabId = 'home' | 'activity' | 'account';
export type SheetId = 'menu' | 'search' | 'fleet' | 'route' | 'jobs' | 'request';
export type ActivityTab = 'pickups' | 'alerts';
export type PanelId = 'profile' | 'endpoint' | null;
export type SheetSnap = 'peek' | 'half' | 'full';
export type AuthMode = 'signin' | 'register';

export interface AppState {
  session: Session | null;
  pickups: Pickup[];
  vehicles: Vehicle[];
  users: User[];
  windows: CollectionWindow[];
  notifications: Notification[];
  regions: Region[];
  wards: Ward[];
  areas: WardArea[];
  regionCode: string;
  query: string;
  places: Place[];
  address: string;
  route: Route | null;
  tab: TabId;
  sheet: SheetId;
  snap: SheetSnap;
  activity: ActivityTab;
  accountPanel: PanelId;
  busy: boolean;
  geoBusy: boolean;
  cameraOn: boolean;
  avatar: string;
  authMode: AuthMode;
  error: string;
  notice: string;
  rpcUrl: string;
}

export interface Actions {
  signIn(userId: string): void;
  signOut(): void;
  register(form: HTMLFormElement): void;
  setAuthMode(mode: AuthMode): void;
  locate(): void;
  startCamera(): void;
  stopCamera(): void;
  attachCamera(video: HTMLVideoElement): void;
  capturePhoto(): void;
  refresh(): void;
  create(form: HTMLFormElement): void;
  saveServer(url: string): void;
  setTab(tab: TabId): void;
  setSheet(sheet: SheetId): void;
  toggleSheet(): void;
  setActivity(tab: ActivityTab): void;
  togglePanel(panel: Exclude<PanelId, null>): void;
  pickRegion(code: string): void;
  pickWard(lat: number, lng: number): void;
  submitSearch(form: HTMLFormElement): void;
  pickPlace(index: number): void;
  focusVehicle(vehicleId: string): void;
  focusPickup(pickupId: string): void;
  toggleVehicle(): void;
  optimize(): void;
  createWindow(): void;
}

const REFRESH_MS = 20_000;
const AVATAR_KEY = 'smarttrash.avatar';

const state: AppState = {
  session: null,
  pickups: [],
  vehicles: [],
  users: [],
  windows: [],
  notifications: [],
  regions: [],
  wards: [],
  areas: [],
  regionCode: DEFAULT_REGION,
  query: '',
  places: [],
  address: '',
  route: null,
  tab: 'home',
  sheet: 'menu',
  snap: 'half',
  activity: 'pickups',
  accountPanel: null,
  busy: false,
  geoBusy: false,
  cameraOn: false,
  avatar: localStorage.getItem(AVATAR_KEY) ?? '',
  authMode: 'signin',
  error: '',
  notice: '',
  rpcUrl: rpcUrl(),
};

const root = document.getElementById('app') as HTMLElement;
const tabbarEl = document.getElementById('tabbar') as HTMLElement;
let timer: number | undefined;
let expiryTimer: number | undefined;
let lastFix: { latitude: number; longitude: number } | null = null;

// The bar lives outside #app, so bind it once and let the pill animate.
tabbarEl.addEventListener('click', (event) => {
  const button = (event.target as HTMLElement).closest<HTMLButtonElement>('[data-tab]');
  if (!button?.dataset.tab) return;
  void tap(8);
  buildActions().setTab(button.dataset.tab as TabId);
});

function message(error: unknown): string {
  const text = error instanceof Error ? error.message : String(error);

  if (/failed to fetch|networkerror|load failed|network request failed/i.test(text)) {
    return `Cannot reach the server at ${state.rpcUrl} ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â is smarttrash-server running?`;
  }
  return text;
}

function isUnauthenticated(error: unknown): boolean {
  return error instanceof RpcError && error.code === 'UNAUTHENTICATED';
}

export function render(): void {
  if (!state.session) {
    root.innerHTML = loginView(state);
    bindLogin(root, buildActions());
    tabbarEl.innerHTML = '';
    window.clearTimeout(timer);
    // Do not leave yesterday's markers on the map behind the login card.
    renderMap({ vehicles: [], pickups: [], users: [], route: null });
    return;
  }

  const page =
    state.tab === 'activity'
      ? activityView(state)
      : state.tab === 'account'
        ? accountView(state)
        : homeView(state);

  root.innerHTML = `<div class="shell">${page}</div>`;
  renderTabBar(tabbarEl, state.tab, state.notifications.length);

  const actions = buildActions();
  if (state.tab === 'activity') bindActivity(root, actions);
  else if (state.tab === 'account') bindAccount(root, actions);
  else bindHome(root, actions);

  // The bottom tabs live outside the page views, so bind them here.
  root.querySelectorAll<HTMLButtonElement>('[data-tab]').forEach((button) => {
    button.addEventListener('click', () => {
      const activity = button.dataset.activity;
      if (activity === 'alerts' || activity === 'pickups') state.activity = activity;
      void tap(8);
      actions.setTab(button.dataset.tab as TabId);
    });
  });

  attachPullToRefresh(root.querySelector('.sheet-scroll'), actions.refresh);
  attachPullToRefresh(root.querySelector('.page.plain'), actions.refresh);

  if (state.tab === 'home') {
    renderMap({
      vehicles: state.vehicles,
      pickups: state.pickups,
      users: state.users,
      route: state.route,
    });
  }

  window.clearTimeout(timer);
  // Only refresh while the sheet is at rest, so a half-filled form is never wiped.
  timer = window.setTimeout(() => {
    if (state.sheet === 'menu') void refresh();
  }, REFRESH_MS);
  scheduleExpiry();
}

function scheduleExpiry(): void {
  window.clearTimeout(expiryTimer);
  if (!state.session) return;

  const remaining = Number(state.session.expiresAtUnix) * 1000 - Date.now();
  if (remaining <= 0) {
    expireSession();
    return;
  }
  expiryTimer = window.setTimeout(expireSession, remaining);
}

function expireSession(): void {
  if (!state.session) return;
  logout();
  state.session = null;
  state.pickups = [];
  state.vehicles = [];
  state.notifications = [];
  state.route = null;
  state.error = '';
  state.notice = 'Session expired ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â sign in again';
  state.tab = 'home';
  state.authMode = 'signin';
  render();
}

function ownVehicle(): Vehicle | undefined {
  return state.vehicles.find((v) => v.driverId === state.session?.userId);
}

function buildActions(): Actions {
  return {
    signIn: (userId) => void signIn(userId),
    signOut: () => {
      logout();
      stopCamera();
      Object.assign(state, {
        session: null,
        pickups: [],
        vehicles: [],
        users: [],
        windows: [],
        notifications: [],
        route: null,
        error: '',
        notice: '',
        tab: 'home',
        sheet: 'menu',
        authMode: 'signin',
        cameraOn: false,
      });
      render();
    },
    register: (form) => void registerAccount(form),
    setAuthMode: (mode) => {
      state.authMode = mode;
      state.error = '';
      state.notice = '';
      state.cameraOn = false;
      stopCamera();
      render();
    },
    locate: () => void locate(),
    startCamera: () => {
      state.cameraOn = true;
      state.error = '';
      render();
    },
    stopCamera: () => {
      stopCamera();
      state.cameraOn = false;
      render();
    },
    attachCamera: (video) => {
      void startCamera(video).catch((error: unknown) => {
        state.cameraOn = false;
        state.error = message(error);
        render();
      });
    },
    capturePhoto: () => {
      const video = document.getElementById('cam') as HTMLVideoElement | null;
      if (!video) return;
      state.avatar = captureFrame(video);
      localStorage.setItem(AVATAR_KEY, state.avatar);
      stopCamera();
      state.cameraOn = false;
      state.notice = 'Photo saved';
      render();
    },
    refresh: () => void refresh(),
    create: (form) => void createFromForm(form),
    saveServer: (url) => {
      saveRpcUrl(url);
      state.rpcUrl = rpcUrl();
      resetTransport();
      state.notice = 'Endpoint saved';
      render();
      void refresh();
    },
    setTab: (tab) => {
      state.tab = tab;
      state.sheet = 'menu';
      render();
    },
    setSheet: (sheet) => {
      state.sheet = sheet;
      state.snap = 'half';
      render();

      // Opening "Areas" should already list the wards of the current region.
      if (sheet === 'search' && !state.wards.length && !state.areas.length) {
        void pickRegion(state.regionCode);
      }
    },
    toggleSheet: () => {
      // Peek ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ half ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ full ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ peek, like the Bolt sheet.
      if (state.sheet !== 'menu') {
        state.sheet = 'menu';
        state.snap = 'half';
      } else {
        state.snap = state.snap === 'peek' ? 'half' : state.snap === 'half' ? 'full' : 'peek';
      }
      render();
    },
    setActivity: (tab) => {
      state.activity = tab;
      render();
    },
    togglePanel: (panel) => {
      state.accountPanel = state.accountPanel === panel ? null : panel;
      render();
    },
    pickRegion: (code) => void pickRegion(code),
    pickWard: (lat, lng) => {
      focusMap(lat, lng, 14);
      state.sheet = 'menu';
      render();
      void describePoint(lat, lng).then((label) => {
        if (label) {
          state.address = label;
          state.notice = label.split(',').slice(0, 2).join(',');
          render();
        }
      });
    },
    submitSearch: (form) => void runSearch(form),
    pickPlace: (index) => {
      const place = state.places[index];
      if (!place) return;
      state.address = place.label;
      focusMap(place.latitude, place.longitude, 16);
      render();
    },
    focusVehicle: (vehicleId) => {
      const v = state.vehicles.find((x) => x.vehicleId === vehicleId);
      if (v) focusMap(v.latitude, v.longitude, 15);
    },
    focusPickup: (pickupId) => {
      const p = state.pickups.find((x) => x.pickupId === pickupId);
      if (p) {
        state.tab = 'home';
        focusMap(p.latitude, p.longitude, 16);
        render();
      }
    },
    toggleVehicle: () => void toggleOwnVehicle(),
    optimize: () => void optimize(),
    createWindow: () => void newWindow(),
  };
}

async function pickRegion(code: string): Promise<void> {
  state.regionCode = code;
  state.busy = true;
  render();
  try {
    state.wards = await listWards(code);
    // Dar es Salaam has the full official ward set on disk; other regions do not.
    state.areas = code === DEFAULT_REGION ? await listAreas() : [];
  } catch {
    state.wards = [];
    state.areas = [];
  } finally {
    state.busy = false;
    render();
  }
}

async function signIn(userId: string): Promise<void> {
  state.busy = true;
  state.error = '';
  state.notice = '';
  render();

  try {
    state.session = await login(userId);
    state.tab = 'home';
    // A session means the auth screen is gone; do not leave the camera running.
    stopCamera();
    state.cameraOn = false;
    await refresh();
  } catch (error) {
    state.error = message(error);
  } finally {
    state.busy = false;
    render();
  }
}

async function registerAccount(form: HTMLFormElement): Promise<void> {
  const data = new FormData(form);
  state.busy = true;
  state.error = '';
  render();

  try {
    state.session = await register({
      displayName: String(data.get('display_name') ?? ''),
      role: Number(data.get('role') ?? Role.HOUSEHOLD),
      ward: String(data.get('ward') ?? ''),
      latitude: Number(data.get('latitude')),
      longitude: Number(data.get('longitude')),
    });
    stopCamera();
    state.cameraOn = false;
    state.tab = 'home';
    state.notice = `Welcome ${state.session.userId}`;
    await refresh();
  } catch (error) {
    state.error = message(error);
  } finally {
    state.busy = false;
    render();
  }
}

async function locate(): Promise<void> {
  state.geoBusy = true;
  state.error = '';
  render();

  try {
    const fix = await currentFix();
    lastFix = { latitude: fix.latitude, longitude: fix.longitude };
    state.address = await describePoint(fix.latitude, fix.longitude);
    state.notice = `Location set (Ãƒâ€šÃ‚Â±${Math.round(fix.accuracy)} m)`;
  } catch (error) {
    state.error = message(error);
  } finally {
    state.geoBusy = false;
    render();

    if (lastFix) {
      for (const form of ['#register-form', '#create-form']) {
        const lat = document.querySelector<HTMLInputElement>(`${form} [name="latitude"]`);
        const lng = document.querySelector<HTMLInputElement>(`${form} [name="longitude"]`);
        if (lat && lng) {
          lat.value = lastFix.latitude.toFixed(6);
          lng.value = lastFix.longitude.toFixed(6);
        }
      }
    }
  }
}

async function runSearch(form: HTMLFormElement): Promise<void> {
  const data = new FormData(form);
  const query = String(data.get('q') ?? '').trim();
  if (!query) return;

  state.query = query;
  state.geoBusy = true;
  state.error = '';
  render();

  try {
    state.places = await searchAddress(query);
    if (!state.places.length) state.notice = 'No match in Tanzania for that address';
  } catch (error) {
    state.error = message(error);
  } finally {
    state.geoBusy = false;
    render();
  }
}

async function refresh(): Promise<void> {
  if (!state.session) return;

  if (isExpired(state.session)) {
    expireSession();
    return;
  }

  state.busy = true;
  render();
  try {
    const [pickups, vehicles, windows, notifications, users, regions] = await Promise.all([
      listPickups(),
      listVehicles().catch(() => [] as Vehicle[]),
      listWindows().catch(() => [] as CollectionWindow[]),
      listNotifications().catch(() => [] as Notification[]),
      state.session.role === Role.COUNCIL ? listUsers().catch(() => [] as User[]) : Promise.resolve([]),
      state.regions.length ? Promise.resolve(state.regions) : listRegions().catch(() => [] as Region[]),
    ]);

    state.pickups = pickups;
    state.vehicles = vehicles;
    state.windows = windows;
    state.notifications = notifications;
    state.users = users;
    state.regions = regions;
    state.error = '';
  } catch (error) {
    state.error = message(error);
    if (isUnauthenticated(error)) {
      logout();
      state.session = null;
      state.pickups = [];
      state.vehicles = [];
    }
  } finally {
    state.busy = false;
    render();
  }
}

async function createFromForm(form: HTMLFormElement): Promise<void> {
  const user = state.session;
  if (!user) return;

  const data = new FormData(form);
  const waste = WASTE_OPTIONS.find((o) => o.value === String(data.get('waste_type')));

  const request: CreatePickupRequest = {
    customerId: user.userId,
    wasteType: waste ? waste.type : WASTE_OPTIONS[0].type,
    latitude: Number(data.get('latitude')),
    longitude: Number(data.get('longitude')),
    estimatedVolume: Number(data.get('estimated_volume')),
  };

  state.busy = true;
  state.error = '';
  render();
  try {
    const pickup = await createPickup(request);
    state.notice = `${pickup.pickupId} requested`;
    state.sheet = 'menu';
    await refresh();
  } catch (error) {
    state.error = message(error);
    state.busy = false;
    render();
  }
}

async function toggleOwnVehicle(): Promise<void> {
  const vehicle = ownVehicle();
  if (!vehicle) {
    state.error = 'No vehicle is assigned to you';
    render();
    return;
  }

  const next = vehicle.activated ? VehicleState.OUT_OF_SERVICE : VehicleState.AVAILABLE;
  state.busy = true;
  render();
  try {
    await setVehicleState(vehicle.vehicleId, next);
    state.notice = `${vehicle.vehicleId} ${next === VehicleState.AVAILABLE ? 'on duty' : 'off duty'}`;
    await refresh();
  } catch (error) {
    state.error = message(error);
    state.busy = false;
    render();
  }
}

async function optimize(): Promise<void> {
  const vehicle = state.vehicles[0];
  const pending = state.pickups
    .filter((p) => p.status === PickupStatus.PENDING || p.status === PickupStatus.SCHEDULED)
    .map((p) => p.pickupId)
    .slice(0, 20);

  if (!vehicle || !pending.length) {
    state.error = 'Need a vehicle and at least one pending pickup';
    render();
    return;
  }

  state.busy = true;
  render();
  try {
    state.route = await optimizeRoute(vehicle.vehicleId, pending, vehicle.capacityCubicMetres);
    state.error = '';
    state.notice = `${state.route.stops.length} stops planned`;
  } catch (error) {
    state.error = message(error);
  } finally {
    state.busy = false;
    render();
  }
}

async function newWindow(): Promise<void> {
  const start = new Date();
  start.setDate(start.getDate() + 1);
  start.setHours(8, 0, 0, 0);
  const end = new Date(start);
  end.setHours(12, 0, 0, 0);

  state.busy = true;
  render();
  try {
    await createWindow('Kinondoni', Math.floor(start.getTime() / 1000), Math.floor(end.getTime() / 1000), 20);
    state.notice = 'Collection window created';
    await refresh();
  } catch (error) {
    state.error = message(error);
    state.busy = false;
    render();
  }
}

export function boot(): void {
  state.session = restore() ?? session();
  render();
  if (state.session) void refresh();
}
