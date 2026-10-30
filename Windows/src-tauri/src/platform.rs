use tauri::{AppHandle,Manager,WebviewWindow};

pub fn shape(window:&WebviewWindow)->Result<(),String>{
    #[cfg(not(windows))] {let _=window;return Err("原生窗口区域仅能在 Windows 验证".into());}
    #[cfg(windows)] unsafe {
        use windows_sys::Win32::Graphics::Gdi::{CreateRoundRectRgn,DeleteObject,SetWindowRgn};
        // 先取得 HWND，避免后续错误返回泄漏尚未交给系统的区域。
        let hwnd=window.hwnd().map_err(|_|"无法读取 HWND")?.0 as _;
        let scale=window.scale_factor().map_err(|_|"无法读取 DPI")?;
        let size=window.outer_size().map_err(|_|"无法读取窗口尺寸")?;
        let diameter=36.0;
        let region=CreateRoundRectRgn(0,0,size.width as i32+1,size.height as i32+1,(diameter*scale).round()as i32,(diameter*scale).round()as i32);
        if region.is_null(){return Err("无法创建窗口形状".into());}
        if SetWindowRgn(hwnd,region,1)==0{DeleteObject(region);return Err("无法应用窗口形状".into());}
        // 成功后系统拥有 HRGN，不能自行释放。
    }
    #[cfg(windows)] {Ok(())}
}
pub fn appearance(window:&WebviewWindow,dark:bool)->Result<(),String>{
    if dark {window.set_effects(None).map_err(|_|"无法移除浅色材质")?;}
    else {
        window.set_effects(tauri::window::EffectsBuilder::new().effect(tauri::window::Effect::Acrylic).color(tauri::webview::Color(242,247,244,190)).build()).map_err(|_|"无法启用浅色磨砂；该平台能力尚未通过")?;
    }
    shape(window)
}
pub fn drag_threshold(window:&WebviewWindow)->f64{
    #[cfg(windows)] unsafe {return (windows_sys::Win32::UI::WindowsAndMessaging::GetSystemMetrics(windows_sys::Win32::UI::WindowsAndMessaging::SM_CXDRAG) as f64/window.scale_factor().unwrap_or(1.0)).max(2.0);}
    #[cfg(not(windows))] {let _=window;4.0}
}
pub fn tray_dark()->bool{
    #[cfg(windows)] unsafe {
        use windows_sys::Win32::System::Registry::*;
        let key:Vec<u16>="Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0".encode_utf16().collect();
        let name:Vec<u16>="SystemUsesLightTheme\0".encode_utf16().collect();
        let mut light:u32=1;let mut size=4;
        RegGetValueW(HKEY_CURRENT_USER,key.as_ptr(),name.as_ptr(),RRF_RT_REG_DWORD,std::ptr::null_mut(),&mut light as *mut _ as _,&mut size);
        return light==0;
    }
    #[cfg(not(windows))] {false}
}

/// 真实 Windows 唤醒消息；不使用仅移动平台发出的 tauri://resumed。
pub fn install_wake(window:&WebviewWindow,app:AppHandle)->Result<(),String>{
    #[cfg(windows)] unsafe {
        use windows_sys::Win32::UI::Shell::SetWindowSubclass;
        let hwnd=window.hwnd().map_err(|_|"无法读取 HWND")?.0 as _;
        let context=Box::into_raw(Box::new(app));
        if SetWindowSubclass(hwnd,Some(wake_proc),1,context as usize)==0{
            drop(Box::from_raw(context));return Err("无法注册 Windows 唤醒通知".into());
        }
    }
    #[cfg(not(windows))] {let _=(window,app);return Err("原生唤醒通知仅能在 Windows 验证".into());}
    #[cfg(windows)] {Ok(())}
}
#[cfg(windows)] unsafe extern "system" fn wake_proc(hwnd:windows_sys::Win32::Foundation::HWND,message:u32,wparam:usize,lparam:isize,id:usize,context:usize)->isize {
    use windows_sys::Win32::UI::{Shell::{DefSubclassProc,RemoveWindowSubclass},WindowsAndMessaging::{WM_POWERBROADCAST,WM_NCDESTROY,WM_SETTINGCHANGE,WM_DISPLAYCHANGE}};
    let app=&*(context as *const AppHandle);
    if message==WM_POWERBROADCAST&&wparam==0x12{crate::refresh_background(app.clone());}
    if message==WM_SETTINGCHANGE||message==WM_DISPLAYCHANGE{
        let app=app.clone();let _=app.clone().run_on_main_thread(move||{
            // 显示器拔除、工作区变化只校正位置，不改变用户的隐藏意图。
            if let Err(error)=clamp_and_dock(&app,None).and_then(|_|crate::position_details(&app)){crate::report(&app,error);}
            crate::publish(&app);
        });
    }
    if message==WM_NCDESTROY{RemoveWindowSubclass(hwnd,Some(wake_proc),id);drop(Box::from_raw(context as *mut AppHandle));}
    DefSubclassProc(hwnd,message,wparam,lparam)
}

pub fn clamp_and_dock(app:&AppHandle,right:Option<bool>)->Result<(),String>{
    let window=app.get_webview_window("orb").ok_or("找不到浮窗")?;
    let monitor=window.current_monitor().map_err(|_|"无法读取显示器")?.or(window.primary_monitor().map_err(|_|"无法读取显示器")?).ok_or("没有显示器")?;
    let area=monitor.work_area();let size=window.outer_size().map_err(|_|"无法读取窗口尺寸")?;
    let mut position=window.outer_position().map_err(|_|"无法读取窗口位置")?;
    let original=position;
    let max_x=area.position.x+(area.size.width.saturating_sub(size.width))as i32;
    let max_y=area.position.y+(area.size.height.saturating_sub(size.height))as i32;
    position.x=match right{Some(true)=>max_x,Some(false)=>area.position.x,None=>position.x.clamp(area.position.x,max_x)};
    position.y=position.y.clamp(area.position.y,max_y);
    if position==original{return Ok(());}
    window.set_position(position).map_err(|_|"无法移动浮窗".into())
}
