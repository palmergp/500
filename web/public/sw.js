// Offline support. On install, cache the app shell and every built file listed in precache.json
// (scripts, the engine worker, the WebAssembly engine, styles). Afterwards: pages come
// from the network when it's available (so updates arrive), otherwise from the cache; hashed
// assets come from the cache first.

// The build adds its own name to this (vite.config.ts), so each release gets a fresh cache.
const CACHE = "fh500-v2";
const SHELL = ["./", "./index.html", "./manifest.webmanifest", "./icon.svg", "./icon-192.png", "./icon-512.png", "./apple-touch-icon.png"];

async function precache() {
  const cache = await caches.open(CACHE);
  await cache.addAll(SHELL);
  try {
    // Every built file (see precacheList in vite.config.ts), including the WebAssembly engine.
    const files = await (await fetch("./precache.json", { cache: "no-store" })).json();
    await cache.addAll(files.map((f) => "./" + f));
  } catch {
    // Development builds have no list; runtime caching below still applies.
  }
}

self.addEventListener("install", (event) => {
  self.skipWaiting();
  event.waitUntil(precache());
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) => Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k))))
      .then(() => self.clients.claim()),
  );
});

self.addEventListener("fetch", (event) => {
  const request = event.request;
  const url = new URL(request.url);
  // The LAN host's live endpoints are never cached.
  if (request.method !== "GET" || url.origin !== self.location.origin || url.pathname.includes("/api/")) return;
  if (request.mode === "navigate") {
    event.respondWith(
      fetch(request)
        .then((response) => {
          const copy = response.clone();
          caches.open(CACHE).then((cache) => cache.put("./index.html", copy));
          return response;
        })
        .catch(() => caches.match("./index.html")),
    );
    return;
  }
  event.respondWith(
    caches.match(request).then(
      (hit) =>
        hit ||
        fetch(request).then((response) => {
          if (response.ok) {
            const copy = response.clone();
            caches.open(CACHE).then((cache) => cache.put(request, copy));
          }
          return response;
        }),
    ),
  );
});
