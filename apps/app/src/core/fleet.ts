// Fleet data access.

import type { Vehicle } from '../generated/fleet';
import { VehicleState } from '../generated/fleet';
import { FleetServiceClient } from '../generated/fleet.client';
import { getTransport } from './transport';

export async function listVehicles(): Promise<Vehicle[]> {
  const client = new FleetServiceClient(getTransport());
  const { response } = await client.listVehicles({
    stateFilter: VehicleState.UNSPECIFIED,
    pageSize: 50,
  });
  return response.vehicles;
}

/** A driver switches its own truck on or off for the day. */
export async function setVehicleState(
  vehicleId: string,
  state: VehicleState,
): Promise<Vehicle> {
  const client = new FleetServiceClient(getTransport());
  const { response } = await client.setVehicleState({ vehicleId, state });
  return response;
}
