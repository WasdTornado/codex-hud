fn main() {
    // 更新源和公钥变化必须重新编译 option_env!，避免复用旧配置。
    println!("cargo:rerun-if-env-changed=HUD_WINDOWS_UPDATE_URL");
    println!("cargo:rerun-if-env-changed=HUD_WINDOWS_UPDATE_PUBLIC_KEY");
    // 集成测试不继承应用资源；对话框依赖要求 Common Controls v6。
    // 与 Tauri 自身 embed_manifest_for_tests 相同，单独给测试链接 manifest。
    if std::env::var("TARGET").unwrap_or_default().ends_with("windows-msvc") {
        let manifest=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("tests/windows.manifest");
        println!("cargo:rerun-if-changed={}",manifest.display());
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}",manifest.display());
    }
    tauri_build::build()
}
