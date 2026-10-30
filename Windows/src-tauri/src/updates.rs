use tauri::{AppHandle,Manager};
use tauri_plugin_updater::UpdaterExt;
use tauri_plugin_dialog::{DialogExt,MessageDialogButtons};

/// 更新源与公钥仅从编译配置注入，默认关闭；绝不复用 macOS Sparkle 源或密钥。
pub async fn check(app:AppHandle,manual:bool)->Result<(),String>{
    let (Some(endpoint),Some(key))=(option_env!("HUD_WINDOWS_UPDATE_URL"),option_env!("HUD_WINDOWS_UPDATE_PUBLIC_KEY")) else {
        if manual{app.dialog().message("本地平台验证版尚未配置 Windows 更新源和验签公钥。").title("检查更新").show(|_|{});}return Ok(());
    };
    if !endpoint.starts_with("https://")||key.is_empty(){return Err("Windows 更新配置无效".into());}
    let updater=app.updater_builder().endpoints(vec![endpoint.parse().map_err(|_|"更新地址无效")?]).map_err(|_|"更新配置无效")?.pubkey(key).restart_after_install(true).build().map_err(|_|"更新配置无效")?;
    let update=updater.check().await.map_err(|_|"检查更新失败，请稍后重试")?;
    if let Some(update)=update{
        let version=update.version.clone();
        app.state::<crate::Runtime>().data.lock().unwrap().update_available=Some(version.clone());crate::publish(&app);
        // 自动检查只提示托盘状态；安装必须由用户主动检查并在原生对话框确认。
        if manual&&app.dialog().message(format!("发现 Windows 版本 {version}。下载并验证签名后安装，安装程序可能退出并重启 Codex HUD。是否继续？")).title("Codex HUD 更新").buttons(MessageDialogButtons::OkCancel).blocking_show(){
            update.download_and_install(|_,_|{},||{}).await.map_err(|_|"更新下载、验签或安装失败；未确认完成更新")?;
        }
    }else{
        app.state::<crate::Runtime>().data.lock().unwrap().update_available=None;crate::publish(&app);
        if manual{app.dialog().message("当前没有可用的 Windows 更新。").title("检查更新").show(|_|{});}
    }
    Ok(())
}
