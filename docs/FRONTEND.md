# 前端约定（模板）

## 目标

- 演示登录 + 设置页的最小实现
- 演示统一 API 调用封装（自动带 token、401 触发登出）

## UI 组件策略（shadcn-svelte 优先）

- 前端页面与交互开发必须优先复用 shadcn-svelte 组件。
- 组件选型优先级：
  1. 优先使用仓库内已有组件：`frontend/src/lib/shadcn/components/ui/*`
  2. 缺失时通过 shadcn-svelte CLI 添加新组件（例如：`bunx shadcn-svelte@latest add button`）
  3. 仅在 shadcn-svelte 无法覆盖需求时，才在业务目录编写自定义组件
- 禁止直接修改 `frontend/src/lib/shadcn` 下的组件源码；如需变更样式或行为，优先通过组合封装、样式变量与业务层适配实现。

## shadcn-svelte 参考

- 官方文档入口：`https://shadcn-svelte.com/docs`

### Overview

- [About](https://shadcn-svelte.com/docs/about.md)
- [Changelog](https://shadcn-svelte.com/docs/changelog.md)
- [CLI](https://shadcn-svelte.com/docs/cli.md)
- [components.json](https://shadcn-svelte.com/docs/components-json.md)
- [JavaScript](https://shadcn-svelte.com/docs/javascript.md)
- [Legacy Docs](https://shadcn-svelte.com/docs/legacy.md)
- [Theming](https://shadcn-svelte.com/docs/theming.md)

### Installation

- [Astro](https://shadcn-svelte.com/docs/installation/astro.md)
- [Manual Installation](https://shadcn-svelte.com/docs/installation/manual.md)
- [SvelteKit](https://shadcn-svelte.com/docs/installation/sveltekit.md)
- [Vite](https://shadcn-svelte.com/docs/installation/vite.md)

### Components

#### Form & Input

- [Button](https://shadcn-svelte.com/docs/components/button.md)
- [Button Group](https://shadcn-svelte.com/docs/components/button-group.md)
- [Calendar](https://shadcn-svelte.com/docs/components/calendar.md)
- [Checkbox](https://shadcn-svelte.com/docs/components/checkbox.md)
- [Combobox](https://shadcn-svelte.com/docs/components/combobox.md)
- [Date Picker](https://shadcn-svelte.com/docs/components/date-picker.md)
- [Field](https://shadcn-svelte.com/docs/components/field.md)
- [Formsnap / Form](https://shadcn-svelte.com/docs/components/form.md)
- [Input](https://shadcn-svelte.com/docs/components/input.md)
- [Input Group](https://shadcn-svelte.com/docs/components/input-group.md)
- [Input OTP](https://shadcn-svelte.com/docs/components/input-otp.md)
- [Label](https://shadcn-svelte.com/docs/components/label.md)
- [Native Select](https://shadcn-svelte.com/docs/components/native-select.md)
- [Radio Group](https://shadcn-svelte.com/docs/components/radio-group.md)
- [Select](https://shadcn-svelte.com/docs/components/select.md)
- [Slider](https://shadcn-svelte.com/docs/components/slider.md)
- [Switch](https://shadcn-svelte.com/docs/components/switch.md)
- [Textarea](https://shadcn-svelte.com/docs/components/textarea.md)

#### Layout & Navigation

- [Accordion](https://shadcn-svelte.com/docs/components/accordion.md)
- [Breadcrumb](https://shadcn-svelte.com/docs/components/breadcrumb.md)
- [Navigation Menu](https://shadcn-svelte.com/docs/components/navigation-menu.md)
- [Resizable](https://shadcn-svelte.com/docs/components/resizable.md)
- [Scroll Area](https://shadcn-svelte.com/docs/components/scroll-area.md)
- [Separator](https://shadcn-svelte.com/docs/components/separator.md)
- [Sidebar](https://shadcn-svelte.com/docs/components/sidebar.md)
- [Tabs](https://shadcn-svelte.com/docs/components/tabs.md)

#### Overlays & Dialogs

- [Alert Dialog](https://shadcn-svelte.com/docs/components/alert-dialog.md)
- [Command](https://shadcn-svelte.com/docs/components/command.md)
- [Context Menu](https://shadcn-svelte.com/docs/components/context-menu.md)
- [Dialog](https://shadcn-svelte.com/docs/components/dialog.md)
- [Drawer](https://shadcn-svelte.com/docs/components/drawer.md)
- [Dropdown Menu](https://shadcn-svelte.com/docs/components/dropdown-menu.md)
- [Hover Card](https://shadcn-svelte.com/docs/components/hover-card.md)
- [Menubar](https://shadcn-svelte.com/docs/components/menubar.md)
- [Popover](https://shadcn-svelte.com/docs/components/popover.md)
- [Sheet](https://shadcn-svelte.com/docs/components/sheet.md)
- [Tooltip](https://shadcn-svelte.com/docs/components/tooltip.md)

#### Feedback & Status

- [Alert](https://shadcn-svelte.com/docs/components/alert.md)
- [Badge](https://shadcn-svelte.com/docs/components/badge.md)
- [Empty](https://shadcn-svelte.com/docs/components/empty.md)
- [Progress](https://shadcn-svelte.com/docs/components/progress.md)
- [Skeleton](https://shadcn-svelte.com/docs/components/skeleton.md)
- [Sonner](https://shadcn-svelte.com/docs/components/sonner.md)
- [Spinner](https://shadcn-svelte.com/docs/components/spinner.md)

#### Display & Media

- [Aspect Ratio](https://shadcn-svelte.com/docs/components/aspect-ratio.md)
- [Avatar](https://shadcn-svelte.com/docs/components/avatar.md)
- [Card](https://shadcn-svelte.com/docs/components/card.md)
- [Carousel](https://shadcn-svelte.com/docs/components/carousel.md)
- [Chart](https://shadcn-svelte.com/docs/components/chart.md)
- [Data Table](https://shadcn-svelte.com/docs/components/data-table.md)
- [Item](https://shadcn-svelte.com/docs/components/item.md)
- [Kbd](https://shadcn-svelte.com/docs/components/kbd.md)
- [Table](https://shadcn-svelte.com/docs/components/table.md)
- [Typography](https://shadcn-svelte.com/docs/components/typography.md)

#### Misc

- [Collapsible](https://shadcn-svelte.com/docs/components/collapsible.md)
- [Pagination](https://shadcn-svelte.com/docs/components/pagination.md)
- [Range Calendar](https://shadcn-svelte.com/docs/components/range-calendar.md)
- [Toggle](https://shadcn-svelte.com/docs/components/toggle.md)
- [Toggle Group](https://shadcn-svelte.com/docs/components/toggle-group.md)

### Dark Mode

- [Astro](https://shadcn-svelte.com/docs/dark-mode/astro.md)
- [Svelte](https://shadcn-svelte.com/docs/dark-mode/svelte.md)

### Migration

- [Svelte 5](https://shadcn-svelte.com/docs/migration/svelte-5.md)
- [Tailwind v4](https://shadcn-svelte.com/docs/migration/tailwind-v4.md)

### Registry

- [Examples](https://shadcn-svelte.com/docs/registry/examples.md)
- [FAQ](https://shadcn-svelte.com/docs/registry/faq.md)
- [Getting Started](https://shadcn-svelte.com/docs/registry/getting-started.md)
- [registry-item.json](https://shadcn-svelte.com/docs/registry/registry-item-json.md)
- [registry.json](https://shadcn-svelte.com/docs/registry/registry-json.md)

- 若涉及主题与暗色模式，优先按官方 Theming / Dark Mode 文档实现，避免在业务代码中分散硬编码主题色。

## Svelte 5 约束

- 组件内状态优先使用 runes（`$state/$derived/$effect`）
- Kit 状态用 `$app/state`（禁止 `$app/stores`）
