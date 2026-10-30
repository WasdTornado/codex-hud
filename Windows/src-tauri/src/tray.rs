use codex_hud_core::state::Snapshot;
use tauri::image::Image;

// 托盘标题在 Windows 不显示。用透明 RGBA 画环与数字，避免依赖字体或 WebView。
pub fn image(quota:&Snapshot,dark:bool,notice:bool)->Image<'static>{
    let selected=quota.windows.iter().find(|w|w.minutes==Some(300)).or(quota.windows.first());
    let value=selected.and_then(|w|w.remaining);
    let digits=value.map(|v|format!("{:.0}",v.clamp(0.0,100.0).round())).unwrap_or_else(||"-".into());
    let fg=if dark{[235,240,238]}else{[35,44,48]};
    let mut data=vec![0u8;32*32*4];
    let mut pixel=|x:usize,y:usize,alpha:u8|{let i=(y*32+x)*4;data[i..i+3].copy_from_slice(&fg);data[i+3]=alpha;};
    for y in 0..32 {for x in 0..32 {
        let dx=x as f64+0.5-16.0;let dy=y as f64+0.5-16.0;let radius=(dx*dx+dy*dy).sqrt();
        if (radius-14.0).abs()<1.1 {let angle=(dy.atan2(dx)+std::f64::consts::FRAC_PI_2).rem_euclid(std::f64::consts::TAU);
            let progress=value.is_some_and(|v|angle/std::f64::consts::TAU<v/100.0);pixel(x,y,if progress{255}else{65});}
    }}
    const FONT:[[u8;5];11]=[[7,5,5,5,7],[2,6,2,2,7],[7,1,7,4,7],[7,1,7,1,7],[5,5,7,1,1],[7,4,7,1,7],[7,4,7,5,7],[7,1,1,1,1],[7,5,7,5,7],[7,5,7,1,7],[0,0,7,0,0]];
    let scale=2;let width=digits.len()*8-2;let start=(32-width)/2;
    for (i,ch) in digits.bytes().enumerate(){let glyph=if ch.is_ascii_digit(){(ch-b'0')as usize}else{10};for row in 0..5{for col in 0..3{if FONT[glyph][row]&(1<<(2-col))!=0{for dy in 0..scale{for dx in 0..scale{pixel(start+i*8+col*scale+dx,11+row*scale+dy,255);}}}}}}
    if notice||quota.error.is_some(){for y in 1..6{for x in 25..30{pixel(x,y,255);}}}
    Image::new_owned(data,32,32)
}

#[cfg(test)] mod tests{
    use super::*;
    use codex_hud_core::model::QuotaWindow;
    fn quota(value:Option<f64>)->Snapshot{Snapshot{windows:vec![QuotaWindow{minutes:Some(300),remaining:value,resets_at:None}],..Default::default()}}
    #[test] fn digits_round_like_the_orb(){assert_eq!(image(&quota(Some(82.5)),false,false).rgba()[11*32*4..21*32*4],image(&quota(Some(83.0)),false,false).rgba()[11*32*4..21*32*4]);}
    #[test] fn five_hour_window_wins_even_when_input_is_reversed(){
        let only=quota(Some(82.0));let mut dual=only.clone();dual.windows.insert(0,QuotaWindow{minutes:Some(10080),remaining:Some(47.0),resets_at:None});
        assert_eq!(image(&dual,false,false).rgba(),image(&only,false,false).rgba());
    }
    #[test] fn transparent_corners_and_distinct_unknown_zero_and_full(){
        let unknown=image(&quota(None),false,false);let zero=image(&quota(Some(0.0)),false,false);let full=image(&quota(Some(100.0)),false,false);
        assert_eq!(full.rgba().len(),32*32*4);assert_eq!(full.rgba()[3],0);
        assert_ne!(unknown.rgba(),zero.rgba());assert_ne!(zero.rgba(),full.rgba());
    }
}
