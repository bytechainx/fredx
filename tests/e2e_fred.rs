#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! E2E（fredx）：在**真实字符串输入 + 真实类型构造 + 真实守卫判定**上端到端执行
//! **全部**公开接口。
//!
//! fredx 是纯离线库：无外部服务、无凭据、无网络、无进程级环境变量。它的 E2E 面就是：
//! 真实的离线 JSON 输入（合成夹具，带 `_synthetic` 标注）、真实的类型构造与穷尽解构、
//! 以及真实执行的 fail-closed 守卫（清单范围 / BAML 冻结令 / 单位与频率一致性 /
//! 近义非同 ID / 写入主权 / 曲线边界 / 授权判定）。
//!
//! 与分点单测不同，本文件的对齐对象是 `cargo +nightly public-api --simplified` 导出的
//! 完整公开面（171 条）：`fn` / `type` / `field` / `const` / `variant` 五类逐条登记在
//! [`E2E_MANIFEST`]，运行期由 `cover` 登记表核对「声明 = 实际执行」（缺一即失败）。
//!
//! **独立核对**：`scripts/verify-e2e-coverage.mjs` 会重新派生公开面与清单双向 diff，并用
//! `-C instrument-coverage` + `llvm-cov report` 断言每条公开 `fn` 执行次数 > 0；本文件内的
//! 登记表只是**声明**，不是唯一证据。
//!
//! ```text
//! cd /home/workspace/bytechainx/.worktrees/fredx/fredx
//! CARGO_TARGET_DIR=/home/workspace/bytechainx/.cargo/wt/fredx cargo test --test e2e_fred
//! ```

use std::collections::BTreeSet;
use std::hint::black_box;

use fredx::{
    authorize_fred, claim_authoritative_write, documented_fred_evidence, ensure_baml_freeze,
    ensure_known_series, ensure_not_silent_substitution, ensure_product_local,
    ensure_source_frequency, ensure_source_unit, fred_publication_semantics, is_bea_table_id,
    is_collected_series, is_curve_input_series, is_formal_pit_eligible, is_known_series,
    is_mapping_only_series, is_near_synonym_pair, is_unique_write_authority, mode_label,
    parse_fred_observations, series_frequency, series_unit, validate_authorization_evidence,
    validate_date, validate_observation, validate_period, AvailabilityEvidence, Date,
    FredAccessMode, FredAuthorization, FredAuthorizationEvidence, FredError, FredErrorKind,
    FredMissingReason, FredObservation, FredProduct, FredPublicationSemantics, FredResult,
    FredUnit, FredValue, Frequency, Period, PitEligibility, TimePrecision, ALL_SERIES,
    AUTHORITATIVE_WRITE_SERIES, BAMLC0A0CM, BAMLH0A0HYM2, BAML_ALLOWED_SERIES, BEA_TABLE_OWNER,
    BLS_FORWARD_SERIES, BOGZ1FL662090005Q, CES0500000003, CPHPTT01EZM659N, CPILFESL,
    CURVE_BUILD_OWNER, CURVE_SERIES, DCOILWTICO, DEXCHUS, DEXJPUS, DEXUSEU, DFF, DFII10, DGS10,
    DGS2, FDHBFRBN, FRED_DECISION_ID, FRED_SIGNED_BY, FUNDAMENTAL_SERIES, FYFSGDA188S,
    GLOBAL_REGIONAL_SERIES, ICSA, IRLTLT01JPM156N, JPNCPIALLMINMEI, JTSJOL, NASDAQCOM,
    NEAR_SYNONYM_PAIRS, NIKKEI225, P0_SERIES, PAYEMS, PCEPILFE, PCOPPUSDM, RESPPLLOPNWW, RRPONTSYD,
    SOFR, SP500, T10Y2Y, T10YIE, T5YIFR, TEDRATE, UNIT_BILLIONS_OF_USD, UNIT_MILLIONS_OF_USD,
    UNRATE, VIXCLS, WALCL, WRESBAL, WTREGEN,
};

/// 公开面清单：`(条目类别, 入口 id)`，由权威公开面派生并冻结。
///
/// 类别取值域：`fn` / `type` / `field` / `const` / `variant`。
/// 该清单由 `scripts/verify-e2e-coverage.mjs --write-manifest` 生成，MUST NOT 手抄。
const E2E_MANIFEST: &[(&str, &str)] = &[
    ("type", "FredAccessMode"),
    ("variant", "FredAccessMode::Live"),
    ("variant", "FredAccessMode::Offline"),
    ("type", "FredAuthorization"),
    ("variant", "FredAuthorization::Authorized"),
    ("variant", "FredAuthorization::Denied"),
    ("type", "FredAuthorizationEvidence"),
    ("field", "FredAuthorizationEvidence::authorized_modes"),
    ("field", "FredAuthorizationEvidence::decision_id"),
    ("field", "FredAuthorizationEvidence::scope_note"),
    ("field", "FredAuthorizationEvidence::signed_at"),
    ("field", "FredAuthorizationEvidence::signed_by"),
    ("field", "FredAuthorizationEvidence::valid_until"),
    ("const", "FRED_DECISION_ID"),
    ("const", "FRED_SIGNED_BY"),
    ("fn", "authorize_fred"),
    ("fn", "documented_fred_evidence"),
    ("fn", "mode_label"),
    ("fn", "validate_authorization_evidence"),
    ("type", "FredError"),
    ("variant", "FredError::AuthorizationDenied"),
    ("variant", "FredError::Invalid"),
    ("variant", "FredError::Invariant"),
    ("variant", "FredError::Missing"),
    ("variant", "FredError::NotApplicable"),
    ("variant", "FredError::RoutedElsewhere"),
    ("variant", "FredError::SemanticallyRejected"),
    ("variant", "FredError::WriteAuthorityDenied"),
    ("fn", "FredError::is_retryable"),
    ("fn", "FredError::kind"),
    ("type", "FredErrorKind"),
    ("variant", "FredErrorKind::AuthorizationDenied"),
    ("variant", "FredErrorKind::Invalid"),
    ("variant", "FredErrorKind::Invariant"),
    ("variant", "FredErrorKind::Missing"),
    ("variant", "FredErrorKind::NotApplicable"),
    ("variant", "FredErrorKind::RoutedElsewhere"),
    ("variant", "FredErrorKind::SemanticallyRejected"),
    ("variant", "FredErrorKind::WriteAuthorityDenied"),
    ("type", "FredResult"),
    ("fn", "parse_fred_observations"),
    ("type", "AvailabilityEvidence"),
    ("variant", "AvailabilityEvidence::Calendar"),
    ("variant", "AvailabilityEvidence::Inferred"),
    ("variant", "AvailabilityEvidence::Official"),
    ("type", "PitEligibility"),
    ("variant", "PitEligibility::Formal"),
    ("variant", "PitEligibility::NotEligible"),
    ("type", "TimePrecision"),
    ("variant", "TimePrecision::Date"),
    ("variant", "TimePrecision::Instant"),
    ("type", "FredPublicationSemantics"),
    ("field", "FredPublicationSemantics::availability"),
    ("field", "FredPublicationSemantics::eligibility"),
    ("field", "FredPublicationSemantics::time_precision"),
    ("fn", "fred_publication_semantics"),
    ("fn", "is_formal_pit_eligible"),
    ("type", "FredProduct"),
    ("variant", "FredProduct::Observation"),
    ("variant", "FredProduct::YieldCurve"),
    ("const", "BEA_TABLE_OWNER"),
    ("const", "CURVE_BUILD_OWNER"),
    ("const", "NEAR_SYNONYM_PAIRS"),
    ("fn", "claim_authoritative_write"),
    ("fn", "ensure_baml_freeze"),
    ("fn", "ensure_not_silent_substitution"),
    ("fn", "ensure_product_local"),
    ("fn", "is_bea_table_id"),
    ("fn", "is_near_synonym_pair"),
    ("const", "ALL_SERIES"),
    ("const", "AUTHORITATIVE_WRITE_SERIES"),
    ("const", "BAMLC0A0CM"),
    ("const", "BAMLH0A0HYM2"),
    ("const", "BAML_ALLOWED_SERIES"),
    ("const", "BLS_FORWARD_SERIES"),
    ("const", "BOGZ1FL662090005Q"),
    ("const", "CES0500000003"),
    ("const", "CPHPTT01EZM659N"),
    ("const", "CPILFESL"),
    ("const", "CURVE_SERIES"),
    ("const", "DCOILWTICO"),
    ("const", "DEXCHUS"),
    ("const", "DEXJPUS"),
    ("const", "DEXUSEU"),
    ("const", "DFF"),
    ("const", "DFII10"),
    ("const", "DGS10"),
    ("const", "DGS2"),
    ("const", "FDHBFRBN"),
    ("const", "FUNDAMENTAL_SERIES"),
    ("const", "FYFSGDA188S"),
    ("const", "GLOBAL_REGIONAL_SERIES"),
    ("const", "ICSA"),
    ("const", "IRLTLT01JPM156N"),
    ("const", "JPNCPIALLMINMEI"),
    ("const", "JTSJOL"),
    ("const", "NASDAQCOM"),
    ("const", "NIKKEI225"),
    ("const", "P0_SERIES"),
    ("const", "PAYEMS"),
    ("const", "PCEPILFE"),
    ("const", "PCOPPUSDM"),
    ("const", "RESPPLLOPNWW"),
    ("const", "RRPONTSYD"),
    ("const", "SOFR"),
    ("const", "SP500"),
    ("const", "T10Y2Y"),
    ("const", "T10YIE"),
    ("const", "T5YIFR"),
    ("const", "TEDRATE"),
    ("const", "UNRATE"),
    ("const", "VIXCLS"),
    ("const", "WALCL"),
    ("const", "WRESBAL"),
    ("const", "WTREGEN"),
    ("fn", "ensure_known_series"),
    ("fn", "ensure_source_frequency"),
    ("fn", "ensure_source_unit"),
    ("fn", "is_collected_series"),
    ("fn", "is_curve_input_series"),
    ("fn", "is_known_series"),
    ("fn", "is_mapping_only_series"),
    ("fn", "is_unique_write_authority"),
    ("fn", "series_frequency"),
    ("fn", "series_unit"),
    ("type", "FredMissingReason"),
    ("variant", "FredMissingReason::NoObservation"),
    ("type", "FredValue"),
    ("variant", "FredValue::Missing"),
    ("variant", "FredValue::Present"),
    ("fn", "FredValue::as_f64"),
    ("fn", "FredValue::is_missing"),
    ("type", "Frequency"),
    ("variant", "Frequency::Annual"),
    ("variant", "Frequency::Daily"),
    ("variant", "Frequency::Event"),
    ("variant", "Frequency::Irregular"),
    ("variant", "Frequency::Monthly"),
    ("variant", "Frequency::Quarterly"),
    ("variant", "Frequency::Weekly"),
    ("fn", "Frequency::as_str"),
    ("fn", "Frequency::parse"),
    ("type", "Period"),
    ("variant", "Period::Day"),
    ("variant", "Period::Event"),
    ("variant", "Period::Month"),
    ("variant", "Period::Quarter"),
    ("variant", "Period::Year"),
    ("type", "Date"),
    ("field", "Date::day"),
    ("field", "Date::month"),
    ("field", "Date::year"),
    ("fn", "Date::days_in_month"),
    ("fn", "Date::is_leap_year"),
    ("fn", "Date::new"),
    ("fn", "Date::parse"),
    ("type", "FredObservation"),
    ("field", "FredObservation::frequency"),
    ("field", "FredObservation::period"),
    ("field", "FredObservation::series_id"),
    ("field", "FredObservation::unit"),
    ("field", "FredObservation::value"),
    ("field", "FredObservation::vintage"),
    ("type", "FredUnit"),
    ("fn", "FredUnit::as_str"),
    ("fn", "FredUnit::new"),
    ("const", "UNIT_BILLIONS_OF_USD"),
    ("const", "UNIT_MILLIONS_OF_USD"),
    ("fn", "validate_date"),
    ("fn", "validate_observation"),
    ("fn", "validate_period"),
];

/// 覆盖登记表：只登记**真实发生**的调用/读取，不登记「计划要调用」。
mod cover {
    use std::collections::BTreeSet;
    use std::sync::{Mutex, OnceLock};

    static EXECUTED: OnceLock<Mutex<BTreeSet<(&'static str, &'static str)>>> = OnceLock::new();

    fn log() -> &'static Mutex<BTreeSet<(&'static str, &'static str)>> {
        EXECUTED.get_or_init(|| Mutex::new(BTreeSet::new()))
    }

    /// 登记一次真实执行。清单外的 `(类别, id)` 立即 panic，防止调用点与清单漂移。
    pub fn hit(kind: &'static str, id: &'static str) {
        assert!(
            super::E2E_MANIFEST
                .iter()
                .any(|(declared_kind, declared_id)| *declared_kind == kind && *declared_id == id),
            "登记了清单外的公开条目：{kind} {id}"
        );
        log().lock().expect("覆盖登记表锁中毒").insert((kind, id));
    }

    pub fn executed() -> BTreeSet<(&'static str, &'static str)> {
        log().lock().expect("覆盖登记表锁中毒").clone()
    }
}

/// 覆盖登记的简写入口（保持调用点可读）。
fn hit(kind: &'static str, id: &'static str) {
    cover::hit(kind, id);
}

/// 清单自身良构：类别取值域合法、`(类别, id)` 不重复。
fn assert_manifest_wellformed() {
    let mut seen: BTreeSet<(&str, &str)> = BTreeSet::new();
    for &(kind, id) in E2E_MANIFEST {
        assert!(
            matches!(kind, "fn" | "type" | "field" | "const" | "variant"),
            "未知条目类别 {kind}（id={id}）"
        );
        assert!(seen.insert((kind, id)), "清单重复条目：{kind} {id}");
    }
    assert!(!E2E_MANIFEST.is_empty(), "清单不得为空");
}

/// 收尾断言：声明集合与执行集合必须**双向相等**。
fn assert_coverage_complete() {
    let declared: BTreeSet<(&'static str, &'static str)> = E2E_MANIFEST.iter().copied().collect();
    let executed = cover::executed();

    let missing: Vec<&(&str, &str)> = declared.difference(&executed).collect();
    let ghost: Vec<&(&str, &str)> = executed.difference(&declared).collect();

    assert!(
        missing.is_empty(),
        "以下 {} 条公开条目被声明却未执行：{missing:?}",
        missing.len()
    );
    assert!(
        ghost.is_empty(),
        "以下 {} 条执行未登记在清单：{ghost:?}",
        ghost.len()
    );
    eprintln!(
        "E2E 覆盖：{}/{} 条公开条目全部执行（fredx）",
        executed.len(),
        declared.len()
    );
}

/// 构造一条合法观测（供守卫阶段复用）。
fn make_observation(series_id: &str, unit: &str, frequency: Frequency) -> FredObservation {
    FredObservation {
        series_id: series_id.to_owned(),
        period: Period::Day(Date::new(2026, 8, 15).expect("日期合法")),
        value: FredValue::Present(1.0),
        unit: FredUnit::new(unit).expect("单位合法"),
        frequency,
        vintage: None,
    }
}

/// 断言一次授权判定落到 `Denied` 并取出理由。
fn expect_denied(result: FredAuthorization) -> String {
    match result {
        FredAuthorization::Denied { reason } => reason,
        other => panic!("应为 Denied，实得 {other:?}"),
    }
}

/// 阶段 1：53 个公开常量逐条取值断言。
///
/// 38 个 series ID 常量的契约是「字面值 == 常量名」；分组常量按清单声明的条数断言，
/// 并校验分组之和 == 全集、全集无重复。
fn phase_constants() {
    let series_ids: [(&'static str, &'static str); 38] = [
        ("BAMLC0A0CM", BAMLC0A0CM),
        ("BAMLH0A0HYM2", BAMLH0A0HYM2),
        ("BOGZ1FL662090005Q", BOGZ1FL662090005Q),
        ("CES0500000003", CES0500000003),
        ("CPHPTT01EZM659N", CPHPTT01EZM659N),
        ("CPILFESL", CPILFESL),
        ("DCOILWTICO", DCOILWTICO),
        ("DEXCHUS", DEXCHUS),
        ("DEXJPUS", DEXJPUS),
        ("DEXUSEU", DEXUSEU),
        ("DFF", DFF),
        ("DFII10", DFII10),
        ("DGS10", DGS10),
        ("DGS2", DGS2),
        ("FDHBFRBN", FDHBFRBN),
        ("FYFSGDA188S", FYFSGDA188S),
        ("ICSA", ICSA),
        ("IRLTLT01JPM156N", IRLTLT01JPM156N),
        ("JPNCPIALLMINMEI", JPNCPIALLMINMEI),
        ("JTSJOL", JTSJOL),
        ("NASDAQCOM", NASDAQCOM),
        ("NIKKEI225", NIKKEI225),
        ("PAYEMS", PAYEMS),
        ("PCEPILFE", PCEPILFE),
        ("PCOPPUSDM", PCOPPUSDM),
        ("RESPPLLOPNWW", RESPPLLOPNWW),
        ("RRPONTSYD", RRPONTSYD),
        ("SOFR", SOFR),
        ("SP500", SP500),
        ("T10Y2Y", T10Y2Y),
        ("T10YIE", T10YIE),
        ("T5YIFR", T5YIFR),
        ("TEDRATE", TEDRATE),
        ("UNRATE", UNRATE),
        ("VIXCLS", VIXCLS),
        ("WALCL", WALCL),
        ("WRESBAL", WRESBAL),
        ("WTREGEN", WTREGEN),
    ];
    for (name, literal) in series_ids {
        hit("const", name);
        assert_eq!(literal, name, "{name} 的字面值必须等于常量名");
    }

    hit("const", "P0_SERIES");
    assert_eq!(P0_SERIES.len(), 13);
    hit("const", "CURVE_SERIES");
    assert_eq!(CURVE_SERIES.len(), 4);
    hit("const", "FUNDAMENTAL_SERIES");
    assert_eq!(FUNDAMENTAL_SERIES.len(), 4);
    hit("const", "GLOBAL_REGIONAL_SERIES");
    assert_eq!(GLOBAL_REGIONAL_SERIES.len(), 12);
    hit("const", "BLS_FORWARD_SERIES");
    assert_eq!(BLS_FORWARD_SERIES.len(), 2);
    hit("const", "ALL_SERIES");
    assert_eq!(ALL_SERIES.len(), 35);
    assert_eq!(
        P0_SERIES.len()
            + CURVE_SERIES.len()
            + FUNDAMENTAL_SERIES.len()
            + GLOBAL_REGIONAL_SERIES.len()
            + BLS_FORWARD_SERIES.len(),
        ALL_SERIES.len(),
        "5 个分组条数之和必须等于采集全集"
    );
    let unique: BTreeSet<&str> = ALL_SERIES.iter().copied().collect();
    assert_eq!(unique.len(), ALL_SERIES.len(), "采集全集不得有重复 ID");
    for group in [
        P0_SERIES,
        CURVE_SERIES,
        FUNDAMENTAL_SERIES,
        GLOBAL_REGIONAL_SERIES,
        BLS_FORWARD_SERIES,
    ] {
        for id in group {
            assert!(ALL_SERIES.contains(id), "分组项不在全集：{id}");
        }
    }

    hit("const", "AUTHORITATIVE_WRITE_SERIES");
    assert_eq!(AUTHORITATIVE_WRITE_SERIES.len(), 4);
    assert!(
        AUTHORITATIVE_WRITE_SERIES.contains(&WRESBAL),
        "mapping-only 的 WRESBAL 仍在主权集合内"
    );
    hit("const", "BAML_ALLOWED_SERIES");
    assert_eq!(BAML_ALLOWED_SERIES.len(), 1);
    assert_eq!(BAML_ALLOWED_SERIES[0], BAMLH0A0HYM2);

    hit("const", "NEAR_SYNONYM_PAIRS");
    assert_eq!(NEAR_SYNONYM_PAIRS.len(), 14);

    hit("const", "CURVE_BUILD_OWNER");
    assert_eq!(CURVE_BUILD_OWNER, "yieldx");
    hit("const", "BEA_TABLE_OWNER");
    assert_eq!(BEA_TABLE_OWNER, "beax");

    hit("const", "UNIT_MILLIONS_OF_USD");
    assert_eq!(UNIT_MILLIONS_OF_USD, "Millions of U.S. Dollars");
    hit("const", "UNIT_BILLIONS_OF_USD");
    assert_eq!(UNIT_BILLIONS_OF_USD, "Billions of U.S. Dollars");
    assert_ne!(UNIT_MILLIONS_OF_USD, UNIT_BILLIONS_OF_USD);

    hit("const", "FRED_DECISION_ID");
    assert!(!FRED_DECISION_ID.is_empty());
    hit("const", "FRED_SIGNED_BY");
    assert!(!FRED_SIGNED_BY.is_empty());
    assert_ne!(FRED_DECISION_ID, FRED_SIGNED_BY);
}

/// 阶段 2：值对象面（`Date` / `Period` / `Frequency` / `FredUnit` / `FredValue` /
/// `FredObservation`）——变体逐个构造、字段穷尽解构、校验两侧都跑到。
fn phase_value_types() {
    // —— Date：构造 / 解析 / 闰年 / 月天数 / 校验 ——
    hit("type", "Date");
    hit("fn", "Date::new");
    let date = Date::new(black_box(2026), black_box(8), black_box(15)).expect("合法日期");
    let Date { year, month, day } = date;
    hit("field", "Date::year");
    assert_eq!(year, 2026);
    hit("field", "Date::month");
    assert_eq!(month, 8);
    hit("field", "Date::day");
    assert_eq!(day, 15);
    assert!(Date::new(2026, 2, 30).is_err(), "2 月 30 日必须被拒绝");
    assert!(Date::new(0, 1, 1).is_err(), "年份 0 必须被拒绝");
    assert!(Date::new(2026, 13, 1).is_err(), "13 月必须被拒绝");

    hit("fn", "Date::parse");
    let parsed = Date::parse("2026-08-15").expect("严格 ISO 应可解析");
    assert_eq!(parsed, date);
    for bad in [
        "2026-2-3",
        "2026/02/03",
        "2026-02-30",
        "2026-02-03T00:00:00Z",
        "2026-02-03extra",
        "",
    ] {
        assert!(Date::parse(bad).is_err(), "应拒绝：{bad}");
    }
    assert!(Date::parse("2024-02-29").is_ok(), "闰年 2 月 29 日合法");
    assert!(Date::parse("2025-02-29").is_err(), "平年 2 月 29 日非法");

    hit("fn", "Date::is_leap_year");
    assert!(Date::is_leap_year(black_box(2024)));
    assert!(!Date::is_leap_year(black_box(2025)));
    assert!(!Date::is_leap_year(black_box(1900)));
    assert!(Date::is_leap_year(black_box(2000)));

    hit("fn", "Date::days_in_month");
    assert_eq!(Date::days_in_month(black_box(2026), black_box(2)), 28);
    assert_eq!(Date::days_in_month(black_box(2024), black_box(2)), 29);
    assert_eq!(Date::days_in_month(black_box(2026), black_box(12)), 31);
    assert_eq!(Date::days_in_month(black_box(2026), black_box(4)), 30);
    assert_eq!(Date::days_in_month(black_box(2026), black_box(13)), 0);
    assert_eq!(Date::days_in_month(black_box(2026), black_box(0)), 0);

    hit("fn", "validate_date");
    assert!(validate_date(&date).is_ok());
    assert!(validate_date(&Date {
        year: 2026,
        month: 2,
        day: 30
    })
    .is_err());

    // —— Period：5 个变体 ——
    hit("type", "Period");
    hit("variant", "Period::Day");
    let day_period = Period::Day(date);
    hit("variant", "Period::Month");
    let month_period = Period::Month {
        year: 2026,
        month: 8,
    };
    hit("variant", "Period::Quarter");
    let quarter_period = Period::Quarter {
        year: 2026,
        quarter: 3,
    };
    hit("variant", "Period::Year");
    let year_period = Period::Year(2026);
    hit("variant", "Period::Event");
    let event_period = Period::Event { date };
    let periods = [
        day_period,
        month_period,
        quarter_period,
        year_period,
        event_period,
    ];
    let distinct: BTreeSet<String> = periods.iter().map(|p| format!("{p:?}")).collect();
    assert_eq!(distinct.len(), 5, "5 个期间变体必须互异");

    hit("fn", "validate_period");
    for period in periods {
        assert!(validate_period(&period).is_ok(), "{period:?} 应合法");
    }
    assert!(validate_period(&Period::Month {
        year: 2026,
        month: 13
    })
    .is_err());
    assert!(validate_period(&Period::Quarter {
        year: 2026,
        quarter: 5
    })
    .is_err());
    assert!(validate_period(&Period::Year(0)).is_err());
    assert!(validate_period(&Period::Day(Date {
        year: 2026,
        month: 2,
        day: 30
    }))
    .is_err());

    // —— Frequency：7 个变体 + as_str / parse ——
    hit("type", "Frequency");
    hit("variant", "Frequency::Daily");
    let daily = Frequency::Daily;
    hit("variant", "Frequency::Weekly");
    let weekly = Frequency::Weekly;
    hit("variant", "Frequency::Monthly");
    let monthly = Frequency::Monthly;
    hit("variant", "Frequency::Quarterly");
    let quarterly = Frequency::Quarterly;
    hit("variant", "Frequency::Annual");
    let annual = Frequency::Annual;
    hit("variant", "Frequency::Event");
    let event = Frequency::Event;
    hit("variant", "Frequency::Irregular");
    let irregular = Frequency::Irregular;
    let frequencies = [daily, weekly, monthly, quarterly, annual, event, irregular];

    hit("fn", "Frequency::as_str");
    assert_eq!(Frequency::as_str(black_box(Frequency::Daily)), "daily");
    assert_eq!(Frequency::as_str(black_box(Frequency::Weekly)), "weekly");
    assert_eq!(Frequency::as_str(black_box(Frequency::Monthly)), "monthly");
    assert_eq!(
        Frequency::as_str(black_box(Frequency::Quarterly)),
        "quarterly"
    );
    assert_eq!(Frequency::as_str(black_box(Frequency::Annual)), "annual");
    assert_eq!(Frequency::as_str(black_box(Frequency::Event)), "event");
    assert_eq!(
        Frequency::as_str(black_box(Frequency::Irregular)),
        "irregular"
    );

    hit("fn", "Frequency::parse");
    let tokens: BTreeSet<&str> = frequencies.iter().map(|f| f.as_str()).collect();
    assert_eq!(tokens.len(), 7, "7 个频率记号必须互异");
    for frequency in frequencies {
        assert_eq!(
            Frequency::parse(frequency.as_str()).expect("回环"),
            frequency
        );
    }
    assert!(Frequency::parse("Daily").is_err(), "只接受小写");
    assert!(Frequency::parse("fortnightly").is_err());
    assert!(Frequency::parse("").is_err());

    // —— FredUnit：开放 newtype ——
    hit("type", "FredUnit");
    hit("fn", "FredUnit::new");
    let unit = FredUnit::new(black_box(UNIT_MILLIONS_OF_USD)).expect("合法单位");
    hit("fn", "FredUnit::as_str");
    assert_eq!(unit.as_str(), UNIT_MILLIONS_OF_USD);
    assert!(FredUnit::new("").is_err(), "空单位必须被拒绝");
    assert!(FredUnit::new(" Index").is_err(), "首尾空白必须被拒绝");
    assert!(FredUnit::new("Index\n").is_err(), "尾随控制字符必须被拒绝");
    assert!(
        FredUnit::new("Per\u{7}cent").is_err(),
        "内部控制字符必须被拒绝"
    );

    // —— FredValue / FredMissingReason ——
    hit("type", "FredMissingReason");
    hit("variant", "FredMissingReason::NoObservation");
    let missing_reason = FredMissingReason::NoObservation;
    assert!(format!("{missing_reason:?}").contains("NoObservation"));

    hit("type", "FredValue");
    hit("variant", "FredValue::Present");
    let present = FredValue::Present(black_box(3.5));
    hit("variant", "FredValue::Missing");
    let missing = FredValue::Missing(FredMissingReason::NoObservation);
    assert_ne!(present, missing);
    hit("fn", "FredValue::as_f64");
    assert_eq!(present.as_f64(), Some(3.5));
    assert_eq!(missing.as_f64(), None);
    hit("fn", "FredValue::is_missing");
    assert!(missing.is_missing());
    assert!(!present.is_missing());
    assert_ne!(missing.as_f64(), Some(0.0), "缺失不得静默折算为 0");

    // —— FredObservation：6 个公开字段穷尽解构 ——
    hit("type", "FredObservation");
    let observation = make_observation(WALCL, UNIT_MILLIONS_OF_USD, weekly);
    let FredObservation {
        series_id,
        period,
        value,
        unit,
        frequency,
        vintage,
    } = observation;
    hit("field", "FredObservation::series_id");
    assert_eq!(series_id, WALCL);
    hit("field", "FredObservation::period");
    assert_eq!(period, Period::Day(date));
    hit("field", "FredObservation::value");
    assert_eq!(value.as_f64(), Some(1.0));
    hit("field", "FredObservation::unit");
    assert_eq!(unit.as_str(), UNIT_MILLIONS_OF_USD);
    hit("field", "FredObservation::frequency");
    assert_eq!(frequency, weekly);
    hit("field", "FredObservation::vintage");
    assert!(vintage.is_none());

    hit("fn", "validate_observation");
    assert!(validate_observation(&make_observation(WALCL, UNIT_MILLIONS_OF_USD, weekly)).is_ok());
    let mut bad = make_observation(WALCL, UNIT_MILLIONS_OF_USD, weekly);
    bad.series_id = " WALCL".to_owned();
    assert!(
        validate_observation(&bad).is_err(),
        "首尾空白的 series_id 非法"
    );
    let mut bad = make_observation(WALCL, UNIT_MILLIONS_OF_USD, weekly);
    bad.value = FredValue::Present(f64::NAN);
    assert!(validate_observation(&bad).is_err(), "NaN 不得通过");
    let mut bad = make_observation(WALCL, UNIT_MILLIONS_OF_USD, weekly);
    bad.value = FredValue::Present(f64::INFINITY);
    assert!(validate_observation(&bad).is_err(), "无穷不得通过");
    let mut bad = make_observation(WALCL, UNIT_MILLIONS_OF_USD, weekly);
    bad.period = Period::Month {
        year: 2026,
        month: 0,
    };
    assert!(validate_observation(&bad).is_err(), "0 月非法");
    let mut bad = make_observation(WALCL, UNIT_MILLIONS_OF_USD, weekly);
    bad.vintage = Some(Date {
        year: 2026,
        month: 2,
        day: 30,
    });
    assert!(validate_observation(&bad).is_err(), "非法 vintage 非法");
}

/// 阶段 3：错误面（`FredError` 8 变体 + `FredErrorKind` 8 变体 + `FredResult`）。
fn phase_error_types() {
    hit("type", "FredError");
    let cases: [(&'static str, FredError, FredErrorKind, bool); 8] = [
        (
            "FredError::Invalid",
            FredError::Invalid("e2e".into()),
            FredErrorKind::Invalid,
            false,
        ),
        (
            "FredError::Missing",
            FredError::Missing("e2e".into()),
            FredErrorKind::Missing,
            false,
        ),
        (
            "FredError::AuthorizationDenied",
            FredError::AuthorizationDenied("e2e".into()),
            FredErrorKind::AuthorizationDenied,
            false,
        ),
        (
            "FredError::RoutedElsewhere",
            FredError::RoutedElsewhere("e2e".into()),
            FredErrorKind::RoutedElsewhere,
            false,
        ),
        (
            "FredError::WriteAuthorityDenied",
            FredError::WriteAuthorityDenied("e2e".into()),
            FredErrorKind::WriteAuthorityDenied,
            false,
        ),
        (
            "FredError::SemanticallyRejected",
            FredError::SemanticallyRejected("e2e".into()),
            FredErrorKind::SemanticallyRejected,
            false,
        ),
        (
            "FredError::NotApplicable",
            FredError::NotApplicable("e2e".into()),
            FredErrorKind::NotApplicable,
            false,
        ),
        (
            "FredError::Invariant",
            FredError::Invariant("e2e".into()),
            FredErrorKind::Invariant,
            true,
        ),
    ];
    for (variant, error, kind, retryable) in cases {
        hit("variant", variant);
        hit("fn", "FredError::kind");
        assert_eq!(error.kind(), kind, "{variant} 的分类不符");
        hit("fn", "FredError::is_retryable");
        assert_eq!(
            error.is_retryable(),
            retryable,
            "{variant} 的可重试分类不符"
        );
        let shown = error.to_string();
        assert!(!shown.is_empty(), "{variant} 的 Display 不得为空");
        assert!(
            !shown.contains("password") && !shown.contains("api_key"),
            "{variant} 的错误消息不得泄漏凭据字样"
        );
    }

    hit("type", "FredErrorKind");
    let kinds: [(&'static str, FredErrorKind); 8] = [
        ("FredErrorKind::Invalid", FredErrorKind::Invalid),
        ("FredErrorKind::Missing", FredErrorKind::Missing),
        (
            "FredErrorKind::AuthorizationDenied",
            FredErrorKind::AuthorizationDenied,
        ),
        (
            "FredErrorKind::RoutedElsewhere",
            FredErrorKind::RoutedElsewhere,
        ),
        (
            "FredErrorKind::WriteAuthorityDenied",
            FredErrorKind::WriteAuthorityDenied,
        ),
        (
            "FredErrorKind::SemanticallyRejected",
            FredErrorKind::SemanticallyRejected,
        ),
        ("FredErrorKind::NotApplicable", FredErrorKind::NotApplicable),
        ("FredErrorKind::Invariant", FredErrorKind::Invariant),
    ];
    let mut seen = BTreeSet::new();
    for (id, kind) in kinds {
        hit("variant", id);
        assert!(seen.insert(format!("{kind:?}")), "{id} 与其它变体重复");
    }
    assert_eq!(seen.len(), 8);

    hit("type", "FredResult");
    let ok: FredResult<u8> = Ok(7);
    match ok {
        Ok(value) => assert_eq!(value, 7),
        Err(error) => panic!("必须是 Ok 分支，实得 {error}"),
    }
    let err: FredResult<u8> = Err(FredError::Invalid("e2e".into()));
    assert!(err.is_err());
}

/// 阶段 4：publication 语义三元组（`TimePrecision` / `AvailabilityEvidence` /
/// `PitEligibility` / `FredPublicationSemantics`）。
fn phase_pit() {
    hit("type", "TimePrecision");
    hit("variant", "TimePrecision::Date");
    hit("variant", "TimePrecision::Instant");
    assert_ne!(TimePrecision::Date, TimePrecision::Instant);

    hit("type", "AvailabilityEvidence");
    hit("variant", "AvailabilityEvidence::Official");
    hit("variant", "AvailabilityEvidence::Calendar");
    hit("variant", "AvailabilityEvidence::Inferred");
    let layers = [
        AvailabilityEvidence::Official,
        AvailabilityEvidence::Calendar,
        AvailabilityEvidence::Inferred,
    ];
    assert_eq!(
        layers
            .iter()
            .map(|l| format!("{l:?}"))
            .collect::<BTreeSet<_>>()
            .len(),
        3
    );

    hit("type", "PitEligibility");
    hit("variant", "PitEligibility::Formal");
    hit("variant", "PitEligibility::NotEligible");
    assert_ne!(PitEligibility::Formal, PitEligibility::NotEligible);

    hit("fn", "fred_publication_semantics");
    let semantics = fred_publication_semantics();
    hit("type", "FredPublicationSemantics");
    let FredPublicationSemantics {
        time_precision,
        availability,
        eligibility,
    } = semantics;
    hit("field", "FredPublicationSemantics::time_precision");
    assert_eq!(time_precision, TimePrecision::Date);
    hit("field", "FredPublicationSemantics::availability");
    assert_eq!(availability, AvailabilityEvidence::Inferred);
    hit("field", "FredPublicationSemantics::eligibility");
    assert_eq!(eligibility, PitEligibility::NotEligible);
    assert_eq!(semantics, fred_publication_semantics(), "三元组必须是常量");
    assert_ne!(
        semantics.time_precision,
        TimePrecision::Instant,
        "MUST NOT 升格为 Instant"
    );
    assert_ne!(semantics.eligibility, PitEligibility::Formal);

    hit("fn", "is_formal_pit_eligible");
    assert!(!is_formal_pit_eligible(), "本层正式 PIT 资格恒为 false");
}

/// 阶段 5：跨源守卫面（近义非同 ID / BAML 冻结令 / 写入主权 / 曲线边界 / BEA 表号）。
fn phase_routing() {
    hit("type", "FredProduct");
    hit("variant", "FredProduct::Observation");
    hit("variant", "FredProduct::YieldCurve");
    assert_ne!(FredProduct::Observation, FredProduct::YieldCurve);

    hit("fn", "ensure_product_local");
    assert!(ensure_product_local(FredProduct::Observation).is_ok());
    let routed = ensure_product_local(FredProduct::YieldCurve).expect_err("曲线必须被路由");
    assert_eq!(routed.kind(), FredErrorKind::RoutedElsewhere);
    assert!(routed.to_string().contains(CURVE_BUILD_OWNER));

    hit("fn", "is_bea_table_id");
    assert!(is_bea_table_id("T10101"));
    assert!(is_bea_table_id("T20100"));
    assert!(!is_bea_table_id("T10Y2Y"), "T10Y2Y 非 BEA 表号");
    assert!(!is_bea_table_id("T1010"), "长度不足");
    assert!(!is_bea_table_id("t10101"), "小写 t 非 BEA 表号");
    assert!(!is_bea_table_id("NIPA"));

    hit("fn", "is_near_synonym_pair");
    for (left, right) in NEAR_SYNONYM_PAIRS {
        assert!(is_near_synonym_pair(left, right), "{left}/{right} 应被识别");
        assert!(is_near_synonym_pair(right, left), "{right}/{left} 应被识别");
    }
    assert!(!is_near_synonym_pair(WALCL, RRPONTSYD));

    hit("fn", "ensure_not_silent_substitution");
    for (left, right) in NEAR_SYNONYM_PAIRS {
        let err = ensure_not_silent_substitution(left, right).expect_err("近义对必须拒绝");
        assert_eq!(err.kind(), FredErrorKind::SemanticallyRejected);
    }
    assert!(ensure_not_silent_substitution(WALCL, RRPONTSYD).is_ok());

    hit("fn", "ensure_baml_freeze");
    assert!(ensure_baml_freeze(BAMLH0A0HYM2).is_ok(), "唯一允许的 BAML");
    assert!(ensure_baml_freeze(WALCL).is_ok(), "非 BAML 不受冻结令约束");
    let dropped = ensure_baml_freeze(BAMLC0A0CM).expect_err("已 drop 的 BAML 必须拒绝");
    assert_eq!(dropped.kind(), FredErrorKind::SemanticallyRejected);
    assert!(dropped.to_string().contains("书面豁免"));
    assert!(
        ensure_baml_freeze("BAMLH0A3HYC").is_err(),
        "其它 BAML 一律拒绝"
    );

    hit("fn", "claim_authoritative_write");
    for id in AUTHORITATIVE_WRITE_SERIES {
        assert!(claim_authoritative_write(id).is_ok(), "{id} 应为主权序列");
    }
    let curve = claim_authoritative_write(DGS10).expect_err("曲线点");
    assert_eq!(curve.kind(), FredErrorKind::WriteAuthorityDenied);
    assert!(curve.to_string().contains(CURVE_BUILD_OWNER));
    let table = claim_authoritative_write("T20100").expect_err("BEA 表号");
    assert_eq!(table.kind(), FredErrorKind::WriteAuthorityDenied);
    assert!(table.to_string().contains(BEA_TABLE_OWNER));
    let pending = claim_authoritative_write(RESPPLLOPNWW).expect_err("未决候选");
    assert_eq!(pending.kind(), FredErrorKind::WriteAuthorityDenied);
    let other = claim_authoritative_write(SP500).expect_err("非主权序列");
    assert_eq!(other.kind(), FredErrorKind::WriteAuthorityDenied);
}

/// 阶段 6：series 事实面（采集范围 / 单位 / 频率 / 主权 + 三条守卫的合法与非法两侧）。
fn phase_series() {
    hit("fn", "is_collected_series");
    assert!(is_collected_series(WALCL));
    assert!(!is_collected_series(WRESBAL), "mapping-only 不进采集集合");
    assert!(!is_collected_series("NOT_A_FRED_ID"));

    hit("fn", "is_mapping_only_series");
    assert!(is_mapping_only_series(WRESBAL));
    assert!(!is_mapping_only_series(WALCL));

    hit("fn", "is_known_series");
    assert!(is_known_series(WALCL));
    assert!(is_known_series(WRESBAL), "mapping-only 仍属已登记");
    assert!(!is_known_series(BAMLC0A0CM), "已 drop 的 ID 不再登记");
    assert!(!is_known_series("NOT_A_FRED_ID"));

    hit("fn", "is_curve_input_series");
    assert!(is_curve_input_series(DGS2));
    assert!(is_curve_input_series(DGS10));
    assert!(is_curve_input_series(T10Y2Y));
    assert!(is_curve_input_series(T10YIE));
    assert!(!is_curve_input_series(BAMLH0A0HYM2));
    assert!(!is_curve_input_series(T5YIFR));

    hit("fn", "series_unit");
    assert_eq!(series_unit(WALCL), Some(UNIT_MILLIONS_OF_USD));
    assert_eq!(series_unit(WTREGEN), Some(UNIT_MILLIONS_OF_USD));
    assert_eq!(series_unit(RRPONTSYD), Some(UNIT_BILLIONS_OF_USD));
    assert_ne!(
        series_unit(WALCL),
        series_unit(RRPONTSYD),
        "量级不同不得混同"
    );
    assert_eq!(series_unit(SP500), None, "未声明源单位的序列返回 None");

    hit("fn", "series_frequency");
    for id in ALL_SERIES {
        assert!(series_frequency(id).is_some(), "采集集合缺频率声明：{id}");
    }
    assert_eq!(series_frequency(ICSA), Some(Frequency::Weekly));
    assert_eq!(series_frequency(FYFSGDA188S), Some(Frequency::Quarterly));
    assert_eq!(series_frequency(PCEPILFE), Some(Frequency::Monthly));
    assert_eq!(series_frequency(WRESBAL), None);

    hit("fn", "is_unique_write_authority");
    for id in AUTHORITATIVE_WRITE_SERIES {
        assert!(is_unique_write_authority(id), "{id}");
    }
    assert!(!is_unique_write_authority(SP500));
    assert!(!is_unique_write_authority(RESPPLLOPNWW));

    hit("fn", "ensure_known_series");
    assert!(ensure_known_series(WALCL).is_ok());
    assert!(ensure_known_series(WRESBAL).is_ok());
    let unknown = ensure_known_series("CPIAUCSL").expect_err("清单外 ID 必须拒绝");
    assert_eq!(unknown.kind(), FredErrorKind::SemanticallyRejected);

    hit("fn", "ensure_source_unit");
    assert!(ensure_source_unit(&make_observation(
        WALCL,
        UNIT_MILLIONS_OF_USD,
        Frequency::Weekly
    ))
    .is_ok());
    let wrong_unit = ensure_source_unit(&make_observation(
        RRPONTSYD,
        UNIT_MILLIONS_OF_USD,
        Frequency::Daily,
    ))
    .expect_err("把 Billions 当 Millions 必须拒绝");
    assert_eq!(wrong_unit.kind(), FredErrorKind::SemanticallyRejected);
    assert!(
        ensure_source_unit(&make_observation(SP500, "Index", Frequency::Daily)).is_ok(),
        "未声明源单位的序列不做该判定"
    );

    hit("fn", "ensure_source_frequency");
    assert!(ensure_source_frequency(&make_observation(
        WALCL,
        UNIT_MILLIONS_OF_USD,
        Frequency::Weekly
    ))
    .is_ok());
    let wrong_frequency = ensure_source_frequency(&make_observation(
        WALCL,
        UNIT_MILLIONS_OF_USD,
        Frequency::Monthly,
    ))
    .expect_err("频率漂移必须拒绝");
    assert_eq!(wrong_frequency.kind(), FredErrorKind::SemanticallyRejected);
}

/// 阶段 7：授权面（证据字段穷尽解构 + `authorize_fred` 的授权与**全部**拒绝分支）。
fn phase_authz() {
    hit("type", "FredAccessMode");
    hit("variant", "FredAccessMode::Offline");
    hit("variant", "FredAccessMode::Live");
    assert_ne!(FredAccessMode::Offline, FredAccessMode::Live);

    hit("fn", "mode_label");
    assert_eq!(mode_label(FredAccessMode::Offline), "offline");
    assert_eq!(mode_label(FredAccessMode::Live), "live");

    hit("type", "FredAuthorization");
    hit("variant", "FredAuthorization::Authorized");
    let authorized = FredAuthorization::Authorized {
        scope: "scope".to_owned(),
    };
    hit("variant", "FredAuthorization::Denied");
    let denied = FredAuthorization::Denied {
        reason: "reason".to_owned(),
    };
    assert_ne!(authorized, denied);
    assert!(format!("{authorized:?}").contains("Authorized"));
    assert!(format!("{denied:?}").contains("Denied"));

    // —— 证据：6 个公开字段穷尽解构 ——
    hit("fn", "documented_fred_evidence");
    let documented = documented_fred_evidence();
    hit("type", "FredAuthorizationEvidence");
    let FredAuthorizationEvidence {
        decision_id,
        signed_by,
        signed_at,
        valid_until,
        authorized_modes,
        scope_note,
    } = documented.clone();
    hit("field", "FredAuthorizationEvidence::decision_id");
    assert_eq!(decision_id, FRED_DECISION_ID);
    hit("field", "FredAuthorizationEvidence::signed_by");
    assert_eq!(signed_by, FRED_SIGNED_BY);
    hit("field", "FredAuthorizationEvidence::signed_at");
    assert_eq!(signed_at, Date::new(2026, 8, 15).ok());
    hit("field", "FredAuthorizationEvidence::valid_until");
    assert_eq!(valid_until, None, "清单登记的证据未声明有效期");
    hit("field", "FredAuthorizationEvidence::authorized_modes");
    assert_eq!(authorized_modes, vec![FredAccessMode::Offline]);
    hit("field", "FredAuthorizationEvidence::scope_note");
    assert!(!scope_note.is_empty());

    // —— 证据形态校验：合法 / 日期非法 / 区间倒置 ——
    hit("fn", "validate_authorization_evidence");
    assert!(validate_authorization_evidence(&documented).is_ok());
    let mut invalid_date = documented.clone();
    invalid_date.valid_until = Some(Date {
        year: 2026,
        month: 2,
        day: 30,
    });
    assert!(validate_authorization_evidence(&invalid_date).is_err());
    let mut inverted = documented.clone();
    inverted.signed_at = Date::new(2026, 9, 23).ok();
    inverted.valid_until = Date::new(2026, 9, 22).ok();
    assert!(
        validate_authorization_evidence(&inverted).is_err(),
        "签署日晚于有效期上界必须拒绝"
    );

    // —— authorize_fred：全部拒绝分支 + 授权路径 ——
    hit("fn", "authorize_fred");
    let as_of = Date::new(black_box(2026), black_box(8), black_box(20)).expect("合法日期");

    assert_eq!(
        expect_denied(authorize_fred(None, FredAccessMode::Offline, as_of)),
        "缺少 Owner 签核证据"
    );

    let mut blank = documented.clone();
    blank.decision_id = "   ".to_owned();
    assert_eq!(
        expect_denied(authorize_fred(Some(&blank), FredAccessMode::Offline, as_of)),
        "签核编号不明"
    );

    let mut blank = documented.clone();
    blank.signed_by = String::new();
    assert_eq!(
        expect_denied(authorize_fred(Some(&blank), FredAccessMode::Offline, as_of)),
        "签署者不明"
    );

    let mut blank = documented.clone();
    blank.scope_note = "\u{3000}".to_owned();
    assert_eq!(
        expect_denied(authorize_fred(Some(&blank), FredAccessMode::Offline, as_of)),
        "证据范围说明不明"
    );

    let illegal_as_of = Date {
        year: 2026,
        month: 99,
        day: 99,
    };
    assert_eq!(
        expect_denied(authorize_fred(
            Some(&documented),
            FredAccessMode::Offline,
            illegal_as_of
        )),
        "评估日期或证据有效区间非法"
    );

    assert_eq!(
        expect_denied(authorize_fred(
            Some(&invalid_date),
            FredAccessMode::Offline,
            as_of
        )),
        "评估日期或证据有效区间非法"
    );

    let mut early = documented.clone();
    early.signed_at = Date::new(2026, 9, 23).ok();
    assert_eq!(
        expect_denied(authorize_fred(Some(&early), FredAccessMode::Offline, as_of)),
        "证据尚未签署生效"
    );

    let mut uncovered = documented.clone();
    uncovered.authorized_modes.clear();
    assert_eq!(
        expect_denied(authorize_fred(
            Some(&uncovered),
            FredAccessMode::Offline,
            as_of
        )),
        "证据未声明任何被覆盖的范围"
    );

    let mut expired = documented.clone();
    expired.valid_until = Date::new(2026, 8, 16).ok();
    assert_eq!(
        expect_denied(authorize_fred(
            Some(&expired),
            FredAccessMode::Offline,
            as_of
        )),
        "证据已过期"
    );

    let not_covered = expect_denied(authorize_fred(
        Some(&documented),
        FredAccessMode::Live,
        as_of,
    ));
    assert!(not_covered.contains("未被证据覆盖"), "实得：{not_covered}");

    match authorize_fred(Some(&documented), FredAccessMode::Offline, as_of) {
        FredAuthorization::Authorized { scope } => {
            assert!(scope.contains(FRED_DECISION_ID));
            assert!(scope.contains("offline"));
        }
        other => panic!("应授权 offline，实得 {other:?}"),
    }

    // 边界：签署日 == 到期日 == 评估日仍授权；次日过期。
    let mut boundary_evidence = documented.clone();
    boundary_evidence.signed_at = Date::new(2026, 9, 23).ok();
    boundary_evidence.valid_until = Date::new(2026, 9, 23).ok();
    let boundary = Date::new(2026, 9, 23).expect("合法日期");
    assert!(matches!(
        authorize_fred(Some(&boundary_evidence), FredAccessMode::Offline, boundary),
        FredAuthorization::Authorized { .. }
    ));
    assert!(matches!(
        authorize_fred(
            Some(&boundary_evidence),
            FredAccessMode::Offline,
            Date::new(2026, 9, 24).expect("合法日期")
        ),
        FredAuthorization::Denied { .. }
    ));
}

/// 阶段 8：真实离线解析面（成功路径 + 各类拒绝路径，全部落在合成夹具上）。
fn phase_parse() {
    hit("fn", "parse_fred_observations");

    // 合成输入带 `_synthetic` 标注（该仓硬约束）；不作为任何证据。
    let synthetic = r#"{
        "_synthetic": true,
        "_note": "合成样本，仅为 E2E 走通离线解析路径",
        "records": [
            {"series_id": "WALCL", "date": "2026-08-13", "value": 7000000.0,
             "unit": "Millions of U.S. Dollars", "frequency": "weekly"},
            {"series_id": "RRPONTSYD", "date": "2026-08-14", "value": 450.25,
             "unit": "Billions of U.S. Dollars", "frequency": "daily"},
            {"series_id": "PCEPILFE", "date": "2026-06-30", "value": 1.0,
             "unit": "Index", "frequency": "monthly"},
            {"series_id": "FYFSGDA188S", "date": "2026-06-30", "value": -6.5,
             "unit": "Percent", "frequency": "quarterly"},
            {"series_id": "SOFR", "date": "2026-08-14", "value": ".", "unit": "Percent",
             "frequency": "daily"},
            {"series_id": "DFF", "date": "2026-08-14", "value": null, "unit": "Percent",
             "frequency": "daily"},
            {"series_id": "UNRATE", "date": "2026-06-30", "value": 4.2, "unit": "Percent",
             "frequency": "monthly", "vintage": "2026-07-03"}
        ]
    }"#;
    let observations = parse_fred_observations(synthetic).expect("合成输入必须可解析");
    assert_eq!(observations.len(), 7);
    assert_eq!(observations[0].series_id, WALCL);
    assert_eq!(
        observations[0].period,
        Period::Day(Date::new(2026, 8, 13).expect("合法日期"))
    );
    assert_eq!(observations[0].value.as_f64(), Some(7_000_000.0));
    assert!(observations[0].vintage.is_none());
    assert_eq!(observations[1].frequency, Frequency::Daily);
    assert_eq!(
        observations[2].period,
        Period::Month {
            year: 2026,
            month: 6
        }
    );
    assert_eq!(
        observations[3].period,
        Period::Quarter {
            year: 2026,
            quarter: 2
        }
    );
    assert!(observations[4].value.is_missing(), "`.` 必须是具名缺失");
    assert!(observations[5].value.is_missing(), "null 必须是具名缺失");
    assert_ne!(
        observations[4].value.as_f64(),
        Some(0.0),
        "缺失不得折算为 0"
    );
    assert_eq!(
        observations[6].vintage,
        Some(Date::new(2026, 7, 3).expect("合法日期"))
    );

    // 空 records → 空集合。
    assert!(parse_fred_observations(r#"{"records": []}"#)
        .expect("空 records 合法")
        .is_empty());

    // 未知字段 → 原子失败（Invalid，且不回显输入）。
    let unknown_record_field = r#"{"records": [
        {"series_id": "SOFR", "date": "2026-08-14", "value": 1.0, "unit": "Percent",
         "frequency": "daily", "endpoint": "x"}
    ]}"#;
    let err = parse_fred_observations(unknown_record_field).expect_err("记录级未知字段必须拒绝");
    assert_eq!(err.kind(), FredErrorKind::Invalid);
    assert!(!err.to_string().contains("endpoint"), "不得回显未知字段名");
    assert!(
        parse_fred_observations(r#"{"records": [], "token": "x"}"#).is_err(),
        "顶层未知字段必须拒绝"
    );

    // 缺必需字段 → Invalid。
    let missing_field = r#"{"records": [
        {"date": "2026-08-14", "value": 1.0, "unit": "Percent", "frequency": "daily"}
    ]}"#;
    assert!(parse_fred_observations(missing_field).is_err());

    // 清单外 series → SemanticallyRejected。
    let out_of_scope = r#"{"records": [
        {"series_id": "CPIAUCSL", "date": "2026-06-30", "value": 1.0, "unit": "Index",
         "frequency": "monthly"}
    ]}"#;
    let err = parse_fred_observations(out_of_scope).expect_err("清单外 ID 必须拒绝");
    assert_eq!(err.kind(), FredErrorKind::SemanticallyRejected);

    // BAML 冻结令 → SemanticallyRejected。
    let baml = r#"{"records": [
        {"series_id": "BAMLC0A0CM", "date": "2026-08-14", "value": 1.0, "unit": "Percent",
         "frequency": "daily"}
    ]}"#;
    let err = parse_fred_observations(baml).expect_err("BAML 冻结令必须拒绝");
    assert_eq!(err.kind(), FredErrorKind::SemanticallyRejected);

    // 单位漂移 / 频率漂移 → SemanticallyRejected。
    let wrong_unit = r#"{"records": [
        {"series_id": "RRPONTSYD", "date": "2026-08-14", "value": 450.25,
         "unit": "Millions of U.S. Dollars", "frequency": "daily"}
    ]}"#;
    let err = parse_fred_observations(wrong_unit).expect_err("单位漂移必须拒绝");
    assert_eq!(err.kind(), FredErrorKind::SemanticallyRejected);
    let wrong_frequency = r#"{"records": [
        {"series_id": "WALCL", "date": "2026-08-13", "value": 1.0,
         "unit": "Millions of U.S. Dollars", "frequency": "daily"}
    ]}"#;
    let err = parse_fred_observations(wrong_frequency).expect_err("频率漂移必须拒绝");
    assert_eq!(err.kind(), FredErrorKind::SemanticallyRejected);

    // 未知频率记号 → Invalid。
    let unknown_frequency = r#"{"records": [
        {"series_id": "SOFR", "date": "2026-08-14", "value": 1.0, "unit": "Percent",
         "frequency": "fortnightly"}
    ]}"#;
    let err = parse_fred_observations(unknown_frequency).expect_err("未知频率记号必须拒绝");
    assert_eq!(err.kind(), FredErrorKind::Invalid);

    // 重复身份（series_id + 期间 + vintage）→ 拒绝且不去重。
    let duplicate = r#"{"records": [
        {"series_id": "SOFR", "date": "2026-08-14", "value": 1.0, "unit": "Percent",
         "frequency": "daily"},
        {"series_id": "SOFR", "date": "2026-08-14", "value": 1.1, "unit": "Percent",
         "frequency": "daily"}
    ]}"#;
    let err = parse_fred_observations(duplicate).expect_err("重复身份必须拒绝");
    assert_eq!(err.kind(), FredErrorKind::SemanticallyRejected);

    // 非法日期 → Invalid。
    for bad_date in [
        "2026-2-3",
        "2026/02/03",
        "2026-02-30",
        "2026-08-14T00:00:00Z",
    ] {
        let input = format!(
            r#"{{"records": [{{"series_id": "SOFR", "date": "{bad_date}", "value": 1.0,
                "unit": "Percent", "frequency": "daily"}}]}}"#
        );
        let err = parse_fred_observations(&input).expect_err("非法日期必须拒绝");
        assert_eq!(err.kind(), FredErrorKind::Invalid, "{bad_date}");
    }

    // 非法值形态 → Invalid（不得把缺失折算为 0）。
    let bad_value = r#"{"records": [
        {"series_id": "SOFR", "date": "2026-08-14", "value": "n/a", "unit": "Percent",
         "frequency": "daily"}
    ]}"#;
    let err = parse_fred_observations(bad_value).expect_err("非法值形态必须拒绝");
    assert_eq!(err.kind(), FredErrorKind::Invalid);

    // 非法 JSON → Invalid，错误消息只给位置、不回显输入。
    let err = parse_fred_observations("{ not json").expect_err("非法 JSON 必须拒绝");
    assert_eq!(err.kind(), FredErrorKind::Invalid);
    assert!(!err.to_string().contains("not json"), "不得回显原始输入");
    assert!(err.to_string().contains('第'), "应给出行列位置");
}

/// 单一驱动用例：fredx 无进程级共享状态，但保持单驱动更易归因。
#[test]
fn e2e_fred_all_public_api() {
    assert_manifest_wellformed();
    phase_constants();
    phase_value_types();
    phase_error_types();
    phase_pit();
    phase_routing();
    phase_series();
    phase_authz();
    phase_parse();
    assert_coverage_complete();
}
