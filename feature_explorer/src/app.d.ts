declare global {
  namespace App {}
}

declare module "*.svx" {
  import type { Component } from "svelte";
  const component: Component;
  export default component;
}

export {};
