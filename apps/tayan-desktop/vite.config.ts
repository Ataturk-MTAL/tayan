import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vitest/config";
import path from "node:path";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 5183 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  /**
   * Birim testler saf modüller içindir: ayrıştırıcılar, doğrulayıcılar,
   * öneri üreticileri. Bileşen testi YOK — tarayıcı ortamı gerektirir ve
   * asıl riskin olduğu yer orası değil.
   *
   * $lib takma adı elle çözülüyor: sveltekit() eklentisi test koşumunda
   * devrede olmadığı için kendi takma adlarını kurmuyor.
   */
  test: {
    include: ["src/**/*.test.ts"],
    environment: "node",
    alias: { $lib: path.resolve("./src/lib") },
  },
  build: {
    target:
      process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari16",
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
