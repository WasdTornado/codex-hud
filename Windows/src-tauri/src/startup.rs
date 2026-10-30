use tauri::{AppHandle,Manager};

/// 初始化期间 IPC 允许先于受管状态到达；稍后由订阅事件补发状态。
pub fn snapshot<R:tauri::Runtime,T:Send+Sync+'static,O>(app:&AppHandle<R>,read:impl FnOnce(&T)->O)->Option<O>{
    app.try_state::<T>().map(|state|read(&state))
}
