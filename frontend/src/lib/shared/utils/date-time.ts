type DateFormatter = Pick<Intl.DateTimeFormat, "format">;

function createBrowserLocalFormatter(): DateFormatter {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  });
}

export function formatBrowserLocalDateTime(
  value: string,
  formatter: DateFormatter = createBrowserLocalFormatter(),
): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return "-";
  }

  return formatter.format(date);
}
