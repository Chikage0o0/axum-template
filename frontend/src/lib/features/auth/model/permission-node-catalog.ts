import {
  listPermissionNodesHandler,
  type PermissionNodeDictionaryResponse,
  type PermissionNodeItemResponse,
} from "$lib/api/generated/client";

export type PermissionOption = {
  value: string;
  label: string;
  module: string;
  description: string;
};

export function toPermissionOptions(
  dictionary: PermissionNodeDictionaryResponse,
): PermissionOption[] {
  return dictionary.items.map(mapPermissionNodeToOption);
}

export async function loadPermissionOptions(): Promise<PermissionOption[]> {
  const dictionary = await listPermissionNodesHandler();
  return toPermissionOptions(dictionary);
}

function mapPermissionNodeToOption(item: PermissionNodeItemResponse): PermissionOption {
  return {
    value: item.code,
    label: `${item.name} (${item.code})`,
    module: item.module,
    description: item.description,
  };
}
