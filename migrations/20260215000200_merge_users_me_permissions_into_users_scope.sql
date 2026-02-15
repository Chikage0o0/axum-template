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
        WHEN 'users:me:view' THEN 'users:list'
        WHEN 'users:me:update' THEN 'users:update'
    END,
    p.effect,
    p.scope_rule,
    p.constraints,
    p.expire_at,
    p.priority
FROM sys_policy p
WHERE p.perm_code IN ('users:me:view', 'users:me:update')
  AND NOT EXISTS (
      SELECT 1
      FROM sys_policy t
      WHERE t.subject_type = p.subject_type
        AND t.subject_key = p.subject_key
        AND t.perm_code = CASE p.perm_code
            WHEN 'users:me:view' THEN 'users:list'
            WHEN 'users:me:update' THEN 'users:update'
        END
        AND t.effect = p.effect
        AND t.scope_rule = p.scope_rule
        AND t.constraints = p.constraints
        AND t.priority = p.priority
        AND t.expire_at IS NOT DISTINCT FROM p.expire_at
  );

DELETE FROM sys_policy
WHERE perm_code IN ('users:me:view', 'users:me:update');

DELETE FROM sys_permission
WHERE perm_code IN ('users:me:view', 'users:me:update');
