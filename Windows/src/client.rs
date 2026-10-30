use crate::model::{normalize, QuotaWindow};
use serde_json::{json, Value};
use std::{path::{Path, PathBuf}, process::Stdio, time::Duration};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, process::Command, sync::watch};

pub const TIMEOUT: Duration = Duration::from_secs(25);
const LINE_LIMIT: usize = 1_048_576;
const TOTAL_LIMIT: usize = 4 * LINE_LIMIT;
const CONNECTION: &str = "同步失败，请检查原生 Windows Codex 登录与网络";
const CLEANUP_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Debug)]
pub struct Launch { pub executable: PathBuf, pub arguments: Vec<String> }

/// 只发现 Windows 原生入口；不读取凭据、不尝试 WSL，也不执行 PATH 中的其他脚本。
pub fn discover() -> Result<Launch, String> {
    let dirs = std::env::var_os("PATH").map(|v| std::env::split_paths(&v).collect::<Vec<_>>()).unwrap_or_default();
    discover_in(&dirs)
}
pub fn discover_in(dirs: &[PathBuf]) -> Result<Launch, String> {
    for name in ["codex.exe", "codex.cmd"] {
        for dir in dirs.iter().filter(|d| d.is_absolute()) {
            let path=dir.join(name);
            if path.is_file() { return native_launch(&path); }
        }
    }
    Err("未找到原生 Windows Codex CLI；请在 Windows 中安装并登录。WSL 登录不通用".into())
}
pub fn native_launch(path: &Path) -> Result<Launch, String> {
    if !path.is_absolute() || !path.is_file() { return Err("Codex 路径必须是现有的绝对路径".into()); }
    let extension=path.extension().and_then(|v|v.to_str()).unwrap_or("").to_ascii_lowercase();
    if extension != "exe" && extension != "cmd" { return Err("仅支持原生 .exe 或 .cmd 入口".into()); }
    // cmd.exe 的展开规则不同于 argv；拒绝可触发展开/注入的路径，不静默改写用户路径。
    if extension=="cmd" { cmd_line(path)?; }
    Ok(Launch {executable:path.into(),arguments:vec!["app-server".into(),"--listen".into(),"stdio://".into()]})
}

fn cmd_line(path:&Path)->Result<String,String>{
    // 不允许有损 Unicode 转换把路径悄悄改成另一个文件名。
    let path=path.to_str().ok_or("此 .cmd 路径不是有效 Unicode，请改用 codex.exe")?;
    if path.chars().any(|c|c.is_control()||['%', '!', '&', '|', '<', '>', '^', '"'].contains(&c)) {
        return Err("此 .cmd 路径包含不支持的命令解释器字符，请改用 codex.exe".into());
    }
    Ok(format!("\"\"{path}\" app-server --listen stdio://\""))
}

fn command(launch: &Launch) -> Result<Command,String> {
    #[cfg(windows)]
    if launch.executable.extension().is_some_and(|v| v.eq_ignore_ascii_case("cmd")) {
        // Launch 同时用于无账号测试，不能只信任调用者曾执行 native_launch。
        native_launch(&launch.executable)?;
        if launch.arguments!=["app-server","--listen","stdio://"] {return Err(".cmd 入口只接受固定额度读取参数".into());}
        let interpreter=std::env::var_os("SystemRoot").map(PathBuf::from).ok_or("无法定位系统命令解释器")?.join("System32/cmd.exe");
        if !interpreter.is_absolute()||!interpreter.is_file(){return Err("系统命令解释器路径无效".into());}
        let mut c=Command::new(interpreter);
        // 禁用 AutoRun 和延迟展开；固定参数不能来自前端任意输入。
        c.args(["/d", "/v:off", "/s", "/c"]);
        c.raw_arg(cmd_line(&launch.executable)?);
        return Ok(c);
    }
    let mut c=Command::new(&launch.executable); c.args(&launch.arguments); Ok(c)
}

async fn stop_child(child:&mut tokio::process::Child)->Result<(),String>{
    // start_kill 失败时仍尝试回收：进程可能已经退出，wait 的结果才是最终证据。
    let _=child.start_kill();
    match tokio::time::timeout(CLEANUP_TIMEOUT,child.wait()).await {
        Ok(Ok(_))=>Ok(()),
        Ok(Err(_))=>Err("无法确认额度进程已清理，请退出应用后重试".into()),
        Err(_)=>Err("额度进程清理超时，请退出应用后重试".into()),
    }
}

pub async fn read(launch: &Launch, directory: &Path, timeout: Duration, mut cancel: watch::Receiver<bool>) -> Result<Vec<QuotaWindow>, String> {
    if *cancel.borrow() { return Err("额度读取已取消".into()); }
    let deadline=tokio::time::Instant::now()+timeout;
    std::fs::create_dir_all(directory).map_err(|_|"无法创建本地工作目录")?;
    let mut cmd=command(launch)?;
    cmd.current_dir(directory).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).kill_on_drop(true);
    #[cfg(windows)] cmd.creation_flags(0x08000000 | 0x00000004); // 无控制台，挂起后先加入作业对象。
    #[cfg(unix)] cmd.process_group(0);
    let mut child=cmd.spawn().map_err(|_|"无法启动额度读取程序")?;
    let id=child.id().ok_or("无法获取子进程")?;
    #[cfg(unix)] let mut group=ProcessGroup(Some(id));
    #[cfg(windows)]
    let mut job=match windows_job::attach_and_resume(id) {
        Ok(job)=>job,
        Err(())=>{ stop_child(&mut child).await?; return Err("无法隔离额度进程，已终止启动".into()); }
    };
    let mut input=child.stdin.take().ok_or(CONNECTION)?;
    let mut output=child.stdout.take().ok_or(CONNECTION)?;
    let protocol=async {
        async fn send(input: &mut tokio::process::ChildStdin, value: Value) -> Result<(),String> {
            let mut bytes=serde_json::to_vec(&value).map_err(|_|CONNECTION)?; bytes.push(b'\n');
            input.write_all(&bytes).await.map_err(|_|CONNECTION.into())
        }
        send(&mut input,json!({"id":1,"method":"initialize","params":{"clientInfo":{"name":"codex_hud_windows","title":"Codex HUD","version":"0.1.0"},"capabilities":null}})).await?;
        let mut pending=Vec::new(); let mut chunk=[0u8;4096]; let mut total=0; let mut initialized=false;
        loop {
            let n=output.read(&mut chunk).await.map_err(|_|CONNECTION)?;
            if n==0 {return Err(CONNECTION.into());}
            total+=n; if total>TOTAL_LIMIT {return Err("额度响应过大".into());}
            pending.extend_from_slice(&chunk[..n]);
            while let Some(end)=pending.iter().position(|b|*b==b'\n') {
                if end>=LINE_LIMIT {return Err("额度响应行过长".into());}
                let line:Vec<_>=pending.drain(..=end).collect();
                let Ok(message)=serde_json::from_slice::<Value>(&line) else {continue;};
                if message.get("method").and_then(Value::as_str).is_some() {
                    if let Some(id)=message.get("id") {
                        send(&mut input,json!({"id":id,"error":{"code":-32601,"message":"Unsupported method"}})).await?;
                    }
                    continue;
                }
                // Swift 接受 1 和 1.0，但不接受 true、字符串或小数 id。
                let Some(id)=message.get("id").and_then(Value::as_f64) else {continue;};
                if !((id==1.0 && !initialized)||(id==2.0 && initialized)) {continue;}
                if message.get("error").is_some() {return Err(CONNECTION.into());}
                let Some(result)=message.get("result").filter(|r|r.is_object()) else {return Err(CONNECTION.into());};
                if id==1.0 {
                    initialized=true;
                    send(&mut input,json!({"method":"initialized"})).await?;
                    send(&mut input,json!({"id":2,"method":"account/rateLimits/read"})).await?;
                } else {return Ok(normalize(result));}
            }
            if pending.len()>=LINE_LIMIT {return Err("额度响应行过长".into());}
        }
    };
    let result=tokio::select! {
        result=tokio::time::timeout_at(deadline,protocol)=>result.unwrap_or_else(|_|Err("额度读取超时，请稍后重试".into())),
        _=cancel.changed()=>Err("额度读取已取消".into()),
    };
    drop(input); drop(output);
    #[cfg(unix)] let group_cleanup=group.terminate();
    #[cfg(windows)] let job_cleanup=match tokio::time::timeout(CLEANUP_TIMEOUT,job.terminate()).await {
        Ok(result)=>result,
        Err(_)=>Err("额度进程树清理超时，请退出应用后重试".into()),
    };
    #[cfg(windows)] drop(job); // 清理失败时仍由 kill-on-close 兜底，不把兜底当作已确认成功。
    stop_child(&mut child).await?;
    #[cfg(unix)] if group_cleanup.is_err() {
        // macOS 对仅含未回收僵尸的进程组可能返回 EPERM。回收直接子进程后，
        // 只用信号 0 确认组已消失；不能凭父进程退出就假定孙进程也已结束。
        if !process_group_missing(id){return Err("无法确认额度子进程组已清理".into());}
    }
    #[cfg(windows)] job_cleanup?;
    result
}

#[cfg(unix)] struct ProcessGroup(Option<u32>);
#[cfg(unix)] fn process_group_missing(id:u32)->bool{
    (unsafe{libc::kill(-(id as i32),0)})<0&&std::io::Error::last_os_error().raw_os_error()==Some(libc::ESRCH)
}
#[cfg(unix)] impl ProcessGroup {
    fn terminate(&mut self)->Result<(),String>{
        // 无论成功失败都只发送一次，避免直接子进程回收后 Drop 再次使用旧 PID。
        let Some(id)=self.0.take() else{return Ok(());};
        if unsafe{libc::kill(-(id as i32),libc::SIGKILL)}==0||std::io::Error::last_os_error().raw_os_error()==Some(libc::ESRCH){Ok(())}
        else{Err("无法确认额度子进程组已清理".into())}
    }
}
// 包括 future 被取消/丢弃等提前返回路径，避免仅终止直接子进程。
#[cfg(unix)] impl Drop for ProcessGroup {fn drop(&mut self){let _=self.terminate();}}

#[cfg(test)] mod launch_tests {
    use super::*;
    #[test] fn cmd_quotes_spaces_unicode_and_parentheses(){
        assert_eq!(cmd_line(Path::new(r"C:\Program Files (x86)\中文\codex.cmd")).unwrap(),r#"""C:\Program Files (x86)\中文\codex.cmd" app-server --listen stdio://""#);
    }
    #[test] fn cmd_rejects_interpreter_expansion_and_control_characters(){
        for character in ['%', '!', '&', '|', '<', '>', '^', '"', '\r', '\n', '\t', '\0'] {
            assert!(cmd_line(Path::new(&format!("C:\\bad{character}path\\codex.cmd"))).is_err());
        }
    }
    #[cfg(unix)] #[test] fn cmd_rejects_lossy_paths(){
        use std::os::unix::ffi::OsStringExt;
        let path=PathBuf::from(std::ffi::OsString::from_vec(b"/tmp/invalid-\xff.cmd".to_vec()));
        assert!(cmd_line(&path).is_err());
    }
}

#[cfg(windows)] mod windows_job {
    use windows_sys::Win32::{Foundation::{CloseHandle,HANDLE,INVALID_HANDLE_VALUE},System::{JobObjects::*,Threading::*,Diagnostics::ToolHelp::*}};
    pub struct Job(HANDLE);
    // 句柄独占所有权可移动到异步任务；不实现 Clone/Sync，Drop 只关闭一次。
    unsafe impl Send for Job {}
    impl Drop for Job {fn drop(&mut self){unsafe{CloseHandle(self.0);}}}
    impl Job {
        pub async fn terminate(&mut self)->Result<(),String>{
            if unsafe{TerminateJobObject(self.0,1)}==0{return Err("无法终止额度进程树".into());}
            loop {
                let empty=unsafe{
                    let mut info:JOBOBJECT_BASIC_ACCOUNTING_INFORMATION=std::mem::zeroed();
                    if QueryInformationJobObject(self.0,JobObjectBasicAccountingInformation,&mut info as *mut _ as _,std::mem::size_of_val(&info)as u32,std::ptr::null_mut())==0{return Err("无法确认额度进程树已清理".into());}
                    info.ActiveProcesses==0
                };
                if empty{return Ok(());}
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
    }
    /// 主线程恢复之前绑定作业对象，避免包装器抢先派生逃逸进程。
    pub fn attach_and_resume(pid:u32)->Result<Job,()> {unsafe{
        let handle=CreateJobObjectW(std::ptr::null(),std::ptr::null()); if handle.is_null(){return Err(());}
        let job=Job(handle);
        let mut info:JOBOBJECT_EXTENDED_LIMIT_INFORMATION=std::mem::zeroed();
        info.BasicLimitInformation.LimitFlags=JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if SetInformationJobObject(handle,JobObjectExtendedLimitInformation,&info as *const _ as _,std::mem::size_of_val(&info) as u32)==0{return Err(());}
        let process=OpenProcess(PROCESS_SET_QUOTA|PROCESS_TERMINATE,0,pid); if process.is_null(){return Err(());}
        let assigned=AssignProcessToJobObject(handle,process); CloseHandle(process); if assigned==0{return Err(());}
        let snapshot=CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD,0); if snapshot==INVALID_HANDLE_VALUE{return Err(());}
        let mut entry:THREADENTRY32=std::mem::zeroed();entry.dwSize=std::mem::size_of_val(&entry) as u32;
        let mut found=false;let mut valid=Thread32First(snapshot,&mut entry);
        while valid!=0 {
            if entry.th32OwnerProcessID==pid {
                let thread=OpenThread(THREAD_SUSPEND_RESUME,0,entry.th32ThreadID);
                if !thread.is_null(){found=ResumeThread(thread)!=u32::MAX;CloseHandle(thread);} break;
            }
            valid=Thread32Next(snapshot,&mut entry);
        }
        CloseHandle(snapshot);if found{Ok(job)}else{Err(())}
    }}
}
