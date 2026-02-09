import type { Load } from "@sveltejs/kit";

export const ssr = false;
export const prerender = false;

export const load: Load = () => {
  return {};
};
