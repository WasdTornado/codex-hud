use serde_json::{json,Value};
use std::{path::PathBuf,time::Duration};
pub struct Fixture {pub root:PathBuf}
impl Fixture {
    pub fn new(mode:&str,child:bool,gate:bool)->Self{
        static SEQUENCE:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
        let root=std::env::temp_dir().join(format!("codex HUD 空格-{}-{}",std::process::id(),SEQUENCE.fetch_add(1,std::sync::atomic::Ordering::Relaxed)));
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("fixture.json"),json!({"mode":mode,"child":child,"gate":gate}).to_string()).unwrap();Self{root}
    }
    #[allow(dead_code)] pub async fn ready(&self)->Value{
        tokio::time::timeout(Duration::from_secs(5),async{
            loop{if let Ok(bytes)=std::fs::read(self.root.join("ready.json")){if let Ok(value)=serde_json::from_slice(&bytes){return value;}}
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }).await.unwrap_or_else(|_|panic!("假 CLI 未启动到 ready；wrapper={:?}; fixture error={:?}",std::fs::read_to_string(self.root.join("wrapper.txt")),std::fs::read_to_string(self.root.join("fixture-error.txt"))))
    }
}
impl Drop for Fixture{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.root);}}
