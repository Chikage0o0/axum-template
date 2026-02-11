export const avatarBeamColors = [
  "var(--sidebar-primary)",
  "var(--chart-1)",
  "var(--chart-2)",
  "var(--chart-3)",
  "var(--chart-4)",
];

type BuildAvatarSeedInput = {
  email?: string | null;
  displayName?: string | null;
  id?: string | null;
};

export function buildAvatarSeed(input: BuildAvatarSeedInput): string {
  const email = input.email?.trim();
  if (email) return email;

  const displayName = input.displayName?.trim();
  if (displayName) return displayName;

  const id = input.id?.trim();
  if (id) return id;

  return "unknown-user";
}

export function normalizeAvatarBeamSize(size?: number): number {
  if (typeof size !== "number" || !Number.isFinite(size)) {
    return 32;
  }

  const normalized = Math.floor(size);
  if (normalized < 16) return 16;
  if (normalized > 128) return 128;
  return normalized;
}
