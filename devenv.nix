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
    pkgs.pre-commit
    pkgs.bacon
    pkgs.taplo
    pkgs.pgcli
    pkgs.go-task
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
  };

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
    echo "  pre-git-check"
    echo "  backend-dev"
    echo "  frontend-dev"
  '';
}
