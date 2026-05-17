import adapter from "@sveltejs/adapter-static";
import { mdsvex } from "mdsvex";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

const config = {
  extensions: [".svelte", ".svx"],
  preprocess: [
    vitePreprocess(),
    mdsvex({
      extensions: [".svx"]
    })
  ],
  kit: {
    adapter: adapter({
      fallback: "index.html"
    })
  }
};

export default config;
