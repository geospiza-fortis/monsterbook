import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  // The session worker uses dynamic import (wasm vs mock), which requires
  // code-splitting; the default iife worker format does not support it.
  worker: { format: "es" },
});
