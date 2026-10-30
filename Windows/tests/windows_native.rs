// 本文件只在真实 Windows + test-fixtures 下收集；Mac 的 0 tests 绝不是通过。
#![cfg(all(windows,feature="test-fixtures"))]
#[path="support/harness.rs"] mod harness;
use codex_hud_core::{client,model::QuotaWindow};
use std::{path::Path,time::{Duration,Instant}};
use tokio::sync::watch;
use windows_sys::Win32::{Foundation::{CloseHandle,HANDLE},System::Threading::{OpenProcess,TerminateProcess,WaitForSingleObject,PROCESS_TERMINATE}};

fn native_launch(root:&Path,cmd:bool)->client::Launch{
    let name=if cmd{"fixture.exe"}else{"codex.exe"};
    std::fs::copy(env!("CARGO_BIN_EXE_hud-fake-cli"),root.join(name)).unwrap();
    if cmd{
        // ASCII 包装器通过自身目录定位原生假服务；空格/Unicode 均来自真实文件路径。
        std::fs::write(root.join("codex.cmd"),b"@echo off\r\necho wrapper-started> wrapper.txt\r\n\"%~dp0fixture.exe\" %* 2> fixture-error.txt\r\n").unwrap();
    }
    if cmd {
        eprintln!("cmd fixture path={}",root.display());
        // 只探测固定 echo，捕获 cmd 对测试路径/引号的诊断；不调用真实 CLI。
        use std::os::windows::process::CommandExt;
        let mut probe=std::process::Command::new(std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join("System32/cmd.exe"));
        let output=probe.args(["/d","/v:off","/s","/c"]).raw_arg(format!("\"\"{}\"\"",root.join("probe.cmd").display()));
        std::fs::write(root.join("probe.cmd"),b"@echo probe-ok\r\n").unwrap();
        let output=output.output().unwrap();
        eprintln!("probe status={} stdout={} stderr={}",output.status,String::from_utf8_lossy(&output.stdout),String::from_utf8_lossy(&output.stderr));
    }
    let launch=client::discover_in(&[root.to_path_buf()]).unwrap();
    assert_eq!(launch.executable,root.join(if cmd{"codex.cmd"}else{"codex.exe"}));launch
}
struct ReadTask(tokio::task::JoinHandle<Result<Vec<QuotaWindow>,String>>);
impl Drop for ReadTask{fn drop(&mut self){self.0.abort();}}
struct Process(HANDLE);
impl Process{
    fn open(pid:u32)->Self{
        // SYNCHRONIZE + TERMINATE；持有准确进程对象，禁止在退出后重新按 PID 猜测。
        let handle=unsafe{OpenProcess(0x00100000|PROCESS_TERMINATE,0,pid)};
        assert!(!handle.is_null(),"无法取得假服务进程句柄，不能判定通过");Self(handle)
    }
    async fn assert_stopped(&self){
        tokio::time::timeout(Duration::from_secs(5),async{
            loop{match unsafe{WaitForSingleObject(self.0,0)}{
                0=>return,0x102=>tokio::time::sleep(Duration::from_millis(10)).await,
                _=>panic!("无法查询假服务进程退出状态"),
            }}
        }).await.expect("客户端未清理假服务/孙进程；测试兜底清理不计成功");
    }
}
impl Drop for Process{fn drop(&mut self){unsafe{
    // 失败也只清理本测试已打开的句柄，不查询或终止真实 Codex。
    if WaitForSingleObject(self.0,0)==0x102{TerminateProcess(self.0,1);WaitForSingleObject(self.0,5000);}
    CloseHandle(self.0);
}}}

async fn tree_case(cmd:bool,finish:&str){
    eprintln!("fixture entry={} finish={finish}",if cmd{"cmd"}else{"exe"});
    let fixture=harness::Fixture::new(if finish=="success"{"success"}else{"timeout"},true,true);
    let launch=native_launch(&fixture.root,cmd);let root=fixture.root.clone();let (tx,rx)=watch::channel(false);
    let started=Instant::now();
    let mut task=ReadTask(tokio::spawn(async move{client::read(&launch,&root,Duration::from_secs(8),rx).await}));
    let ready=fixture.ready().await;
    let parent=Process::open(ready["parent"].as_u64().unwrap().try_into().unwrap());
    let child=Process::open(ready["child"].as_u64().unwrap().try_into().unwrap());
    std::fs::write(fixture.root.join("continue"),b"go").unwrap();
    if finish=="abort"{task.0.abort();assert!((&mut task.0).await.unwrap_err().is_cancelled());}
    else{
        if finish=="cancel"{tx.send_replace(true);}
        let result=tokio::time::timeout(Duration::from_secs(13),&mut task.0).await.expect("读取/清理超出测试预算").unwrap();
        if finish=="success"{assert_eq!(result.unwrap().iter().map(|w|w.remaining).collect::<Vec<_>>(),vec![Some(82.0),Some(47.0)]);}
        else{assert!(result.unwrap_err().contains(if finish=="cancel"{"取消"}else{"超时"}));}
    }
    parent.assert_stopped().await;child.assert_stopped().await;
    assert!(started.elapsed()<Duration::from_secs(23));
}
#[tokio::test] async fn windows_exe_cmd_success_noise_and_tree_cleanup(){for cmd in [false,true]{tree_case(cmd,"success").await;}}
#[tokio::test] async fn windows_exe_cmd_timeout_tree_cleanup(){for cmd in [false,true]{tree_case(cmd,"timeout").await;}}
#[tokio::test] async fn windows_exe_cmd_cancellation_tree_cleanup(){for cmd in [false,true]{tree_case(cmd,"cancel").await;}}
#[tokio::test] async fn windows_exe_cmd_aborted_future_tree_cleanup(){for cmd in [false,true]{tree_case(cmd,"abort").await;}}
#[tokio::test] async fn windows_exe_cmd_precancel_never_spawns(){
    for cmd in [false,true]{
        let fixture=harness::Fixture::new("success",true,false);let launch=native_launch(&fixture.root,cmd);let (_tx,rx)=watch::channel(true);
        assert!(client::read(&launch,&fixture.root,Duration::from_secs(8),rx).await.unwrap_err().contains("取消"));
        assert!(!fixture.root.join("ready.json").exists());
    }
}
#[tokio::test] async fn windows_exe_cmd_errors_and_output_limits(){
    for cmd in [false,true]{for (mode,expected) in [("error","登录"),("eof","登录"),("oversize","过长"),("total","过大")]{
        let fixture=harness::Fixture::new(mode,false,false);let launch=native_launch(&fixture.root,cmd);let (_tx,rx)=watch::channel(false);
        let error=client::read(&launch,&fixture.root,Duration::from_secs(8),rx).await.unwrap_err();
        assert!(error.contains(expected),"{mode}: {error}");assert!(!error.contains("secret-token"));
    }}
}
#[test] fn windows_cmd_metacharacters_are_rejected_without_execution(){
    let fixture=harness::Fixture::new("success",false,false);
    for name in ["bad&name.cmd","bad%name.cmd","bad!name.cmd","bad^name.cmd"]{
        let path=fixture.root.join(name);std::fs::write(&path,b"@exit /b 99\r\n").unwrap();
        assert!(client::native_launch(&path).is_err());
    }
    assert!(!fixture.root.join("ready.json").exists());
}
