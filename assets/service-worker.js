const CACHE_NAME = "PULSE_PWA_CACHE_V2";
const ASSETS_TO_CACHE = [];

const preload = async () => {
  console.log("[SW] Installing Pulse PWA Service Worker");
  return await caches.open(CACHE_NAME)
    .then(async (cache) => {
      try {
        const response = await fetch("/asset-manifest.json");
        const assets = await response.json();
        ASSETS_TO_CACHE.push(...assets);
        console.log("[SW] Caching assets:", ASSETS_TO_CACHE);
        return cache.addAll(ASSETS_TO_CACHE);
      } catch (e) {
        console.error("[SW] Preload caching failed:", e);
      }
    });
}

globalThis.addEventListener("install", (event) => {
  event.waitUntil(preload().then(() => globalThis.skipWaiting()));
});

globalThis.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames.map((cacheName) => {
          if (cacheName !== CACHE_NAME) {
            console.log("[SW] Deleting old cache:", cacheName);
            return caches.delete(cacheName);
          }
        })
      );
    }).then(() => clients.claim())
  );
});

// Network-first falling back to cache strategy
globalThis.addEventListener("fetch", (event) => {
  // Bypass service worker caching for API or stats WebSocket requests
  if (event.request.url.includes("/api/") || event.request.url.includes("/config")) {
    return;
  }

  event.respondWith(
    fetch(event.request)
      .then((response) => {
        // If valid response, clone it and update cache
        if (response && response.status === 200 && response.type === "basic") {
          const responseToCache = response.clone();
          caches.open(CACHE_NAME).then((cache) => {
            cache.put(event.request, responseToCache);
          });
        }
        return response;
      })
      .catch(() => {
        // Network failed (e.g. offline), try cache
        return caches.match(event.request);
      })
  );
});
