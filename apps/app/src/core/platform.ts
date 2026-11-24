// Which shell are we running in? Used for the default endpoint and the badge.

const ua = navigator.userAgent;

export const isAndroid = /Android/i.test(ua);
export const isIOS = /iPhone|iPad|iPod/i.test(ua);

export const platform = isAndroid ? 'android' : isIOS ? 'ios' : 'desktop';
