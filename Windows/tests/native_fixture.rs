#![cfg(feature="test-fixtures")]
#[path="support/harness.rs"] mod harness;
use codex_hud_core::client::{self,Launch};
use std::time::Duration;
use tokio::sync::watch;

// 验证与 Windows 测试共用的 Rust 假服务；Mac 上通过只算本机协议模拟。
#[tokio::test] async fn native_fixture_protocol_and_boundaries(){
    for (mode,expected) in [("success",None),("error",Some("登录")),("eof",Some("登录")),("oversize",Some("过长")),("total",Some("过大")),("timeout",Some("超时"))]{
        let fixture=harness::Fixture::new(mode,false,false);let (_tx,rx)=watch::channel(false);
        let launch=Launch{executable:env!("CARGO_BIN_EXE_hud-fake-cli").into(),arguments:vec!["app-server".into(),"--listen".into(),"stdio://".into()]};
        let result=client::read(&launch,&fixture.root,Duration::from_secs(if cfg!(windows){8}else{1}),rx).await;
        if let Some(expected)=expected{let error=result.unwrap_err();assert!(error.contains(expected),"{mode}: {error}");assert!(!error.contains("secret-token"));}
        else{assert_eq!(result.unwrap().iter().map(|w|w.remaining).collect::<Vec<_>>(),vec![Some(82.0),Some(47.0)]);}
    }
}
