import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { viteSingleFile as singleFile } from "vite-plugin-singlefile";

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue(), singleFile()],
});
