use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

macro_rules! permission_nodes {
    (
        $(
            $(#[$meta:meta])*
            $variant:ident => $code:literal
        ),+ $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
        pub enum PermissionNode {
            $(
                $(#[$meta])*
                #[serde(rename = $code)]
                #[schema(rename = $code)]
                $variant,
            )+
        }

        impl PermissionNode {
            pub const ALL: [PermissionNode; permission_nodes!(@count $($variant),+)] = [
                $(PermissionNode::$variant,)+
            ];

            pub const fn as_str(self) -> &'static str {
                match self {
                    $(PermissionNode::$variant => $code,)+
                }
            }

            pub fn try_from_code(code: &str) -> Option<Self> {
                match code {
                    $($code => Some(PermissionNode::$variant),)+
                    _ => None,
                }
            }
        }

        const PERMISSION_CATALOG: [PermissionNode; permission_nodes!(@count $($variant),+)] = [
            $(PermissionNode::$variant,)+
        ];
    };
    (@count $($variant:ident),+ $(,)?) => {
        <[()]>::len(&[$(permission_nodes!(@replace $variant ())),+])
    };
    (@replace $_variant:ident $value:expr) => {
        $value
    };
}

permission_nodes! {
    /// 全局通配符权限。
    All => "*",
    /// 用户模块通配符权限。
    UsersAll => "users:*",
    /// 查看用户列表。
    UsersList => "users:list",
    /// 创建用户。
    UsersCreate => "users:create",
    /// 更新用户。
    UsersUpdate => "users:update",
    /// 删除用户。
    UsersDelete => "users:delete",
    /// 恢复用户。
    UsersRestore => "users:restore",
    /// 读取系统设置。
    SettingsView => "settings:view",
    /// 更新系统设置。
    SettingsUpdate => "settings:update",
    /// 删除会话。
    SessionsDelete => "sessions:delete",
    /// 读取权限节点字典。
    AuthorizationPermissionNodesView => "authorization:permission-nodes:view",
}

pub fn permission_catalog() -> &'static [PermissionNode] {
    &PERMISSION_CATALOG
}

pub fn permission_catalog_version() -> String {
    permission_catalog_version_for_codes(
        permission_catalog()
            .iter()
            .map(|permission| permission.as_str()),
    )
}

pub fn permission_catalog_version_for_codes<'a, I>(codes: I) -> String
where
    I: IntoIterator<Item = &'a str>,
{
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for code in codes {
        hash = fnv1a_update(hash, code.as_bytes(), FNV_PRIME);
        hash = fnv1a_update(hash, b"\x1e", FNV_PRIME);
    }

    format!("{hash:016x}")
}

fn fnv1a_update(mut hash: u64, bytes: &[u8], prime: u64) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(prime);
    }
    hash
}
