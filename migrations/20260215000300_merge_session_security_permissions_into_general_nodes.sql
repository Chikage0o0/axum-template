INSERT INTO sys_permission (perm_code, perm_name, description)
VALUES ('sessions:delete', 'Delete Session', '删除会话')
ON CONFLICT (perm_code) DO UPDATE
SET perm_name = EXCLUDED.perm_name,
    description = EXCLUDED.description,
    updated_at = NOW();

INSERT INTO sys_policy (
    subject_type,
    subject_key,
    perm_code,
    effect,
    scope_rule,
    constraints,
    expire_at,
    priority
)
SELECT
    p.subject_type,
    p.subject_key,
    CASE p.perm_code
        WHEN 'security:password:update' THEN 'users:update'
        WHEN 'sessions:current:delete' THEN 'sessions:delete'
    END,
    p.effect,
    p.scope_rule,
    p.constraints,
    p.expire_at,
    p.priority
FROM sys_policy p
WHERE p.perm_code IN ('security:password:update', 'sessions:current:delete')
  AND NOT EXISTS (
      SELECT 1
      FROM sys_policy t
      WHERE t.subject_type = p.subject_type
        AND t.subject_key = p.subject_key
        AND t.perm_code = CASE p.perm_code
            WHEN 'security:password:update' THEN 'users:update'
            WHEN 'sessions:current:delete' THEN 'sessions:delete'
        END
        AND t.effect = p.effect
        AND t.scope_rule = p.scope_rule
        AND t.constraints = p.constraints
        AND t.priority = p.priority
        AND t.expire_at IS NOT DISTINCT FROM p.expire_at
  );

DELETE FROM sys_policy
WHERE perm_code IN ('security:password:update', 'sessions:current:delete');

DELETE FROM sys_permission
WHERE perm_code IN ('security:password:update', 'sessions:current:delete', 'users:password:update');
