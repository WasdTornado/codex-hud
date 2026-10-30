use serde::{Deserialize,Serialize};
use std::path::Path;

#[derive(Default,Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(default,rename_all="camelCase")]
pub struct Preferences {
    pub hidden:bool,
    pub automatic_updates:bool,
    pub position:Option<(i32,i32)>,
}
impl Preferences {
    pub fn load(directory:&Path)->(Self,Option<String>){
        match std::fs::read(directory.join("preferences.json")){
            Ok(bytes)=>match serde_json::from_slice(&bytes){
                Ok(p)=>(p,None),
                Err(_)=>(Self{hidden:true,..Self::default()},Some("偏好文件损坏，浮窗保持隐藏；可从托盘恢复".into())),
            },
            Err(e) if e.kind()==std::io::ErrorKind::NotFound=>(Self::default(),None),
            Err(_)=>(Self{hidden:true,..Self::default()},Some("无法读取偏好，浮窗保持隐藏".into())),
        }
    }
    /// 调用方持有偏好锁直至落盘；写入或替换失败都保留旧内存状态。
    pub fn update(&mut self,directory:&Path,change:impl FnOnce(&mut Self))->Result<(),String>{
        let mut next=self.clone();change(&mut next);
        let bytes=serde_json::to_vec(&next).map_err(|_|"无法保存偏好")?;
        let temp=directory.join("preferences.tmp");
        std::fs::write(&temp,bytes).map_err(|_|"无法写入偏好")?;
        if std::fs::rename(&temp,directory.join("preferences.json")).is_err(){
            let _=std::fs::remove_file(temp);return Err("无法替换偏好".into());
        }
        *self=next;Ok(())
    }
}

#[cfg(test)] mod tests {
    use super::*;
    struct Directory(std::path::PathBuf);
    impl Directory {fn new()->Self{
        static SEQUENCE:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
        let path=std::env::temp_dir().join(format!("codex-hud-prefs-{}-{}",std::process::id(),SEQUENCE.fetch_add(1,std::sync::atomic::Ordering::Relaxed)));
        std::fs::create_dir(&path).unwrap();Self(path)
    }}
    impl Drop for Directory{fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
    #[test] fn hidden_update_and_position_survive_replacement(){
        let dir=Directory::new();let mut prefs=Preferences::default();
        prefs.update(&dir.0,|p|p.hidden=true).unwrap();
        prefs.update(&dir.0,|p|{p.automatic_updates=true;p.position=Some((-100,200));}).unwrap();
        assert_eq!(Preferences::load(&dir.0),(prefs,None));
    }
    #[test] fn failed_write_and_replacement_preserve_memory_and_saved_state(){
        let dir=Directory::new();let mut prefs=Preferences{hidden:true,..Default::default()};let old=prefs.clone();
        assert!(prefs.update(&dir.0.join("missing"),|p|p.hidden=false).is_err());assert_eq!(prefs,old);
        std::fs::create_dir(dir.0.join("preferences.json")).unwrap();
        assert!(prefs.update(&dir.0,|p|p.automatic_updates=true).is_err());assert_eq!(prefs,old);
        assert!(dir.0.join("preferences.json").is_dir());assert!(!dir.0.join("preferences.tmp").exists());
    }
    #[test] fn corrupt_or_unreadable_preferences_start_hidden(){
        let dir=Directory::new();assert!(!Preferences::load(&dir.0).0.hidden);
        std::fs::write(dir.0.join("preferences.json"),b"invalid").unwrap();
        let (prefs,warning)=Preferences::load(&dir.0);assert!(prefs.hidden);assert!(warning.is_some());
        std::fs::remove_file(dir.0.join("preferences.json")).unwrap();std::fs::create_dir(dir.0.join("preferences.json")).unwrap();
        let (prefs,warning)=Preferences::load(&dir.0);assert!(prefs.hidden);assert!(warning.is_some());
    }
}
