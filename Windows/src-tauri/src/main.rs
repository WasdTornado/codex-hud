#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Mac 只编译检查共享宿主逻辑，不运行 Windows 应用；平台专属分支仍待 Windows 验收。
#![cfg_attr(not(windows), allow(dead_code))]
mod platform;
mod tray;
mod updates;
mod preferences;
use preferences::Preferences;
use codex_hud_core::{client,model::QuotaWindow,state::Snapshot};
use serde::Serialize;
use std::{path::PathBuf,sync::Mutex,time::{Duration,SystemTime,UNIX_EPOCH}};
use tauri::{AppHandle,Emitter,Manager,LogicalSize,PhysicalPosition,WindowEvent};
use tauri::menu::{Menu,MenuItem,CheckMenuItem};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::watch;

#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
struct ViewState {
    quota:Snapshot,hidden:bool,dark:bool,expanded:bool,preview:bool,
    drag_threshold:f64,update_available:Option<String>,platform_warning:Option<String>,
}
struct Runtime {
    data:Mutex<ViewState>,preferences:Mutex<Preferences>,path:PathBuf,
    cancel:watch::Sender<bool>,auto_update_busy:std::sync::atomic::AtomicBool,
}
fn now()->u64{SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()as u64}
fn update_preferences(app:&AppHandle,change:impl FnOnce(&mut Preferences))->Result<(),String>{
    let runtime=app.state::<Runtime>();
    let result=runtime.preferences.lock().unwrap().update(&runtime.path,change);result
}
fn report(app:&AppHandle,error:String){app.state::<Runtime>().data.lock().unwrap().platform_warning=Some(error);publish(app);}

pub(crate) fn publish(app:&AppHandle){
    let state=app.state::<Runtime>().data.lock().unwrap().clone();
    let _=app.emit("hud-state",&state);
    let _=update_menu(app);
    if let Some(tray)=app.tray_by_id("quota"){
        let _=tray.set_icon(Some(tray::image(&state.quota,platform::tray_dark(),state.update_available.is_some()||state.platform_warning.is_some())));
        let summary=state.quota.windows.iter().map(|w|format!("{} {}",match w.minutes{Some(300)=>"5h",Some(10080)=>"周",_=>"额度"},w.remaining.map(|v|format!("{:.0}%",v.round())).unwrap_or_else(||"未知".into()))).collect::<Vec<_>>().join(" · ");
        let _=tray.set_tooltip(Some(format!("Codex HUD{} · {summary}{}{}",if state.preview{" · 演示"}else{""},if state.quota.error.is_some(){" · 同步异常"}else{""},if state.update_available.is_some(){" · 有更新"}else{""})));
    }
}
fn render_windows(app:&AppHandle)->Result<(),String>{
    let state=app.state::<Runtime>().data.lock().unwrap().clone();
    let orb=app.get_webview_window("orb").ok_or("找不到浮窗")?;
    let height=if state.quota.windows.len()>1{72.0}else{36.0};
    orb.set_size(LogicalSize::new(36.0,height)).map_err(|_|"无法设置浮窗尺寸")?;
    platform::shape(&orb)?;platform::clamp_and_dock(app,None)?;
    let details=app.get_webview_window("details").ok_or("找不到详情窗")?;
    platform::shape(&details)?;
    if state.hidden {orb.hide().map_err(|_|"无法隐藏浮窗")?;details.hide().map_err(|_|"无法隐藏详情")?;}
    // 刷新、唤醒和主题更新绝不调用 show；只有启动和用户主动显示才可改变可见性。
    if state.expanded&&!state.hidden{position_details(app)?;}
    Ok(())
}
fn position_details(app:&AppHandle)->Result<(),String>{
    let orb=app.get_webview_window("orb").ok_or("找不到浮窗")?;let details=app.get_webview_window("details").ok_or("找不到详情")?;
    let monitor=orb.current_monitor().map_err(|_|"无法读取显示器")?.ok_or("没有显示器")?;
    let area=monitor.work_area();let p=orb.outer_position().map_err(|_|"无法读取位置")?;let s=orb.outer_size().map_err(|_|"无法读取尺寸")?;
    let d=details.outer_size().map_err(|_|"无法读取详情尺寸")?;let gap=(8.0*monitor.scale_factor())as i32;
    let x=if p.x<area.position.x+area.size.width as i32/2{p.x+s.width as i32+gap}else{p.x-d.width as i32-gap};
    let max_x=area.position.x+area.size.width.saturating_sub(d.width)as i32;let max_y=area.position.y+area.size.height.saturating_sub(d.height)as i32;
    details.set_position(PhysicalPosition::new(x.clamp(area.position.x,max_x),(p.y+(s.height as i32-d.height as i32)/2).clamp(area.position.y,max_y))).map_err(|_|"无法定位详情".into())
}
pub(crate) fn refresh_background(app:AppHandle){tauri::async_runtime::spawn(async move{let _=refresh(app).await;});}
#[tauri::command] async fn refresh(app:AppHandle)->Result<(),String>{
    let runtime=app.state::<Runtime>();
    if *runtime.cancel.borrow(){return Ok(());}
    let preview={let mut data=runtime.data.lock().unwrap();if !data.quota.begin(){return Ok(());}data.preview};publish(&app);
    let result=if preview{Ok(vec![QuotaWindow{minutes:Some(300),remaining:Some(82.0),resets_at:Some(1893456000.0)},QuotaWindow{minutes:Some(10080),remaining:Some(47.0),resets_at:Some(1894060800.0)}])}
        else{match client::discover(){Ok(launch)=>client::read(&launch,&runtime.path.join("cli"),client::TIMEOUT,runtime.cancel.subscribe()).await,Err(error)=>Err(error)}};
    runtime.data.lock().unwrap().quota.finish(result,now());
    let ui_app=app.clone();let _=app.run_on_main_thread(move||{if let Err(error)=render_windows(&ui_app){report(&ui_app,error);}publish(&ui_app);});Ok(())
}
// WebView2 创建期间可能先收到 IPC，此时 setup 尚未注册 Runtime。
// 前端先订阅 hud-state，再读取快照；未就绪返回 null，setup 的刷新会发布首个状态。
fn state_snapshot<R:tauri::Runtime>(app:&tauri::AppHandle<R>)->Option<ViewState>{
    app.try_state::<Runtime>().map(|runtime|runtime.data.lock().unwrap().clone())
}
#[tauri::command] fn get_state(app:AppHandle)->Option<ViewState>{state_snapshot(&app)}
#[tauri::command] fn set_hidden(app:AppHandle,hidden:bool)->Result<(),String>{
    let runtime=app.state::<Runtime>();
    // 先持久化意图；写盘失败时不假装已保存。
    update_preferences(&app,|prefs|prefs.hidden=hidden)?;
    {let mut state=runtime.data.lock().unwrap();state.hidden=hidden;state.expanded=false;}
    if let Some(details)=app.get_webview_window("details"){let _=details.hide();}
    let orb=app.get_webview_window("orb").ok_or("找不到浮窗")?;
    if hidden{orb.hide()}else{orb.show()}.map_err(|_|"无法切换浮窗")?;
    update_menu(&app)?;publish(&app);Ok(())
}
#[tauri::command] fn toggle_details(app:AppHandle)->Result<(),String>{
    let runtime=app.state::<Runtime>();
    {let data=runtime.data.lock().unwrap();if data.hidden{return Ok(());}if data.expanded{drop(data);return close_details(app.clone());}}
    runtime.data.lock().unwrap().expanded=true;position_details(&app)?;publish(&app);
    let details=app.get_webview_window("details").ok_or("找不到详情")?;details.show().map_err(|_|"无法显示详情")?;details.set_focus().map_err(|_|"无法聚焦详情".into())
}
#[tauri::command] fn close_details(app:AppHandle)->Result<(),String>{
    app.state::<Runtime>().data.lock().unwrap().expanded=false;publish(&app);
    tauri::async_runtime::spawn(async move{tokio::time::sleep(Duration::from_millis(170)).await;if !app.state::<Runtime>().data.lock().unwrap().expanded{if let Some(w)=app.get_webview_window("details"){let _=w.hide();}}});Ok(())
}
/// 详情 WebView 报告实际布局高度，避免固定高度留下多余透明/磨砂矩形。
#[tauri::command] fn resize_details(app:AppHandle,window:tauri::WebviewWindow,height:f64)->Result<(),String>{
    if window.label()!="details"||!height.is_finite()||!(64.0..=480.0).contains(&height){return Err("详情尺寸无效".into());}
    window.set_size(LogicalSize::new(192.0,height.ceil())).map_err(|_|"无法设置详情尺寸")?;
    platform::shape(&window)?;position_details(&app)
}
#[tauri::command] fn dock(app:AppHandle,right:bool)->Result<(),String>{if app.state::<Runtime>().data.lock().unwrap().hidden{return Ok(());}platform::clamp_and_dock(&app,Some(right))?;position_details(&app)}
#[tauri::command] fn start_drag(app:AppHandle)->Result<(),String>{close_details(app.clone())?;app.get_webview_window("orb").ok_or("找不到浮窗")?.start_dragging().map_err(|_|"无法拖动浮窗".into())}
#[tauri::command] fn check_updates(app:AppHandle){schedule_update(app,true);}
fn schedule_update(app:AppHandle,manual:bool){
    use std::sync::atomic::Ordering;
    if app.state::<Runtime>().auto_update_busy.swap(true,Ordering::SeqCst){return;}
    tauri::async_runtime::spawn(async move{if let Err(e)=updates::check(app.clone(),manual).await{report(&app,e);}app.state::<Runtime>().auto_update_busy.store(false,Ordering::SeqCst);});
}
fn update_menu(app:&AppHandle)->Result<(),String>{
    let hidden=app.state::<Runtime>().data.lock().unwrap().hidden;let auto=app.state::<Runtime>().preferences.lock().unwrap().automatic_updates;
    let refresh=MenuItem::with_id(app,"refresh","刷新额度",true,None::<&str>).map_err(|_|"无法创建菜单")?;
    let visible=CheckMenuItem::with_id(app,"visible","显示桌面悬浮窗",true,!hidden,None::<&str>).map_err(|_|"无法创建菜单")?;
    let details=MenuItem::with_id(app,"details","额度详情",!hidden,None::<&str>).map_err(|_|"无法创建菜单")?;
    let left=MenuItem::with_id(app,"left","停靠左侧",!hidden,None::<&str>).map_err(|_|"无法创建菜单")?;
    let right=MenuItem::with_id(app,"right","停靠右侧",!hidden,None::<&str>).map_err(|_|"无法创建菜单")?;
    let check=MenuItem::with_id(app,"update","检查更新…",true,None::<&str>).map_err(|_|"无法创建菜单")?;
    let automatic=CheckMenuItem::with_id(app,"automatic","自动检查更新",true,auto,None::<&str>).map_err(|_|"无法创建菜单")?;
    let about=MenuItem::with_id(app,"about","关于 Codex HUD",true,None::<&str>).map_err(|_|"无法创建菜单")?;
    let quit=MenuItem::with_id(app,"quit","退出",true,None::<&str>).map_err(|_|"无法创建菜单")?;
    let data=app.state::<Runtime>().data.lock().unwrap().clone();
    let summary=data.quota.windows.iter().map(|w|format!("{} 剩余 {}",match w.minutes{Some(300)=>"5h",Some(10080)=>"每周",_=>"额度"},w.remaining.map(|v|format!("{:.0}%",v.round())).unwrap_or_else(||"未知".into()))).collect::<Vec<_>>().join(" · ");
    let status=MenuItem::with_id(app,"status",if summary.is_empty(){"暂无额度"}else{&summary},false,None::<&str>).map_err(|_|"无法创建菜单")?;
    let sync=MenuItem::with_id(app,"sync",if data.quota.loading{"同步中"}else if data.quota.error.is_some(){"同步失败，保留上次成功数据"}else if data.quota.updated_at.is_some(){"已有成功同步数据"}else{"尚未成功同步"},false,None::<&str>).map_err(|_|"无法创建菜单")?;
    let menu=Menu::with_items(app,&[&status,&sync,&refresh,&visible,&details,&left,&right,&check,&automatic,&about,&quit]).map_err(|_|"无法创建菜单")?;
    if let Some(tray)=app.tray_by_id("quota"){tray.set_menu(Some(menu)).map_err(|_|"无法更新托盘菜单")?;}
    Ok(())
}
fn menu_action(app:&AppHandle,id:&str){
    let result=match id{
        "refresh"=>{refresh_background(app.clone());Ok(())},
        "visible"=>{let hidden=app.state::<Runtime>().data.lock().unwrap().hidden;set_hidden(app.clone(),!hidden)},
        "details"=>toggle_details(app.clone()),"left"=>dock(app.clone(),false),"right"=>dock(app.clone(),true),
        "update"=>{schedule_update(app.clone(),true);Ok(())},
        "automatic"=>update_preferences(app,|prefs|prefs.automatic_updates=!prefs.automatic_updates).and_then(|_|{
            update_menu(app)?;
            if app.state::<Runtime>().preferences.lock().unwrap().automatic_updates{schedule_update(app.clone(),false);}
            Ok(())
        }),
        "about"=>{app.dialog().message("Codex HUD · Windows 平台验证预览 0.1.0\nwasdTornado · MIT\n尚未完成 Windows 构建与实机验收。\n原生 Windows 与 WSL 登录独立。\n更新签名不等于 Windows 代码签名。").title("关于 Codex HUD").show(|_|{});Ok(())},
        "quit"=>{app.state::<Runtime>().cancel.send_replace(true);let app=app.clone();tauri::async_runtime::spawn(async move{for _ in 0..50{if !app.state::<Runtime>().data.lock().unwrap().quota.loading{break;}tokio::time::sleep(Duration::from_millis(100)).await;}app.exit(0);});Ok(())},
        _=>Ok(())};if let Err(e)=result{report(app,e);}
}

fn application_builder(real:bool)->tauri::Builder<tauri::Wry>{
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init()).plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![get_state,refresh,set_hidden,toggle_details,close_details,resize_details,dock,start_drag,check_updates])
        .setup(move|app|{
            let path=app.path().app_local_data_dir()?.join(if real{"native"}else{"demo"});std::fs::create_dir_all(&path)?;
            let (prefs,warning)=Preferences::load(&path);
            let orb=app.get_webview_window("orb").unwrap();let dark=orb.theme().ok()==Some(tauri::Theme::Dark);let (cancel,_)=watch::channel(false);
            app.manage(Runtime{data:Mutex::new(ViewState{quota:Snapshot::default(),hidden:prefs.hidden,dark,expanded:false,preview:!real,drag_threshold:platform::drag_threshold(&orb),update_available:None,platform_warning:warning}),preferences:Mutex::new(prefs.clone()),path,cancel,auto_update_busy:std::sync::atomic::AtomicBool::new(false)});
            tauri::tray::TrayIconBuilder::with_id("quota").icon(tray::image(&Snapshot::default(),platform::tray_dark(),false)).tooltip("Codex HUD · 平台验证预览").show_menu_on_left_click(true).on_menu_event(|app,event|menu_action(app,event.id.as_ref())).build(app)?;
            update_menu(app.handle())?;
            if let Some((x,y))=prefs.position{orb.set_position(PhysicalPosition::new(x,y))?;platform::clamp_and_dock(app.handle(),None)?;}else{orb.center()?;platform::clamp_and_dock(app.handle(),Some(true))?;}
            for label in ["orb","details"]{if let Some(window)=app.get_webview_window(label){if let Err(e)=platform::appearance(&window,dark){report(app.handle(),e);}}}
            if let Err(e)=platform::install_wake(&orb,app.handle().clone()){report(app.handle(),e);}
            if !prefs.hidden{orb.show()?;}
            let handle=app.handle().clone();refresh_background(handle.clone());
            tauri::async_runtime::spawn(async move{loop{tokio::time::sleep(Duration::from_secs(300)).await;refresh_background(handle.clone());}});
            if prefs.automatic_updates{schedule_update(app.handle().clone(),false);}
            let handle=app.handle().clone();tauri::async_runtime::spawn(async move{loop{tokio::time::sleep(Duration::from_secs(6*3600)).await;if handle.state::<Runtime>().preferences.lock().unwrap().automatic_updates{schedule_update(handle.clone(),false);}}});
            Ok(())
        })
        .on_window_event(|window,event|{
            let app=window.app_handle();
            if app.try_state::<Runtime>().is_none(){return;}
            match event{
                WindowEvent::CloseRequested{api,..}=>{api.prevent_close();let _=close_details(app.clone());},
                WindowEvent::Moved(position) if window.label()=="orb"=>{if app.try_state::<Runtime>().is_some(){if let Err(e)=update_preferences(app,|prefs|prefs.position=Some((position.x,position.y))){report(app,e);}let _=position_details(app);}},
                WindowEvent::ScaleFactorChanged{..}=>{if let Some(web)=app.get_webview_window(window.label()){if let Err(e)=platform::shape(&web){report(app,e);}if window.label()=="orb"{app.state::<Runtime>().data.lock().unwrap().drag_threshold=platform::drag_threshold(&web);publish(app);}}},
                WindowEvent::Resized(_)=>{if let Some(web)=app.get_webview_window(window.label()){if let Err(e)=platform::shape(&web){report(app,e);}}},
                WindowEvent::ThemeChanged(theme)=>{if let Some(runtime)=app.try_state::<Runtime>(){let dark=*theme==tauri::Theme::Dark;runtime.data.lock().unwrap().dark=dark;for label in ["orb","details"]{if let Some(web)=app.get_webview_window(label){if let Err(e)=platform::appearance(&web,dark){report(app,e);}}}publish(app);}},
                _=>{}
            }
        })
}
#[cfg(windows)] fn main(){
    let real=std::env::args().any(|a|a=="--real-cli");
    application_builder(real).run(tauri::generate_context!()).expect("Windows 平台验证程序启动失败");
}
#[cfg(not(windows))] fn main(){eprintln!("原生宿主仅在 Windows 运行；Mac 请使用明确标记的网页预览和核心测试。");}

#[cfg(test)] mod lifecycle_tests {
    #[test] fn early_webview_snapshot_before_setup_does_not_panic(){
        let app=tauri::test::mock_builder().build(tauri::test::mock_context(tauri::test::noop_assets())).unwrap();
        assert!(super::state_snapshot(app.handle()).is_none());
    }
}
