#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! AIDD 对抗 / 边界用例（特性 005）。
//!
//! 候选由 AI 生成，逐条人工复核后仅保留「结论=保留」项；丢弃项登记于 PR 描述。
//!
//! // AIDD: 周频序列 ICSA 标成 daily | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §3 频率一致性 | 结论=保留
//! // AIDD: 合法零值 0.0 与缺失混淆 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §2 缺失不得折算为 0 | 结论=保留
//! // AIDD: 闰日 2024-02-29 与 2025-02-29 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §2 日按月份与闰年校验 | 结论=保留
//! // AIDD: 五位年份 10000-01-01 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §2 严格 ISO 形态 | 结论=保留
//! // AIDD: 未决候选 RESPPLLOPNWW 被当作采集序列 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §1 范围外 ID 拒绝 | 结论=保留
//! // AIDD: 非 ASCII 与超长 series_id | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §2 标识须非空无首尾空白 | 结论=保留
//! // AIDD: 布尔型观测值 true | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §8 值形态白名单 | 结论=保留
//! // AIDD: 曲线输入点同时是采集序列 T10Y2Y | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §5 曲线点可作输入 | 结论=保留
//! // AIDD: 负零与极大有限值 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §2 有限数判定 | 结论=保留
//! // AIDD: 空 records 数组 | 来源=AI | 复核=ZoneCNH/2026-09-22 | 依据=标准.md §8 解析器边界 | 结论=保留

use fredx::{
    claim_authoritative_write, ensure_known_series, ensure_source_frequency, is_collected_series,
    is_curve_input_series, parse_fred_observations, validate_date, validate_observation, Date,
    FredErrorKind, FredMissingReason, FredObservation, FredUnit, FredValue, Frequency, Period,
    UNIT_MILLIONS_OF_USD,
};

fn record(series_id: &str, value: &str, frequency: &str) -> String {
    format!(
        r#"{{"records": [{{"series_id": "{series_id}", "date": "2026-08-14", "value": {value},
            "unit": "Percent", "frequency": "{frequency}"}}]}}"#
    )
}

/// 边界：清单声明为周频的 `ICSA` 被标成 `daily`，必须被频率守卫拒绝。
#[test]
fn frequency_drift_on_weekly_series_is_rejected() {
    let drift = r#"{"records": [
        {"series_id": "ICSA", "date": "2026-08-08", "value": 221000.0, "unit": "Persons",
         "frequency": "daily"}
    ]}"#;
    let err = parse_fred_observations(drift).expect_err("应拒绝频率漂移");
    assert_eq!(err.kind(), FredErrorKind::SemanticallyRejected);

    let observation = FredObservation {
        series_id: "ICSA".to_owned(),
        period: Period::Day(Date::new(2026, 8, 8).expect("日期合法")),
        value: FredValue::Present(221000.0),
        unit: FredUnit::new("Persons").expect("单位合法"),
        frequency: Frequency::Daily,
        vintage: None,
    };
    assert!(ensure_source_frequency(&observation).is_err());
}

/// 边界：合法零值与「缺失」必须可区分；缺失不得折算为 0。
#[test]
fn zero_is_a_reported_value_not_a_missing_marker() {
    let observations = parse_fred_observations(&record("SOFR", "0.0", "daily")).expect("零值合法");
    assert_eq!(observations[0].value.as_f64(), Some(0.0));
    assert!(!observations[0].value.is_missing());
    assert_eq!(
        observations[0].value,
        FredValue::Present(0.0),
        "零值必须保留为已发布数值"
    );

    let missing = parse_fred_observations(&record("SOFR", r#"".""#, "daily")).expect("缺失合法");
    assert!(missing[0].value.is_missing());
    assert_eq!(missing[0].value.as_f64(), None);
    assert_ne!(
        missing[0].value,
        FredValue::Present(0.0),
        "缺失不得被折算为 0"
    );
    assert_eq!(
        missing[0].value,
        FredValue::Missing(FredMissingReason::NoObservation)
    );
}

/// 边界：闰日只在闰年合法。
#[test]
fn leap_day_is_only_valid_in_leap_years() {
    assert!(validate_date(&Date::new(2024, 2, 29).expect("2024 是闰年")).is_ok());
    assert!(Date::new(2025, 2, 29).is_err());
    assert!(Date::new(1900, 2, 29).is_err());
    assert!(Date::new(2000, 2, 29).is_ok());
}

/// 边界：严格四位年份，拒绝五位年份与非 ISO 分隔。
#[test]
fn date_year_width_is_fixed_at_four_digits() {
    assert!(Date::parse("9999-12-31").is_ok());
    assert!(Date::parse("10000-01-01").is_err());
    assert!(Date::parse("0001-01-01").is_ok());
    assert!(Date::parse("2026.08.14").is_err());
}

/// 边界：归属未决的候选 `RESPPLLOPNWW` 不得被当成采集序列，也不得主张权威写入。
#[test]
fn pending_candidate_is_not_in_scope_and_cannot_claim_authority() {
    assert!(!is_collected_series("RESPPLLOPNWW"));
    assert!(ensure_known_series("RESPPLLOPNWW").is_err());
    assert_eq!(
        claim_authoritative_write("RESPPLLOPNWW")
            .expect_err("未决")
            .kind(),
        FredErrorKind::WriteAuthorityDenied
    );
}

/// 边界：非 ASCII 与超长 `series_id` 不 panic，且一律拒绝。
#[test]
fn non_ascii_and_overlong_series_ids_are_rejected_without_panic() {
    for candidate in ["未知序列", "WALCL\u{200b}", &"A".repeat(4096)] {
        assert!(ensure_known_series(candidate).is_err(), "{candidate}");
    }
    let observation = FredObservation {
        series_id: "WALCL ".to_owned(),
        period: Period::Day(Date::new(2026, 8, 14).expect("日期合法")),
        value: FredValue::Present(1.0),
        unit: FredUnit::new(UNIT_MILLIONS_OF_USD).expect("单位合法"),
        frequency: Frequency::Weekly,
        vintage: None,
    };
    assert!(validate_observation(&observation).is_err());
}

/// 边界：布尔型观测值不在值形态白名单内，必须原子失败。
#[test]
fn boolean_value_form_is_rejected() {
    let err = parse_fred_observations(&record("SOFR", "true", "daily")).expect_err("应拒绝");
    assert_eq!(err.kind(), FredErrorKind::Invalid);
}

/// 边界：`T10Y2Y` 同时是采集序列与曲线输入点 —— 两个判定都不得互相压制。
#[test]
fn curve_input_series_may_also_be_a_collected_series() {
    assert!(is_collected_series("T10Y2Y"));
    assert!(is_curve_input_series("T10Y2Y"));
    // 采集允许，但曲线构建仍归 yieldx。
    assert!(parse_fred_observations(&record("T10Y2Y", "0.42", "daily")).is_ok());
    assert!(fredx::ensure_product_local(fredx::FredProduct::YieldCurve).is_err());
}

/// 边界：负零与极大有限值属已发布数值；非有限值一律拒绝。
#[test]
fn finite_extremes_are_accepted_and_non_finite_rejected() {
    assert!(parse_fred_observations(&record("SOFR", "-0.0", "daily")).is_ok());
    assert!(parse_fred_observations(&record("SOFR", "1.7976931348623157e308", "daily")).is_ok());
    assert!(parse_fred_observations(&record("SOFR", "1e400", "daily")).is_err());
}

/// 边界：空 `records` 是合法输入（空集合），不是错误。
#[test]
fn empty_records_is_a_valid_input() {
    let observations = parse_fred_observations(r#"{"records": []}"#).expect("空集合合法");
    assert!(observations.is_empty());
}
