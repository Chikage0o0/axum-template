{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:
let
  dbName = "project_name";
  dbUser = "project_name_user";
  dbPass = "project_name_pass"; # 仅供开发环境使用
in
{
  dotenv.enable = true;

  env.DATABASE_URL = lib.mkDefault "postgres://${dbUser}:${dbPass}@localhost/${dbName}?host=${config.env.PGHOST}";
  env.OPENSSL_DIR = "${pkgs.openssl.dev}";
  env.OPENSSL_NO_VENDOR = "1";

  packages = [
    pkgs.pkg-config
    pkgs.openssl
    pkgs.sqlx-cli
    pkgs.cargo-watch
    pkgs.cargo-edit
    pkgs.bacon
    pkgs.taplo
  ];

  languages.javascript = {
    enable = true;
    directory = "frontend";
    bun = {
      enable = true;
      install.enable = true;
    };
  };

  languages.rust = {
    enable = true;
    channel = "stable";
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];
  };

  services.postgres = {
    enable = true;
    package = pkgs.postgresql_18;
    initialDatabases = [
      {
        name = dbName;
        user = dbUser;
        pass = dbPass;
      }
    ];
    listen_addresses = "";
  };

  scripts.db-migrate.exec = "sqlx migrate run";
  scripts.backend-dev.exec = "PROJECT_NAME_AUTO_MIGRATE=0 cargo watch --skip-local-deps -c -d 0.5 -w src -w Cargo.toml -w Cargo.lock -L info -B 1 -x run";
  scripts.backend-dev-migrate.exec = "PROJECT_NAME_AUTO_MIGRATE=1 cargo watch --skip-local-deps -c -d 0.5 -w src -w migrations -w Cargo.toml -w Cargo.lock -L info -B 1 -x run";
  scripts.check.exec = "bacon";
  scripts.dev-fmt.exec = ''
    set -euo pipefail
    echo "[fmt] cargo fmt"
    cargo fmt
    echo "[fmt] taplo fmt"
    taplo fmt
    echo "[fmt] frontend prettier"
    bun run --cwd frontend format
  '';
  scripts.dev-fmt-check.exec = ''
    set -euo pipefail
    echo "[fmt-check] cargo fmt --check"
    cargo fmt -- --check
    echo "[fmt-check] taplo fmt --check"
    taplo fmt --check
    echo "[fmt-check] frontend prettier --check"
    bun run --cwd frontend format:check
  '';
  scripts.openapi-gen.exec = "cargo run --quiet -- --export-openapi > docs/openapi.json && (cd frontend && bun run gen:openapi && bun run gen:openapi:zod)";
  scripts.openapi-check.exec = "cargo run --quiet -- --export-openapi > docs/openapi.json && (cd frontend && bun run gen:openapi && bun run gen:openapi:zod) && git diff --exit-code";
  scripts.frontend-dev.exec = "cd frontend && bun run dev";
  scripts.process-up.exec = "devenv processes up --detach";
  scripts.process-down.exec = "devenv processes down";

  enterShell = ''
    if [ -f .env ]; then
      echo "Loaded .env"
    else
      echo "No .env found; using defaults"
      echo "Tip: cp .env.example .env"
    fi

    echo ""
    echo "PROJECT_NAME environment loaded"
    echo "Rust: $(rustc --version)"
    echo "Bun: $(bun --version)"
    echo "PostgreSQL: $(pg_isready && echo 'Running' || echo 'Stopped')"
    echo ""
    echo "Quick commands:"
    echo "  devenv up"
    echo "  db-migrate"
    echo "  dev-fmt"
    echo "  dev-fmt-check"
    echo "  backend-dev"
    echo "  frontend-dev"
  '';
}
