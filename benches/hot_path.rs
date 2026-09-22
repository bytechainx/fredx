#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! fredx 热路径：离线解析 + 跨源守卫 + 授权判定。
//!
//! 本基准是**自计时的手写基准**（`harness = false`，自带 `fn main`），
//! 不是可复现的性能证据：它不构成任何 SLA 或能力宣称。

use std::hint::black_box;
use std::time::Instant;

use fredx::{
    authorize_fred, claim_authoritative_write, documented_fred_evidence, ensure_baml_freeze,
    ensure_not_silent_substitution, parse_fred_observations, Date, FredAccessMode,
    FredAuthorization, WALCL,
};

/// 迭代次数（固定值，不读环境变量）。
const ITERS: u32 = 20_000;

/// 热路径输入：两条记录的合成样本（非真实源数据）。
const INPUT: &str = r#"{"records": [
    {"series_id": "WALCL", "date": "2026-08-13", "value": 7000000.0,
     "unit": "Millions of U.S. Dollars", "frequency": "weekly"},
    {"series_id": "RRPONTSYD", "date": "2026-08-14", "value": 450.25,
     "unit": "Billions of U.S. Dollars", "frequency": "daily"}
]}"#;

fn main() {
    let as_of = Date::new(2026, 8, 20).expect("日期合法");
    let evidence = documented_fred_evidence();
    let start = Instant::now();
    let mut acc = 0u64;
    for _ in 0..ITERS {
        let observations = parse_fred_observations(INPUT).expect("解析");
        acc = acc.wrapping_add(observations.len() as u64);
        if matches!(
            authorize_fred(Some(&evidence), FredAccessMode::Offline, as_of),
            FredAuthorization::Authorized { .. }
        ) {
            acc = acc.wrapping_add(1);
        }
        if claim_authoritative_write(WALCL).is_ok() && ensure_baml_freeze(WALCL).is_ok() {
            acc = acc.wrapping_add(1);
        }
        if ensure_not_silent_substitution(WALCL, "RRPONTSYD").is_ok() {
            acc = acc.wrapping_add(1);
        }
        black_box(&observations);
    }
    let elapsed = start.elapsed();
    println!(
        "bench_fredx_hot_path: iters={ITERS} total={elapsed:?} per_iter={:?} acc={}",
        elapsed / ITERS,
        black_box(acc)
    );
}
