use codex_hud_core::client;
use std::path::PathBuf;
use tokio::sync::watch;

/// 真实账号探针必须在用户控制的 Windows 中手动运行；输出不包含实际额度或服务器原文。
#[tokio::main]
async fn main(){
    if !cfg!(windows){eprintln!("未执行：此探针必须在 Windows 中运行，Mac 不能计作平台验收。");std::process::exit(2);}
    let args:Vec<String>=std::env::args().collect();
    if !args.iter().any(|a|a=="--allow-real-cli"){eprintln!("未执行：真实 CLI 验收需显式传入 --allow-real-cli；不会读取 WSL 或复制凭据。");std::process::exit(2);}
    let launch=if let Some(i)=args.iter().position(|a|a=="--executable"){
        args.get(i+1).ok_or_else(||"缺少绝对路径".to_string()).and_then(|path|client::native_launch(&PathBuf::from(path)))
    }else{client::discover()};
    let launch=match launch{Ok(v)=>v,Err(e)=>{eprintln!("{e}");std::process::exit(1);}};
    let directory=std::env::temp_dir().join("codex-hud-native-cli-probe");let (_tx,rx)=watch::channel(false);
    match client::read(&launch,&directory,client::TIMEOUT,rx).await{
        Ok(windows)=>println!("原生 Windows CLI JSONL 读取完成：{} 个窗口（真实数值已省略）；仍需单独验收 UI 和进程清理。",windows.len()),
        Err(error)=>{eprintln!("{error}");std::process::exit(1);}
    }
}
