import { z, type core } from "zod";

type ZodIssueLike = core.$ZodIssue | core.$ZodRawIssue<core.$ZodIssue>;

let initialized = false;

function hasChinese(message: string): boolean {
  return /[\u4e00-\u9fff]/.test(message);
}

function translateDefaultZodMessage(message: string): string {
  const tooSmallString = message.match(/^Too small: expected string to have >=(\d+) characters$/);
  if (tooSmallString) {
    const min = Number(tooSmallString[1]);
    return min <= 1 ? "不能为空" : `长度不能少于 ${min} 个字符`;
  }

  const tooBigString = message.match(/^Too big: expected string to have <=(\d+) characters$/);
  if (tooBigString) {
    const max = Number(tooBigString[1]);
    return `长度不能超过 ${max} 个字符`;
  }

  const tooSmallNumber = message.match(/^Too small: expected number to be >=([\d.-]+)$/);
  if (tooSmallNumber) {
    return `不能小于 ${tooSmallNumber[1]}`;
  }

  const tooBigNumber = message.match(/^Too big: expected number to be <=([\d.-]+)$/);
  if (tooBigNumber) {
    return `不能大于 ${tooBigNumber[1]}`;
  }

  if (message === "Invalid UUID") return "UUID 格式不正确";
  if (message === "Invalid email address") return "邮箱格式不正确";
  if (message === "Invalid ISO datetime") return "日期时间格式不正确";
  if (message.startsWith("Invalid input: expected") && message.endsWith("received undefined")) {
    return "不能为空";
  }

  return message;
}

export function translateZodIssue(issue: ZodIssueLike): string {
  if (issue.message && hasChinese(issue.message)) {
    return issue.message;
  }

  switch (issue.code) {
    case "custom":
      return issue.message || "输入不合法";
    case "invalid_type":
      return typeof issue.input === "undefined" ? "不能为空" : "类型不正确";
    case "too_small": {
      const minimum = Number(issue.minimum ?? 0);
      if (issue.origin === "string") {
        return minimum <= 1 ? "不能为空" : `长度不能少于 ${minimum} 个字符`;
      }
      if (issue.origin === "number" || issue.origin === "int" || issue.origin === "bigint") {
        return `不能小于 ${minimum}`;
      }
      if (issue.origin === "array" || issue.origin === "set") {
        return `至少需要 ${minimum} 项`;
      }
      return `长度不能少于 ${minimum}`;
    }
    case "too_big": {
      const maximum = Number(issue.maximum ?? 0);
      if (issue.origin === "string") {
        return `长度不能超过 ${maximum} 个字符`;
      }
      if (issue.origin === "number" || issue.origin === "int" || issue.origin === "bigint") {
        return `不能大于 ${maximum}`;
      }
      if (issue.origin === "array" || issue.origin === "set") {
        return `不能超过 ${maximum} 项`;
      }
      return `长度不能超过 ${maximum}`;
    }
    case "invalid_format": {
      if (issue.format === "email") return "邮箱格式不正确";
      if (issue.format === "uuid") return "UUID 格式不正确";
      if (issue.format === "datetime") return "日期时间格式不正确";
      if (issue.format === "url") return "URL 格式不正确";
      return "格式不正确";
    }
    default:
      return translateDefaultZodMessage(issue.message || "输入不合法");
  }
}

export function setupZodErrorMap(): void {
  if (initialized) return;

  z.setErrorMap((issue) => ({
    message: translateZodIssue(issue),
  }));

  initialized = true;
}
