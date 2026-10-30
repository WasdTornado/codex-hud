use codex_hud_core::client::{self, Launch};
use std::{path::PathBuf,time::{Duration,Instant}};
use tokio::sync::watch;
fn launch(mode:&str)->Launch {
    Launch {executable:std::env::var_os("HUD_TEST_PYTHON").map(PathBuf::from).unwrap_or_else(||PathBuf::from(if cfg!(windows){"python.exe"}else{"/usr/bin/python3"})),arguments:vec![format!("{}/tests/fake_cli.py",env!("CARGO_MANIFEST_DIR")),mode.into()]}
}
fn directory()->PathBuf{std::env::temp_dir().join(format!("codex-hud-fake-{}",std::process::id()))}
#[tokio::test] async fn protocol_noise_notifications_and_server_requests(){
    let (_tx,rx)=watch::channel(false);
    let values=client::read(&launch("success"),&directory(),Duration::from_secs(3),rx).await.unwrap();
    assert_eq!(values.iter().map(|v|v.remaining).collect::<Vec<_>>(),vec![Some(82.0),Some(47.0)]);
}
#[tokio::test] async fn failure_is_sanitized(){
    for (mode,expected) in [("error","登录"),("eof","登录"),("oversize","过长"),("total","过大")] {
        let (_tx,rx)=watch::channel(false);
        let error=client::read(&launch(mode),&directory(),Duration::from_secs(3),rx).await.unwrap_err();
        assert!(!error.contains("secret-token"));assert!(error.contains(expected),"{mode}: {error}");
    }
}
#[tokio::test] async fn floating_point_ids_match_swift(){
    let (_tx,rx)=watch::channel(false);
    let values=client::read(&launch("float-id"),&directory(),Duration::from_secs(3),rx).await.unwrap();
    assert_eq!(values[0].remaining,Some(82.0));
}
#[tokio::test] async fn timeout_is_bounded(){
    let (_tx,rx)=watch::channel(false);let start=Instant::now();
    let error=client::read(&launch("timeout"),&directory(),Duration::from_millis(150),rx).await.unwrap_err();
    assert!(error.contains("超时"));assert!(start.elapsed()<Duration::from_secs(3));
}
#[tokio::test] async fn cancellation_ends_read(){
    let (tx,rx)=watch::channel(false);
    let task=tokio::spawn(async move {client::read(&launch("timeout"),&directory(),Duration::from_secs(25),rx).await});
    tokio::time::sleep(Duration::from_millis(100)).await;tx.send(true).unwrap();
    assert!(task.await.unwrap().unwrap_err().contains("取消"));
}
#[tokio::test] async fn cancellation_before_spawn(){
    let (_tx,rx)=watch::channel(true);
    assert!(client::read(&launch("success"),&directory(),Duration::from_secs(1),rx).await.unwrap_err().contains("取消"));
}

// 这里只验证当前 Unix 主机的进程组；不能将结果记为 Windows Job Object 通过。
#[cfg(unix)] async fn tree_fixture(mode:&str)->(Launch,PathBuf){
    let file=directory().join(format!("child-{mode}.pid"));
    std::fs::create_dir_all(directory()).unwrap();let _=std::fs::remove_file(&file);
    let mut launch=launch("child");launch.arguments.push(file.to_string_lossy().into_owned());(launch,file)
}
#[cfg(unix)] async fn wait_child(file:&std::path::Path)->String{
    tokio::time::timeout(Duration::from_secs(3),async{
        loop{if let Ok(pid)=std::fs::read_to_string(file){if pid.parse::<u32>().is_ok(){return pid;}}tokio::time::sleep(Duration::from_millis(20)).await;}
    }).await.expect("假 CLI 必须先创建孙进程")
}
#[cfg(unix)] async fn assert_child_stopped(pid:&str){
    tokio::time::timeout(Duration::from_secs(3),async{
        loop{
            // 只查询本测试创建的 PID；僵尸已停止执行，等待系统回收不算仍在运行。
            let output=tokio::process::Command::new("/bin/ps").args(["-o","stat=","-p",pid]).output().await.unwrap();
            let state=String::from_utf8_lossy(&output.stdout);
            if state.trim().is_empty()||state.trim_start().starts_with('Z'){return;}
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }).await.expect("假 CLI 的孙进程应已停止");
}
#[cfg(unix)] #[tokio::test] async fn unix_timeout_stops_grandchild(){
    let (launch,file)=tree_fixture("timeout").await;let (_tx,rx)=watch::channel(false);
    assert!(client::read(&launch,&directory(),Duration::from_secs(1),rx).await.unwrap_err().contains("超时"));
    assert_child_stopped(&wait_child(&file).await).await;
}
#[cfg(unix)] #[tokio::test] async fn unix_cancellation_stops_grandchild(){
    let (launch,file)=tree_fixture("cancel").await;let (tx,rx)=watch::channel(false);
    let task=tokio::spawn(async move{client::read(&launch,&directory(),Duration::from_secs(25),rx).await});
    let pid=wait_child(&file).await;tx.send(true).unwrap();assert!(task.await.unwrap().unwrap_err().contains("取消"));
    assert_child_stopped(&pid).await;
}
#[cfg(unix)] #[tokio::test] async fn unix_aborted_future_stops_grandchild(){
    let (launch,file)=tree_fixture("abort").await;let (_tx,rx)=watch::channel(false);
    let task=tokio::spawn(async move{client::read(&launch,&directory(),Duration::from_secs(25),rx).await});
    let pid=wait_child(&file).await;task.abort();assert!(task.await.unwrap_err().is_cancelled());assert_child_stopped(&pid).await;
}
