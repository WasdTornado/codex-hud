use crate::model::QuotaWindow;
use serde::{Deserialize, Serialize};

pub const REFRESH_SECONDS: u64 = 300;
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Snapshot {
    pub windows: Vec<QuotaWindow>,
    pub updated_at: Option<u64>,
    pub loading: bool,
    pub error: Option<String>,
}
impl Snapshot {
    pub fn begin(&mut self) -> bool {
        if self.loading { return false; }
        self.loading=true; true
    }
    pub fn finish(&mut self, result: Result<Vec<QuotaWindow>, String>, now_ms: u64) {
        match result {
            Ok(windows) => { self.windows=windows; self.updated_at=Some(now_ms); self.error=None; }
            Err(error) => self.error=Some(error),
        }
        self.loading=false;
    }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn deduplication_and_failed_refresh_preserve_success() {
        let mut s=Snapshot::default(); assert!(s.begin()); assert!(!s.begin());
        s.finish(Ok(vec![QuotaWindow {minutes:Some(300),remaining:Some(0.0),resets_at:None}]),1000);
        assert!(s.begin()); s.finish(Err("超时".into()),2000);
        assert_eq!(s.windows[0].remaining,Some(0.0)); assert_eq!(s.updated_at,Some(1000));
        assert!(s.error.is_some()); assert!(!s.loading);
        s.finish(Ok(vec![]),3000); assert!(s.error.is_none()); assert_eq!(s.updated_at,Some(3000));
    }
}
