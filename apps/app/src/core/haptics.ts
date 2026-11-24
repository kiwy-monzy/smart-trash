// Short vibration on Android/iOS. Does nothing on desktop.

import { vibrate } from '@tauri-apps/plugin-haptics';

export async function tap(ms = 8): Promise<void> {
  try {
    await vibrate(ms);
  } catch {
    // no motor, or the plugin is unavailable on this platform
  }
}
