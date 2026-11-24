# Smart Trash app — Vite + TypeScript + Tauri 2

A small client for the Smart Trash backend, map-first: the fleet on an
OpenStreetMap canvas with an H3 hexagon per vehicle, a bottom sheet for
pickups, routing and schedules, and role-based tabs.

One codebase for desktop, Android and iOS. The UI is TypeScript, the shell is
Tauri, and **every call is gRPC-Web straight to `smarttrash-server:50051`** —
no REST, no Tauri `invoke`.

## What is on screen

The layout follows the Bolt app: a full-bleed map, a bottom sheet with a
"Where to?" search, and three tabs — **Home · Activity · Account** — with the
active tab as a floating white pill.

| Tab | What |
| --- | --- |
| Home | map + sheet: search, four quick cards (request, fleet, areas, route/schedule for council) and the last pickups |
| Activity | pickups and alerts, switched with a segmented control |
| Account | full page: avatar, role, rating, "signed in securely" card, rows with chevrons (endpoint, alerts, profile) and sign out |

The map is clamped to Tanzania (`maxBounds`, min zoom 6) and opens on Dar es
Salaam. "Areas" lists all 31 regions from `GeoService.ListRegions`; only Dar es
Salaam returns wards (`ListWards`), the other 30 say so in the sheet.

A driver still gets **Go on duty / Go off duty**, which calls
`FleetService.SetVehicleState` on its own truck (`fleet.activate`); the server
refuses that for any other vehicle.

Map markers: green = on route, blue = available, grey = driver off, amber =
maintenance. The H3 hexagon follows the same colour; council users also see a
dashed blue hexagon per registered user.

> The base map is **not** tiles: it is the Dar es Salaam extract in
> `public/geo/`, drawn by `src/core/basemap.ts` — the region outline, five
> municipality boundaries, 94 ward polygons, 747 rivers, 1 973 major roads,
> 76 729 minor streets and the waste sites. It therefore works offline and
> stays sharp at any zoom. Ward names appear from zoom 12, minor streets from
> zoom 10. OSM attribution (ODbL) is in the Account footer — keep it.
>
> OSM has no district relations for Dar es Salaam, so the five district polygons
> are unioned from their wards; ward outlines that come from the OCHA COD-AB 2018
> shapefile are dashed, and the eight wards with no published polygon at all show
> as a small anchor circle. Only 11 waste sites are mapped as an area (the rest
> are single OSM nodes), so those 11 are drawn as polygons and the others as
> markers.
>
> The Areas sheet reads the same `wards.geojson`: 102 wards grouped by
> municipality, each row centring the map.
>
> Regenerate with `scripts/fetch-map-data.ps1`, `scripts/fetch-rivers.py` and
> `python scripts/prepare-map.py`; see the top-level README.

## How the types stay in sync

```
proto/*.proto
   ├── prost (Rust)          → the services and their handlers
   └── @protobuf-ts (TS)     → src/generated/*.ts, used by the UI
```

One source of truth: change a `.proto`, run `npm run proto`, and the frontend
types change with the backend. `src/generated/` is produced by protoc, never
edited by hand.

## Commands

```powershell
npm install

npm run app          # one command: backend + UI build + app window
npm run proto        # regenerate src/generated from ../../proto
npm run typecheck    # tsc --noEmit (strict)
npm run probe        # login + fleet + windows over gRPC-Web, headless
npm run pentest      # 103 auth/identity/transport probes (see below)
npm run dev          # Vite only (browser, port 4180)
npm run desktop:dev  # Tauri window with hot reload
npm run desktop:build
```

`npm run app` calls `../scripts/run-app.ps1`, which starts `smarttrash-server`
when port 50051 is closed, builds `dist/`, then opens the release binary
(building it the first time).

### Ports on this machine

Windows reserves `8080` and `5173` (Hyper-V/WinNAT), so:

| What | Port | Where it is set |
| --- | --- | --- |
| gRPC / gRPC-Web | 50051 | `.env` → `RPC_ADDR` |
| REST gateway (prototypes) | 8081 | `GATEWAY_ADDR` when starting it |
| Vite dev server | 4180 | `vite.config.ts` + `tauri.dev.conf.json` |

### The Vite proxy

In development every gRPC-Web call goes to `/smarttrash.<pkg>.<Service>/<Method>`
on the Vite origin and is proxied to the RPC server — the backend address is
written once, in `vite.config.ts`:

```ts
proxy: { '/smarttrash': { target: process.env.VITE_RPC_URL ?? 'http://127.0.0.1:50051' } }
```

A packaged build has no dev server, so it uses the endpoint stored in the app
(default `http://127.0.0.1:50051`).

## Layout

```
src/
├── core/
│   ├── config.ts      endpoint address (default 127.0.0.1:50051, 10.0.2.2 on Android)
│   ├── transport.ts   gRPC-Web transport + bearer-token interceptor
│   ├── session.ts     login / register / logout / expiry
│   ├── pickups.ts     pickup RPCs
│   ├── fleet.ts       vehicles + on/off duty
│   ├── route.ts       route optimization
│   ├── schedule.ts    collection windows
│   ├── regions.ts     regions + wards
│   ├── map.ts         Leaflet map, H3 hexes, pins, route line, Tanzania bounds
│   ├── format.ts      labels from the proto enums, 15-decimal coordinates
│   ├── platform.ts    desktop / android / ios detection
│   └── html.ts        escaping for templates
├── views/
│   ├── login.ts       sign in + create account
│   ├── home.ts        map sheet: search, quick cards, fleet, route, schedule
│   ├── activity.ts    pickups + alerts
│   ├── account.ts     the Bolt-style account page
│   ├── tabbar.ts      Home / Activity / Account
│   └── icons.ts       inline SVG
├── generated/         protoc output (do not edit)
├── app.ts             state, tabs, actions, render
├── main.ts            mounts the map, then boots
└── style.css          glass sheet + Bolt layout
src-tauri/             Tauri shell (Rust), window + icons + config
scripts/probe.ts       headless gRPC-Web check
scripts/pentest.mjs    103 auth/identity/transport probes
```

### Layout

* Phone: the map fills the window, a glass HUD floats on top and the sheet is
  docked at the bottom above the home-indicator inset
  (`env(safe-area-inset-bottom)`). Tapping the handle lifts it from 42 vh to
  74 vh; the body scrolls inside.
* Desktop (≥760 px): same map, the sheet is a 520 px card docked bottom-centre,
  the pickup form lays out 4-up.

The sheet tabs are role-based: everyone gets **Fleet · Pickups · Account**;
council also gets **Route · Schedules**.

## Sign-in and registration

`AuthService.Login` and `AuthService.Register` are the only RPCs served without
a token. Registering takes a name, a role, a ward and a position ("Use my
current location" calls the Tauri geolocation plugin on Android/iOS and
`navigator.geolocation` elsewhere), creates the profile in the shared user
directory and returns a session straight away.

The first window a user sees is an in-app splash (ST mark, progress bar) that
fades after the first render; Android also gets the system launch window before
the WebView paints.

### Plugins

| Plugin | Why |
| --- | --- |
| `tauri-plugin-geolocation` | position for registration, pickup requests and "near me" |
| webview camera (`getUserMedia`) | profile photo; Tauri v2 has no official camera plugin (only a barcode scanner) |

Geolocation needs the permissions in `src-tauri/capabilities/default.json`;
they are already granted for `location` and `coarseLocation`.

## Alerts

When a driver reports a GPS ping, the tracking service looks up every user
whose H3 cell is inside `grid_disk(cell, 1)` and writes a "Truck nearby"
notification for them. The app polls it and shows an **Alerts** tab with the
count in the HUD; a council user also sees everyone else's H3 cell on the map
(dashed blue hexagons) through `IdentityService.ListUsers`.

## Security sweep

```powershell
npm run pentest        # 103 probes, exit code 1 on a finding
```

Black-box, from the outside, against `:50051`: missing token on all 10
protected methods, 25 forged/malformed tokens (role swap, expiry push, JWT,
base64, unicode, 8 kB, …), 11 role-escalation attempts, 30 injection payloads
(SQL, shell, path traversal, null bytes), oversized/NaN input, and framing abuse
(GET, wrong content-type, truncated frame, huge declared length, unknown
method, reflection without token). It is a behaviour test, not a CVE scanner —
run `cargo audit` separately for dependency CVEs.

## Desktop, Android, iOS

```powershell
npm run desktop:build    # installers in src-tauri/target/release/bundle
```

Android (JDK 17+, SDK + NDK, `rustup target add aarch64-linux-android ...`):

```powershell
npm run android:init     # once, creates src-tauri/gen/android
npm run android:dev
```

iOS (macOS + Xcode 15+, `rustup target add aarch64-apple-ios`):

```bash
npm run ios:init
npm run ios:dev
```

Mobile notes:

* The app talks to `:50051`; an emulator reaches the host on `10.0.2.2`
  (already the default), a phone needs the PC's LAN address in the endpoint
  field and `RPC_ADDR=0.0.0.0:50051` on the server.
* Android blocks cleartext HTTP: add `android:usesCleartextTraffic="true"` to
  the generated `AndroidManifest.xml` for local development.
* iOS needs `NSAllowsLocalNetworking` in the generated `Info.plist`.
* Production: terminate TLS in front of `:50051` and set
  `GRPC_REFLECTION=false`.

## Why the debug window no longer asks for localhost:5173

`devUrl` lives in `src-tauri/tauri.dev.conf.json`, which only
`npm run desktop:dev` merges in. A plain `cargo build` (debug or release)
loads the bundled `dist/`, so the window always shows the app.
