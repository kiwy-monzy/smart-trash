// Every user who can receive a "truck nearby" alert (council only).

import type { User } from '../generated/identity';
import { IdentityServiceClient } from '../generated/identity.client';
import { getTransport } from './transport';

export async function listUsers(): Promise<User[]> {
  const client = new IdentityServiceClient(getTransport());
  const { response } = await client.listUsers({ pageSize: 200 });
  return response.users;
}
