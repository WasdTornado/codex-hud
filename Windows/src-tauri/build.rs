fn main() {
    // 更新源和公钥变化必须重新编译 option_env!，避免复用旧配置。
    println!("cargo:rerun-if-env-changed=HUD_WINDOWS_UPDATE_URL");
    println!("cargo:rerun-if-env-changed=HUD_WINDOWS_UPDATE_PUBLIC_KEY");
    tauri_build::build()
}
