use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaWindow {
    pub minutes: Option<u64>,
    pub remaining: Option<f64>,
    pub resets_at: Option<f64>,
}

fn number(v: &Value) -> Option<f64> { v.as_f64().filter(|n| n.is_finite()) }

/// 与 Swift 一致：存在有效的分桶表时，不把其他产品的额度冒充 Codex。
pub fn normalize(response: &Value) -> Vec<QuotaWindow> {
    let map = response.get("rateLimitsByLimitId").and_then(Value::as_object);
    let bucket = if map.is_some_and(|m| m.values().any(Value::is_object)) {
        map.and_then(|m| m.get("codex"))
    } else { response.get("rateLimits") };
    let Some(bucket) = bucket.filter(|b| b.is_object()) else { return vec![] };
    let mut windows: Vec<_> = ["primary", "secondary"].iter().filter_map(|key| {
        let w = bucket.get(key)?.as_object()?;
        let minutes = w.get("windowDurationMins").and_then(number)
            .filter(|n| *n > 0.0 && *n < i64::MAX as f64 && n.fract() == 0.0).map(|n| n as u64);
        Some(QuotaWindow { minutes,
            remaining: w.get("usedPercent").and_then(number).map(|n| (100.0-n).clamp(0.0,100.0)),
            resets_at: w.get("resetsAt").and_then(number) })
    }).collect();
    windows.sort_by_key(|w| match w.minutes { Some(300) => 0, Some(10080) => 1, _ => 2 });
    windows
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test] fn normalization_matches_swift() {
        let bucket=json!({"primary":{"windowDurationMins":10080,"usedPercent":53,"resetsAt":9000},"secondary":{"windowDurationMins":300,"usedPercent":18,"resetsAt":2000}});
        let w=normalize(&json!({"rateLimitsByLimitId":{"codex":bucket,"code_review":{} }}));
        assert_eq!(w.iter().map(|w|w.remaining).collect::<Vec<_>>(),vec![Some(82.0),Some(47.0)]);
        assert_eq!(w[0].resets_at,Some(2000.0));
        assert!(normalize(&json!({"rateLimitsByLimitId":{"code_review":bucket},"rateLimits":bucket})).is_empty());
        assert_eq!(normalize(&json!({"rateLimitsByLimitId":{},"rateLimits":bucket})).len(),2);
    }
    #[test] fn unknown_is_not_zero() {
        for (raw, expected) in [(Value::Null,None),(json!(true),None),(json!("18"),None),(json!(120),Some(0.0)),(json!(-10),Some(100.0)),(json!(100),Some(0.0))] {
            let w=normalize(&json!({"rateLimits":{"primary":{"usedPercent":raw,"windowDurationMins":1.5}}}));
            assert_eq!(w[0].remaining,expected); assert_eq!(w[0].minutes,None);
        }
        assert!(normalize(&json!({})).is_empty());
    }
    #[test] fn buckets_and_unknown_durations_match_swift(){
        let bucket=json!({"primary":{"windowDurationMins":60,"usedPercent":25},"secondary":{"windowDurationMins":120,"usedPercent":50}});
        assert_eq!(normalize(&json!({"rateLimitsByLimitId":{"ignored":false},"rateLimits":bucket})).len(),2);
        assert!(normalize(&json!({"rateLimitsByLimitId":{"codex":null,"code_review":{}},"rateLimits":bucket})).is_empty());
        let windows=normalize(&json!({"rateLimits":bucket}));
        assert_eq!(windows.iter().map(|w|w.minutes).collect::<Vec<_>>(),vec![Some(60),Some(120)]);
        for duration in [json!(true),json!(0),json!(-1),json!(1.5),json!("300"),json!(9223372036854775807u64)] {
            let w=normalize(&json!({"rateLimits":{"primary":{"windowDurationMins":duration,"usedPercent":18}}}));
            assert_eq!(w[0].minutes,None);assert_eq!(w[0].remaining,Some(82.0));
        }
    }
}
