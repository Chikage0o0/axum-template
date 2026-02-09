export const APP_DISPLAY_NAME = "PROJECT_NAME";

type PageDataWithTitle = {
  pageTitle?: string | null;
};

export function composeDocumentTitle(tabName: string | null | undefined): string {
  const normalized = tabName?.trim();
  if (!normalized) {
    return APP_DISPLAY_NAME;
  }

  return `${normalized} | ${APP_DISPLAY_NAME}`;
}

export function composeDocumentTitleFromPageData(
  pageData: PageDataWithTitle | null | undefined,
): string {
  return composeDocumentTitle(pageData?.pageTitle);
}
