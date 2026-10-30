// 使用 Tauri 官方公开测试向量；只有公钥和固定 payload 的签名，没有私钥。
// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::{json,Value};
use tauri_plugin_updater::UpdaterExt;

fn fixture()->Value{serde_json::from_str(include_str!("fixtures/update.json")).unwrap()}
fn mock_app(config:Value)->tauri::App<tauri::test::MockRuntime>{
    let mut context=tauri::test::mock_context(tauri::test::noop_assets());
    context.config_mut().plugins.0.insert("updater".into(),config);
    tauri::test::mock_builder().plugin(tauri_plugin_updater::Builder::new().build()).build(context).unwrap()
}
#[test] fn disabled_update_configuration_does_not_block_startup(){
    let config:Value=serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
    assert_eq!(config["plugins"]["updater"]["endpoints"],json!([]));
    assert_eq!(config["plugins"]["updater"]["pubkey"],"");
    let _app=mock_app(config["plugins"]["updater"].clone());
}
fn verified_update(version:&str)->tauri_plugin_updater::Update{
    let fixture=fixture();
    let app=mock_app(json!({"pubkey":fixture["pubkey"],"endpoints":[],"requireSignedVersion":true}));
    app.updater_builder().target("test-target").executable_path("unused-fixture-path/app")
        .configure_client(|_|panic!("离线签名测试不允许网络请求"))
        .endpoints(vec!["https://example.invalid/unused".parse().unwrap()]).unwrap()
        .build().unwrap().restore_update(json!({"version":version,"url":"https://example.invalid/unused","signature":fixture["signature"]})).unwrap().unwrap()
}
#[test] fn updater_accepts_valid_signature_and_rejects_tampered_bytes(){
    let update=verified_update("2.0.0");
    update.verify(b"test").unwrap();
    assert!(update.verify(b"tampered").is_err());
}
#[test] fn updater_rejects_manifest_version_not_bound_by_signature(){
    assert!(verified_update("3.0.0").verify(b"test").is_err());
}
