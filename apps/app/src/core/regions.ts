// Tanzania regions and Dar es Salaam wards.

import type { Region, Ward } from '../generated/geo';
import { GeoServiceClient } from '../generated/geo.client';
import { getTransport } from './transport';

export async function listRegions(): Promise<Region[]> {
  const client = new GeoServiceClient(getTransport());
  const { response } = await client.listRegions({});
  return response.regions;
}

export async function listWards(regionCode: string): Promise<Ward[]> {
  const client = new GeoServiceClient(getTransport());
  const { response } = await client.listWards({ regionCode });
  return response.wards;
}

export const DEFAULT_REGION = 'DSM';
