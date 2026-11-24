// Camera capture for the profile photo.
// Tauri v2 has no official camera plugin (only a barcode scanner), so the
// webview's getUserMedia is the portable route on desktop and Android.

let stream: MediaStream | null = null;

export async function startCamera(video: HTMLVideoElement): Promise<void> {
  stopCamera();
  stream = await navigator.mediaDevices.getUserMedia({
    video: { facingMode: 'user', width: { ideal: 640 } },
    audio: false,
  });
  video.srcObject = stream;
  await video.play();
}

export function stopCamera(): void {
  stream?.getTracks().forEach((track) => track.stop());
  stream = null;
}

/** Square JPEG of the current frame, small enough for localStorage. */
export function captureFrame(video: HTMLVideoElement, size = 192): string {
  const canvas = document.createElement('canvas');
  canvas.width = size;
  canvas.height = size;

  const side = Math.min(video.videoWidth, video.videoHeight) || size;
  const sx = Math.max(0, (video.videoWidth - side) / 2);
  const sy = Math.max(0, (video.videoHeight - side) / 2);

  const context = canvas.getContext('2d');
  if (context) {
    context.drawImage(video, sx, sy, side, side, 0, 0, size, size);
  }
  return canvas.toDataURL('image/jpeg', 0.8);
}
