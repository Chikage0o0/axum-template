-- Seed system_config defaults (idempotent)

-- 示例：应用配置
INSERT INTO system_config (key, value, description)
VALUES
  ('app.check_interval_secs', '3600'::jsonb, '示例：检查间隔（秒）'),
  ('app.welcome_message', '"Hello from PROJECT_NAME"'::jsonb, '示例：欢迎语')
ON CONFLICT (key) DO NOTHING;

-- 示例：集成配置（演示“密钥不回显，仅返回 is_set”）
INSERT INTO system_config (key, value, description)
VALUES
  ('integrations.example_api_base', '"https://example.com/api"'::jsonb, '示例：外部 API Base URL'),
  ('integrations.example_api_key', '""'::jsonb, '示例：外部 API Key（留空表示未设置）')
ON CONFLICT (key) DO NOTHING;
