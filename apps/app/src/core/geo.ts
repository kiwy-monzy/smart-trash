// Current position: the Tauri plugin on Android/iOS, the browser API elsewhere.

import {
  checkPermissions,
  getCurrentPosition,
  requestPermissions,
} from '@tauri-apps/plugin-geolocation';

export interface Fix {
  latitude: number;
  longitude: number;
  accuracy: number;
}

async function fromPlugin(): Promise<Fix | null> {
  try {
    let status = await checkPermissions();
    if (status.location !== 'granted') {
      status = await requestPermissions(['location']);
    }
    if (status.location !== 'granted') return null;

    const position = await getCurrentPosition({ enableHighAccuracy: true, maximumAge: 0, timeout: 10_000 });
    return {
      latitude: position.coords.latitude,
      longitude: position.coords.longitude,
      accuracy: position.coords.accuracy,
    };
  } catch {
    // Desktop has no native provider; fall through to the web API.
    return null;
  }
}

function fromBrowser(): Promise<Fix> {
  return new Promise((resolve, reject) => {
    if (!navigator.geolocation) {
      reject(new Error('this device has no location provider'));
      return;
    }
    navigator.geolocation.getCurrentPosition(
      (position) =>
        resolve({
          latitude: position.coords.latitude,
          longitude: position.coords.longitude,
          accuracy: position.coords.accuracy,
        }),
      (error) => reject(new Error(error.message || 'could not read the location')),
      { enableHighAccuracy: true, timeout: 10_000, maximumAge: 0 },
    );
  });
}

export async function currentFix(): Promise<Fix> {
  return (await fromPlugin()) ?? fromBrowser();
}
