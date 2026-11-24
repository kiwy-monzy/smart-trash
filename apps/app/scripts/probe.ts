// Checks the gRPC-Web path the app uses, without opening a window.
//
//   npm run probe [-- http://127.0.0.1:50051]

import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import { AuthServiceClient } from '../src/generated/auth.client';
import { FleetServiceClient } from '../src/generated/fleet.client';
import { VehicleState } from '../src/generated/fleet';
import { GeoServiceClient } from '../src/generated/geo.client';
import { PickupServiceClient } from '../src/generated/pickup.client';
import { PickupStatus } from '../src/generated/pickup';
import { SchedulingServiceClient } from '../src/generated/scheduling.client';

const baseUrl = process.argv[2] ?? 'http://127.0.0.1:50051';
const transport = new GrpcWebFetchTransport({ baseUrl, format: 'binary' });

const auth = new AuthServiceClient(transport);
const { response: session } = await auth.login({ userId: 'COUNCIL-01' });
console.log(`login    ok  user=${session.userId} role=${session.role}`);

const meta = { authorization: `Bearer ${session.token}` };

const pickups = new PickupServiceClient(transport);
const list = await pickups.listPickups({ pageSize: 5, statusFilter: PickupStatus.UNSPECIFIED }, { meta });
console.log(`pickups  ok  ${list.response.pickups.length} row(s)`);

const fleet = new FleetServiceClient(transport);
const vehicles = await fleet.listVehicles(
  { stateFilter: VehicleState.UNSPECIFIED, pageSize: 10 },
  { meta },
);
console.log(`fleet    ok  ${vehicles.response.vehicles.length} vehicle(s)`);
for (const vehicle of vehicles.response.vehicles) {
  console.log(
    `         ${vehicle.vehicleId} ${vehicle.registrationPlate} state=${vehicle.state} active=${vehicle.activated} cell=${vehicle.h3Cell}`,
  );
}

const scheduling = new SchedulingServiceClient(transport);
const windows = await scheduling.listWindows({}, { meta });
console.log(`windows  ok  ${windows.response.windows.length} window(s)`);

const geo = new GeoServiceClient(transport);
const regions = await geo.listRegions({}, { meta });
const dsm = await geo.listWards({ regionCode: 'DSM' }, { meta });
const arusha = await geo.listWards({ regionCode: 'ARU' }, { meta });
console.log(
  `geo      ok  ${regions.response.regions.length} regions · DSM ${dsm.response.wards.length} wards · ARU ${arusha.response.wards.length} wards`,
);

// The driver switches its own vehicle off, then on again.
const driver = await auth.login({ userId: 'DRIVER-07' });
const driverMeta = { authorization: `Bearer ${driver.response.token}` };
const off = await fleet.setVehicleState(
  { vehicleId: 'VEH-002', state: VehicleState.OUT_OF_SERVICE },
  { meta: driverMeta },
);
console.log(`driver   ok  ${off.response.vehicleId} active=${off.response.activated}`);

const on = await fleet.setVehicleState(
  { vehicleId: 'VEH-002', state: VehicleState.AVAILABLE },
  { meta: driverMeta },
);
console.log(`driver   ok  ${on.response.vehicleId} active=${on.response.activated}`);

// A driver must not touch another truck.
try {
  await fleet.setVehicleState({ vehicleId: 'VEH-001', state: VehicleState.AVAILABLE }, { meta: driverMeta });
  console.log('guard    FAILED (driver changed another vehicle)');
  process.exitCode = 1;
} catch (error) {
  console.log(`guard    ok  ${(error as { code?: string }).code}`);
}
