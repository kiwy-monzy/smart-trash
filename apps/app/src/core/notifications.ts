// "Truck nearby" alerts.

import type { Notification } from '../generated/notification';
import { NotificationServiceClient } from '../generated/notification.client';
import { getTransport } from './transport';

export async function listNotifications(): Promise<Notification[]> {
  const client = new NotificationServiceClient(getTransport());
  const { response } = await client.listNotifications({ limit: 20 });
  return response.notifications;
}
