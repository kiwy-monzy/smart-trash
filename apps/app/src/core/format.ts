// Labels come from the proto enums, so they can never drift from the backend.

import { Role } from '../generated/identity';
import { PickupStatus, WasteType } from '../generated/pickup';

function strip(name: string | undefined, prefix: string): string {
  return (name ?? '').replace(prefix, '').toLowerCase().replace(/_/g, ' ');
}

export const roleLabel = (role: Role): string => strip(Role[role], 'ROLE_');
export const statusLabel = (status: PickupStatus): string =>
  strip(PickupStatus[status], 'PICKUP_STATUS_');
export const statusClass = (status: PickupStatus): string =>
  strip(PickupStatus[status], 'PICKUP_STATUS_').replace(/ /g, '_');
export const wasteLabel = (waste: WasteType): string => strip(WasteType[waste], 'WASTE_TYPE_');

export function volume(value: number): string {
  return `${Number(value).toFixed(1)} m³`;
}

/** 15 decimals, as asked for the accuracy requirement. */
export function coordinate(value: number): string {
  return Number(value).toFixed(15);
}

export const WASTE_OPTIONS: { value: string; label: string; type: WasteType }[] = [
  { value: 'recyclable', label: 'recyclable', type: WasteType.RECYCLABLE },
  { value: 'organic', label: 'organic', type: WasteType.ORGANIC },
  { value: 'general', label: 'general', type: WasteType.GENERAL },
  { value: 'hazardous', label: 'hazardous', type: WasteType.HAZARDOUS },
  { value: 'construction', label: 'construction', type: WasteType.CONSTRUCTION },
];
