// Collection windows (schedules).

import type { CollectionWindow } from '../generated/scheduling';
import { SchedulingServiceClient } from '../generated/scheduling.client';
import { getTransport } from './transport';

export async function listWindows(): Promise<CollectionWindow[]> {
  const client = new SchedulingServiceClient(getTransport());
  const { response } = await client.listWindows({});
  return response.windows;
}

export async function createWindow(
  zone: string,
  startUnix: number,
  endUnix: number,
  capacityLimit: number,
): Promise<CollectionWindow> {
  const client = new SchedulingServiceClient(getTransport());
  const { response } = await client.createWindow({
    zone,
    startUnix: String(startUnix),
    endUnix: String(endUnix),
    capacityLimit,
  });
  return response;
}
