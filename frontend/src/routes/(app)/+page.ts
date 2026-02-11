import type { PageLoad } from "./$types";

export const load: PageLoad = () => {
  return {
    pageTitle: "仪表盘",
    breadcrumb: { section: "管理", page: "仪表盘" },
  };
};
