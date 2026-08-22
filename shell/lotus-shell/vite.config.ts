import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Loopback only: the dev/preview servers must not be reachable from the LAN.
export default defineConfig({
  plugins: [react()],
  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true
  },
  preview: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true
  }
});
