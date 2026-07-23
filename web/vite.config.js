import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  // The session worker uses dynamic import (wasm vs mock), which requires
  // code-splitting; the default iife worker format does not support it.
  worker: { format: "es" },
  // Listen on all interfaces and skip the Host-header check so the dev
  // server is reachable from other machines on the LAN.
  server: { host: true, allowedHosts: true },
});
