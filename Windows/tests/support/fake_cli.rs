//! 无账号原生测试入口；仅 test-fixtures feature 构建，不读取登录或网络。
use serde_json::{json,Value};
use std::{io::{self,BufRead,Write},time::Duration};
fn send(value:Value){println!("{value}");io::stdout().flush().unwrap();}
fn pause(){std::thread::sleep(Duration::from_secs(60));}
fn main(){
    let args:Vec<_>=std::env::args().skip(1).collect();
    if args==["--fixture-child"]{pause();return;}
    assert_eq!(args,["app-server","--listen","stdio://"],"假 CLI 只接受固定协议参数");
    let config:Value=serde_json::from_slice(&std::fs::read("fixture.json").expect("必须提供无账号测试配置")).unwrap();
    let mode=config["mode"].as_str().unwrap();
    let child=if config["child"].as_bool()==Some(true){
        Some(std::process::Command::new(std::env::current_exe().unwrap()).arg("--fixture-child")
            .stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn().unwrap())
    }else{None};
    std::fs::write("ready.json",json!({"parent":std::process::id(),"child":child.as_ref().map(|p|p.id())}).to_string()).unwrap();
    if config["gate"].as_bool()==Some(true){
        // 测试先取得进程句柄再放行，避免仅凭旧 PID 推断清理成功。
        let deadline=std::time::Instant::now()+Duration::from_secs(15);
        while !std::path::Path::new("continue").exists(){assert!(std::time::Instant::now()<deadline,"测试未放行");std::thread::sleep(Duration::from_millis(10));}
    }
    let mut input=io::stdin().lock().lines();let mut initialized=false;
    while let Some(line)=input.next(){
        let request:Value=serde_json::from_str(&line.unwrap()).unwrap();
        match request["method"].as_str(){
            Some("initialize")=>{
                match mode {
                    "timeout"=>pause(),"eof"=>return,
                    "oversize"=>{println!("{}","x".repeat(1_048_576));io::stdout().flush().unwrap();pause();},
                    "total"=>{for _ in 0..4097{println!("{}","x".repeat(1024));}io::stdout().flush().unwrap();pause();},
                    _=>{},
                }
                send(json!({"id":2,"result":{"rateLimits":{}}}));
                send(json!({"id":"1","result":{}}));send(json!({"id":true,"result":{}}));
                send(json!({"id":1.0,"result":{}}));
            },
            Some("initialized")=>initialized=true,
            Some("account/rateLimits/read")=>{
                assert!(initialized);
                if mode=="error"{send(json!({"id":2,"error":{"message":"secret-token-fixture-only"}}));continue;}
                println!("diagnostic noise");send(json!({"method":"unrelated/notification"}));
                send(json!({"id":"server-request","method":"unsupported/approval"}));
                let reply:Value=serde_json::from_str(&input.next().unwrap().unwrap()).unwrap();
                assert_eq!(reply["id"],"server-request");assert_eq!(reply["error"]["code"],-32601);
                send(json!({"id":2.0,"result":{"rateLimitsByLimitId":{"codex":{
                    "primary":{"windowDurationMins":10080,"usedPercent":53},
                    "secondary":{"windowDurationMins":300,"usedPercent":18}}}}}));
                // 成功也保持存活，让客户端负责终止整个作业；不能靠自行退出蒙混通过。
                pause();
            },
            _=>panic!("非预期协议消息"),
        }
    }
}
