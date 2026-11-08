import { createHash } from "node:crypto";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";

/**
 * Writes dist/precache.json: every built asset, for the service worker to cache on install. Vite's
 * manifest misses files the engine worker pulls in (the WebAssembly engine), since the worker is
 * bundled on its own.
 */
function precacheList(): Plugin {
  let outDir = "dist";
  return {
    name: "precache-list",
    apply: "build",
    configResolved(config) {
      outDir = config.build.outDir;
    },
    closeBundle() {
      const files = readdirSync(join(outDir, "assets")).map((f) => `assets/${f}`);
      writeFileSync(join(outDir, "precache.json"), JSON.stringify(files));
      // Name the offline cache after this build (the assets' names carry their content hashes), so
      // a new version installs a fresh service worker: it caches the new files whole and drops
      // the old ones, and an installed app on a phone updates the next time it's opened online.
      const build = createHash("sha256").update([...files].sort().join(" ")).digest("hex").slice(0, 10);
      const sw = join(outDir, "sw.js");
      const source = readFileSync(sw, "utf-8");
      if (!source.includes('const CACHE = "fh500-v2";')) throw new Error("sw.js: CACHE name not found");
      writeFileSync(sw, source.replace('const CACHE = "fh500-v2";', `const CACHE = "fh500-v2-${build}";`));
    },
  };
}

export default defineConfig({
  plugins: [react(), precacheList()],
  base: "./",
  build: { target: "es2022", manifest: true },
});
