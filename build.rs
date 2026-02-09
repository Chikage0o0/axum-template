use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(embed_frontend)");
    println!("cargo:rerun-if-changed=migrations");

    if is_release_profile() {
        println!("cargo:rustc-cfg=embed_frontend");
        println!("cargo:rerun-if-changed=frontend/src");
        println!("cargo:rerun-if-changed=frontend/static");
        println!("cargo:rerun-if-changed=frontend/scripts");
        println!("cargo:rerun-if-changed=frontend/components.json");
        println!("cargo:rerun-if-changed=frontend/vite.config.ts");
        println!("cargo:rerun-if-changed=frontend/tsconfig.json");
        println!("cargo:rerun-if-changed=frontend/orval.config.ts");
        println!("cargo:rerun-if-changed=frontend/svelte.config.js");
        println!("cargo:rerun-if-changed=frontend/package.json");
        println!("cargo:rerun-if-changed=frontend/bun.lock");

        clean_frontend_build_output();
        build_frontend();
    }
}

fn is_release_profile() -> bool {
    std::env::var("PROFILE").is_ok_and(|profile| profile == "release")
}

fn clean_frontend_build_output() {
    let build_dir = Path::new("frontend/build");
    if !build_dir.exists() {
        return;
    }

    if let Err(error) = std::fs::remove_dir_all(build_dir) {
        panic!("清理前端构建目录失败: {error}");
    }
}

fn build_frontend() {
    let frontend_dir = Path::new("frontend");
    if !frontend_dir.exists() {
        panic!("缺少 frontend 目录，无法构建前端静态资源");
    }

    let status = Command::new("bun")
        .arg("run")
        .arg("build")
        .current_dir(frontend_dir)
        .status();

    match status {
        Ok(exit_status) if exit_status.success() => {}
        Ok(exit_status) => {
            panic!("前端构建失败，退出码: {exit_status}");
        }
        Err(error) => {
            panic!("执行 bun run build 失败，请先安装 bun: {error}");
        }
    }
}
