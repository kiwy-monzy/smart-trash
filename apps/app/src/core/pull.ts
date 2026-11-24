// Pull-to-refresh for a scroll container (phone gesture, ignored on desktop).

export function attachPullToRefresh(element: HTMLElement | null, onRefresh: () => void): void {
  if (!element) return;

  let start = 0;
  let distance = 0;
  let pulling = false;

  element.addEventListener(
    'touchstart',
    (event) => {
      if (element.scrollTop <= 0 && event.touches.length === 1) {
        start = event.touches[0].clientY;
        distance = 0;
        pulling = true;
      }
    },
    { passive: true },
  );

  element.addEventListener(
    'touchmove',
    (event) => {
      if (!pulling) return;
      distance = event.touches[0].clientY - start;
      if (distance > 0) {
        element.style.transform = `translateY(${Math.min(distance * 0.4, 56)}px)`;
      }
    },
    { passive: true },
  );

  element.addEventListener('touchend', () => {
    if (!pulling) return;
    pulling = false;
    element.style.transform = '';

    if (distance > 70) onRefresh();
    distance = 0;
  });
}
