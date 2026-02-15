INSERT INTO sys_permission (perm_code, perm_name, description)
VALUES (
    'authorization:permission-nodes:view',
    'View Permission Nodes',
    '读取权限节点字典'
)
ON CONFLICT (perm_code) DO UPDATE
SET perm_name = EXCLUDED.perm_name,
    description = EXCLUDED.description,
    updated_at = NOW();
