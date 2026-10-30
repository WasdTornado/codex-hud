#[path="../src/startup.rs"] mod startup;
use tauri::Manager;

// 使用与宿主相同的状态读取；集成测试继承测试目标的 Common Controls manifest。
#[test] fn early_snapshot_is_empty_then_reads_registered_state(){
    let app=tauri::test::mock_builder().build(tauri::test::mock_context(tauri::test::noop_assets())).unwrap();
    assert!(startup::snapshot::<_,String,_>(app.handle(),Clone::clone).is_none());
    app.manage(String::from("ready"));
    assert_eq!(startup::snapshot::<_,String,_>(app.handle(),Clone::clone).as_deref(),Some("ready"));
}
