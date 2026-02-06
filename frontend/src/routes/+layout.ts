import { browser } from "$app/environment";
import { redirect, type Load } from "@sveltejs/kit";

export const ssr = false;
export const prerender = false;

export const load: Load = ({ url }) => {
  if (!browser) return {};
  if (url.pathname.startsWith("/login")) return {};

  let token: string | null = null;
  try {
    token = localStorage.getItem("token");
  } catch {
    token = null;
  }

  if (!token) {
    throw redirect(303, "/login");
  }

  return {};
};
