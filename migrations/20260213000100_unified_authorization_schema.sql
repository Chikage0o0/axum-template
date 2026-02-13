CREATE TABLE sys_permission (
    perm_code TEXT PRIMARY KEY,
    perm_name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE sys_policy (
    policy_id BIGSERIAL PRIMARY KEY,
    subject_type TEXT NOT NULL,
    subject_key TEXT NOT NULL,
    perm_code TEXT NOT NULL REFERENCES sys_permission(perm_code),
    effect TEXT NOT NULL,
    scope_rule TEXT NOT NULL DEFAULT 'ALL',
    constraints JSONB NOT NULL DEFAULT '{}'::jsonb,
    expire_at TIMESTAMPTZ,
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT sys_policy_subject_type_check CHECK (subject_type IN ('USER', 'ROLE')),
    CONSTRAINT sys_policy_effect_check CHECK (effect IN ('ALLOW', 'DENY')),
    CONSTRAINT sys_policy_scope_rule_non_empty CHECK (char_length(btrim(scope_rule)) > 0),
    CONSTRAINT sys_policy_constraints_object_check CHECK (jsonb_typeof(constraints) = 'object'),
    CONSTRAINT sys_policy_constraints_expire_at_type_check CHECK (
        NOT (constraints ? 'expire_at')
        OR jsonb_typeof(constraints -> 'expire_at') = 'string'
    ),
    CONSTRAINT sys_policy_constraints_ip_range_type_check CHECK (
        NOT (constraints ? 'ip_range')
        OR jsonb_typeof(constraints -> 'ip_range') = 'string'
    )
);

CREATE INDEX idx_sys_policy_subject ON sys_policy (subject_type, subject_key);
CREATE INDEX idx_sys_policy_perm_code ON sys_policy (perm_code);
CREATE INDEX idx_sys_policy_expire_at ON sys_policy (expire_at);

INSERT INTO sys_permission (perm_code, perm_name, description)
VALUES
    ('*', 'All Permissions', '全局通配符权限'),
    ('users:*', 'Users Namespace', '用户模块通配符权限'),
    ('users:me:view', 'View Current User', '查看当前用户信息'),
    ('users:me:update', 'Update Current User', '更新当前用户信息'),
    ('users:list', 'List Users', '查看用户列表'),
    ('users:create', 'Create User', '创建用户'),
    ('users:update', 'Update User', '更新用户'),
    ('users:delete', 'Delete User', '删除用户'),
    ('users:restore', 'Restore User', '恢复用户'),
    ('settings:view', 'View Settings', '读取系统设置'),
    ('settings:update', 'Update Settings', '更新系统设置'),
    ('security:password:update', 'Update Password', '修改当前用户密码'),
    ('sessions:current:delete', 'Logout Current Session', '退出当前会话')
ON CONFLICT (perm_code) DO NOTHING;

INSERT INTO sys_policy (
    subject_type,
    subject_key,
    perm_code,
    effect,
    scope_rule,
    constraints,
    priority
)
VALUES
    ('ROLE', 'admin', '*', 'ALLOW', 'ALL', '{}'::jsonb, 100),
    ('ROLE', 'user', 'settings:view', 'ALLOW', 'ALL', '{}'::jsonb, 10),
    ('ROLE', 'user', 'users:me:view', 'ALLOW', 'SELF', '{}'::jsonb, 10),
    ('ROLE', 'user', 'users:me:update', 'ALLOW', 'SELF', '{}'::jsonb, 10),
    ('ROLE', 'user', 'security:password:update', 'ALLOW', 'SELF', '{}'::jsonb, 10),
    ('ROLE', 'user', 'sessions:current:delete', 'ALLOW', 'SELF', '{}'::jsonb, 10);
