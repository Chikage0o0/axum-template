import { formatBrowserLocalDateTime } from "$lib/shared/utils/date-time";

type UserDetailRowInput = {
  id: string;
  username?: string | null;
  email: string;
  phone?: string | null;
  created_at: string;
  updated_at: string;
  formatDateTime?: (value: string) => string;
};

export type UserDetailRow = {
  label: string;
  value: string;
};

function optionalText(value?: string | null): string {
  const trimmed = value?.trim();
  return trimmed ? trimmed : "-";
}

export function toUserStatusLabel(isActive: boolean): string {
  return isActive ? "启用" : "禁用";
}

export function buildUserDetailRows(user: UserDetailRowInput): UserDetailRow[] {
  const formatDateTime = user.formatDateTime ?? formatBrowserLocalDateTime;

  return [
    { label: "用户ID", value: user.id },
    { label: "用户名", value: optionalText(user.username) },
    { label: "邮箱", value: user.email },
    { label: "手机号", value: optionalText(user.phone) },
    { label: "创建时间", value: formatDateTime(user.created_at) },
    { label: "更新时间", value: formatDateTime(user.updated_at) },
  ];
}
