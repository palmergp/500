import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "e2e",
  timeout: 180_000,
  use: { baseURL: "http://localhost:4173" },
  webServer: {
    command: "npx vite preview --port 4173 --strictPort",
    url: "http://localhost:4173",
    reuseExistingServer: true,
  },
  projects: [
    { name: "desktop", use: { ...devices["Desktop Chrome"] } },
    { name: "phone", use: { ...devices["Pixel 7"] } },
  ],
});
