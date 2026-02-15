export type PermissionSet = {
  permissions: ReadonlySet<string>;
  can: (permCode: string) => boolean;
};

export function createPermissionSet(input: readonly string[]): PermissionSet {
  const permissions = new Set(input.map((item) => item.trim()).filter((item) => item.length > 0));

  return {
    permissions,
    can(permCode: string) {
      const target = permCode.trim();
      if (!target) return false;

      if (permissions.has("*")) {
        return true;
      }
      if (permissions.has(target)) {
        return true;
      }

      const namespaceIndex = target.indexOf(":");
      if (namespaceIndex > 0) {
        const namespaceWildcard = `${target.slice(0, namespaceIndex)}:*`;
        if (permissions.has(namespaceWildcard)) {
          return true;
        }
      }

      return false;
    },
  };
}
