import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "e2e",
  timeout: 180_000,
  use: { baseURL: "http://localhost:4173" },
  webServer: [
    {
      command: "npx vite preview --port 4173 --strictPort",
      url: "http://localhost:4173",
      reuseExistingServer: true,
    },
    {
      // The LAN host, serving the same build, with a fresh table each run.
      command: "cargo run --release -p fh-host -- --port 5599 --dist dist --state test-results/lan-table.json --fresh --no-browser",
      url: "http://localhost:5599/api/info",
      reuseExistingServer: true,
      timeout: 600_000,
    },
  ],
  projects: [
    { name: "desktop", use: { ...devices["Desktop Chrome"] }, testIgnore: "lan.spec.ts" },
    { name: "phone", use: { ...devices["Pixel 7"] }, testIgnore: "lan.spec.ts" },
    { name: "lan", use: { ...devices["Desktop Chrome"], baseURL: "http://localhost:5599" }, testMatch: "lan.spec.ts" },
  ],
});
