import type { PageLoad } from "./$types";

export const load: PageLoad = () => {
  return {
    pageTitle: "设置",
    breadcrumb: { section: "管理", page: "设置" },
  };
};
