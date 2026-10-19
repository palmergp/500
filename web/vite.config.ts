import { readdirSync, writeFileSync } from "node:fs";
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
    },
  };
}

export default defineConfig({
  plugins: [react(), precacheList()],
  base: "./",
  build: { target: "es2022", manifest: true },
});
