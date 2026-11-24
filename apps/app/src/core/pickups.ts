// The only place pickup RPCs are called from.

import type { CreatePickupRequest, Pickup } from '../generated/pickup';
import { PickupStatus } from '../generated/pickup';
import { PickupServiceClient } from '../generated/pickup.client';
import { getTransport } from './transport';

export async function listPickups(): Promise<Pickup[]> {
  const client = new PickupServiceClient(getTransport());
  const { response } = await client.listPickups({
    pageSize: 50,
    statusFilter: PickupStatus.UNSPECIFIED,
  });
  return response.pickups;
}

export async function createPickup(request: CreatePickupRequest): Promise<Pickup> {
  const client = new PickupServiceClient(getTransport());
  const { response } = await client.createPickup(request);
  return response;
}
