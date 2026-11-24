// Route optimization.

import type { Route } from '../generated/routing';
import { RoutingServiceClient } from '../generated/routing.client';
import { getTransport } from './transport';

export async function optimizeRoute(
  vehicleId: string,
  pickupIds: string[],
  capacityCubicMetres = 0,
): Promise<Route> {
  const client = new RoutingServiceClient(getTransport());
  const { response } = await client.optimizeRoute({
    vehicleId,
    pickupIds,
    serviceDate: new Date().toISOString().slice(0, 10),
    capacityCubicMetres,
  });
  return response;
}
