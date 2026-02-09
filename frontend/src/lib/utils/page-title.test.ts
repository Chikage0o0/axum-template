import { describe, expect, it } from "bun:test";
import {
  APP_DISPLAY_NAME,
  composeDocumentTitleFromPageData,
  composeDocumentTitle,
} from "./page-title";

describe("composeDocumentTitle", () => {
  it("应按“标签页名 | PROJECT_NAME”格式拼接标题", () => {
    expect(composeDocumentTitle("设置")).toBe(`设置 | ${APP_DISPLAY_NAME}`);
  });

  it("当标签页名为空时应回退为项目名", () => {
    expect(composeDocumentTitle(" ")).toBe(APP_DISPLAY_NAME);
  });
});

describe("composeDocumentTitleFromPageData", () => {
  it("应从页面配置的 pageTitle 生成标题", () => {
    expect(composeDocumentTitleFromPageData({ pageTitle: "设置" })).toBe(
      `设置 | ${APP_DISPLAY_NAME}`,
    );
  });

  it("页面未配置 pageTitle 时应回退为项目名", () => {
    expect(composeDocumentTitleFromPageData({})).toBe(APP_DISPLAY_NAME);
    expect(composeDocumentTitleFromPageData(null)).toBe(APP_DISPLAY_NAME);
  });
});
