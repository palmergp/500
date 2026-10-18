// Offline support. On install, cache the app shell and every built file listed in Vite's build
// manifest (scripts, the engine worker, the WebAssembly engine, styles). Afterwards: pages come
// from the network when it's available (so updates arrive), otherwise from the cache; hashed
// assets come from the cache first.

const CACHE = "fh500-v1";
const SHELL = ["./", "./index.html", "./manifest.webmanifest", "./icon.svg", "./icon-192.png", "./icon-512.png"];

async function precache() {
  const cache = await caches.open(CACHE);
  await cache.addAll(SHELL);
  try {
    const manifest = await (await fetch("./.vite/manifest.json", { cache: "no-store" })).json();
    const files = new Set();
    for (const entry of Object.values(manifest)) {
      files.add(entry.file);
      (entry.css || []).forEach((f) => files.add(f));
      (entry.assets || []).forEach((f) => files.add(f));
    }
    await cache.addAll([...files].map((f) => "./" + f));
  } catch {
    // Development builds have no manifest; runtime caching below still applies.
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
