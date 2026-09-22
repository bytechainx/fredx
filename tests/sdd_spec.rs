#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! SDD 规格对照（特性 005）：把 `docs/标准.md` 的每个 `##` 章节转成可执行断言。
//!
//! 章节与断言函数须与 `docs/标准.md` 的 `##` 章节 1:1（检查器按标题逐字比对）。
//!
//! // SPEC-MAP: S-1 | 1. 源事实与采集范围标准 | assert_source_scope
//! // SPEC-MAP: S-2 | 2. 值与期间标准 | assert_value_and_period
//! // SPEC-MAP: S-3 | 3. 单位与不换算标准 | assert_units_without_conversion
//! // SPEC-MAP: S-4 | 4. 跨源守卫标准（近义非同 ID / BAML 冻结令） | assert_cross_source_guards
//! // SPEC-MAP: S-5 | 5. 写入主权与曲线边界标准 | assert_write_authority_and_curve_boundary
//! // SPEC-MAP: S-6 | 6. publication 语义标准 | assert_publication_semantics
//! // SPEC-MAP: S-7 | 7. 授权判定标准（fail-closed） | assert_authorization_fail_closed
//! // SPEC-MAP: S-8 | 8. 离线解析标准 | assert_offline_parsing
//! // SPEC-MAP: S-9 | 9. 合成夹具声明 | assert_synthetic_fixtures_declared
//! // SPEC-MAP: S-10 | 10. 依赖与零网络标准 | assert_no_network_and_no_coupling
//! // SPEC-MAP: S-11 | 11. 验收 | assert_acceptance_surface

use fredx::{
    authorize_fred, claim_authoritative_write, documented_fred_evidence, ensure_baml_freeze,
    ensure_not_silent_substitution, ensure_product_local, ensure_source_frequency,
    ensure_source_unit, fred_publication_semantics, is_collected_series, is_curve_input_series,
    is_formal_pit_eligible, is_mapping_only_series, is_unique_write_authority,
    parse_fred_observations, validate_date, validate_observation, validate_period, Date,
    FredAccessMode, FredAuthorization, FredErrorKind, FredMissingReason, FredObservation,
    FredProduct, FredUnit, FredValue, Frequency, Period, ALL_SERIES, AUTHORITATIVE_WRITE_SERIES,
    UNIT_BILLIONS_OF_USD, UNIT_MILLIONS_OF_USD,
};

const FIXTURE: &str = include_str!("fixtures/fred_observations_synthetic.json");

fn observation(series_id: &str, unit: &str, frequency: Frequency) -> FredObservation {
    FredObservation {
        series_id: series_id.to_owned(),
        period: Period::Day(Date::new(2026, 8, 14).expect("日期合法")),
        value: FredValue::Present(1.0),
        unit: FredUnit::new(unit).expect("单位合法"),
        frequency,
        vintage: None,
    }
}

/// 运行期**递归枚举** `<root>` 下全部 `*.rs` 文件。
///
/// 刻意不用手写文件清单、也不用 `include_str!` 逐个点名：清单会漏掉既有文件，
/// 且新文件永远进不了扫描面（这正是本断言要消灭的盲区）。
fn collect_src_rs(root: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_src_rs(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// S-1：采集集合为 35 个纯 FRED ID；`WRESBAL` 为 mapping-only；范围外 ID 被拒。
#[test]
fn assert_source_scope() {
    assert_eq!(ALL_SERIES.len(), 35);
    assert!(is_collected_series("WALCL"));
    assert!(!is_collected_series("WRESBAL"));
    assert!(is_mapping_only_series("WRESBAL"));
    assert!(!is_collected_series("BAMLC0A0CM"));
    assert!(!is_collected_series("CPIAUCSL"));
    assert!(!is_collected_series("RESPPLLOPNWW"));
    assert!(!is_collected_series("GDPNow"));
}

/// S-2：期间与值形态；缺失具名、非有限数被拒。
#[test]
fn assert_value_and_period() {
    assert!(validate_date(&Date::new(2026, 8, 14).expect("日期合法")).is_ok());
    assert!(validate_period(&Period::Quarter {
        year: 2026,
        quarter: 5
    })
    .is_err());
    assert_eq!(
        Frequency::parse("monthly").expect("记号"),
        Frequency::Monthly
    );
    assert!(Frequency::parse("Monthly").is_err());

    let mut record = observation("WALCL", UNIT_MILLIONS_OF_USD, Frequency::Weekly);
    assert!(validate_observation(&record).is_ok());
    record.value = FredValue::Missing(FredMissingReason::NoObservation);
    assert!(validate_observation(&record).is_ok());
    record.value = FredValue::Present(f64::NAN);
    assert!(validate_observation(&record).is_err());
}

/// S-3：保留源单位、不做换算；Billions 与 Millions 不得混用。
#[test]
fn assert_units_without_conversion() {
    let billions = observation("RRPONTSYD", UNIT_BILLIONS_OF_USD, Frequency::Daily);
    assert!(ensure_source_unit(&billions).is_ok());
    assert_eq!(billions.value.as_f64(), Some(1.0));

    let as_millions = observation("RRPONTSYD", UNIT_MILLIONS_OF_USD, Frequency::Daily);
    let err = ensure_source_unit(&as_millions).expect_err("应拒绝");
    assert_eq!(err.kind(), FredErrorKind::SemanticallyRejected);
    // 拒绝时值未被改写（不做换算）。
    assert_eq!(as_millions.value.as_f64(), Some(1.0));

    assert!(ensure_source_frequency(&observation(
        "WALCL",
        UNIT_MILLIONS_OF_USD,
        Frequency::Weekly
    ))
    .is_ok());
}

/// S-4：14 对近义非同 ID 双向禁止；BAML 冻结令只允许 `BAMLH0A0HYM2`。
#[test]
fn assert_cross_source_guards() {
    assert_eq!(fredx::NEAR_SYNONYM_PAIRS.len(), 14);
    for (left, right) in fredx::NEAR_SYNONYM_PAIRS {
        assert!(ensure_not_silent_substitution(left, right).is_err());
        assert!(ensure_not_silent_substitution(right, left).is_err());
    }
    assert!(ensure_not_silent_substitution("WALCL", "RRPONTSYD").is_ok());
    assert!(ensure_baml_freeze("BAMLH0A0HYM2").is_ok());
    assert!(ensure_baml_freeze("BAMLC0A0CM").is_err());
    assert!(ensure_baml_freeze("BAMLH0A3HYC").is_err());
}

/// S-5：四条序列的权威写入唯一归本域；曲线构建路由 `yieldx`。
#[test]
fn assert_write_authority_and_curve_boundary() {
    assert_eq!(AUTHORITATIVE_WRITE_SERIES.len(), 4);
    for id in ["WALCL", "WTREGEN", "RRPONTSYD", "WRESBAL"] {
        assert!(is_unique_write_authority(id));
        assert!(claim_authoritative_write(id).is_ok());
    }
    assert_eq!(
        claim_authoritative_write("DGS10")
            .expect_err("曲线点")
            .kind(),
        FredErrorKind::WriteAuthorityDenied
    );
    assert_eq!(
        claim_authoritative_write("T20100")
            .expect_err("BEA 表号")
            .kind(),
        FredErrorKind::WriteAuthorityDenied
    );
    assert!(ensure_product_local(FredProduct::Observation).is_ok());
    assert_eq!(
        ensure_product_local(FredProduct::YieldCurve)
            .expect_err("曲线")
            .kind(),
        FredErrorKind::RoutedElsewhere
    );
    assert!(is_curve_input_series("DGS2"));
    assert!(is_curve_input_series("T10Y2Y"));
    assert!(!is_curve_input_series("WALCL"));
}

/// S-6：publication 三元组恒为 `(Date, Inferred, NotEligible)`；正式 PIT 恒 `false`。
#[test]
fn assert_publication_semantics() {
    let semantics = fred_publication_semantics();
    assert_eq!(semantics.time_precision, fredx::TimePrecision::Date);
    assert_eq!(
        semantics.availability,
        fredx::AvailabilityEvidence::Inferred
    );
    assert_eq!(semantics.eligibility, fredx::PitEligibility::NotEligible);
    assert!(!is_formal_pit_eligible());
}

/// S-7：offline 被覆盖、live 不被覆盖；缺失/不明/过期一律拒绝。
#[test]
fn assert_authorization_fail_closed() {
    let as_of = Date::new(2026, 8, 20).expect("日期合法");
    let evidence = documented_fred_evidence();
    assert!(matches!(
        authorize_fred(Some(&evidence), FredAccessMode::Offline, as_of),
        FredAuthorization::Authorized { .. }
    ));
    assert!(matches!(
        authorize_fred(Some(&evidence), FredAccessMode::Live, as_of),
        FredAuthorization::Denied { .. }
    ));
    assert!(matches!(
        authorize_fred(None, FredAccessMode::Offline, as_of),
        FredAuthorization::Denied { .. }
    ));

    let mut unknown_scope = documented_fred_evidence();
    unknown_scope.authorized_modes.clear();
    assert!(matches!(
        authorize_fred(Some(&unknown_scope), FredAccessMode::Offline, as_of),
        FredAuthorization::Denied { .. }
    ));

    let mut expired = documented_fred_evidence();
    expired.valid_until = Date::new(2026, 8, 16).ok();
    assert!(matches!(
        authorize_fred(Some(&expired), FredAccessMode::Offline, as_of),
        FredAuthorization::Denied { .. }
    ));
}

/// S-8：未知字段原子失败、重复身份拒绝、缺失具名、错误不回声输入。
#[test]
fn assert_offline_parsing() {
    let observations = parse_fred_observations(FIXTURE).expect("合成夹具应可解析");
    assert_eq!(observations.len(), 10);
    assert!(observations.iter().all(|o| !o.value.is_missing()));

    let unknown_field = r#"{"records": [
        {"series_id": "SOFR", "date": "2026-08-14", "value": 1.0, "unit": "Percent",
         "frequency": "daily", "extra": 1}]}"#;
    assert!(parse_fred_observations(unknown_field).is_err());

    let missing_field = r#"{"records": [
        {"date": "2026-08-14", "value": 1.0, "unit": "Percent", "frequency": "daily"}]}"#;
    assert!(parse_fred_observations(missing_field).is_err());

    let duplicate = r#"{"records": [
        {"series_id": "SOFR", "date": "2026-08-14", "value": 1.0, "unit": "Percent",
         "frequency": "daily"},
        {"series_id": "SOFR", "date": "2026-08-14", "value": 1.1, "unit": "Percent",
         "frequency": "daily"}]}"#;
    assert_eq!(
        parse_fred_observations(duplicate)
            .expect_err("重复身份")
            .kind(),
        FredErrorKind::SemanticallyRejected
    );

    let bad = parse_fred_observations("{ oops").expect_err("非法 JSON");
    assert!(!bad.to_string().contains("oops"));
}

/// S-9：夹具自带合成标注，且不被表述为证据。
#[test]
fn assert_synthetic_fixtures_declared() {
    assert!(FIXTURE.contains("\"_synthetic\": true"));
    assert!(FIXTURE.contains("不是真实源数据，不构成任何证据"));
    let missing_fixture = include_str!("fixtures/fred_missing_value_synthetic.json");
    assert!(missing_fixture.contains("\"_synthetic\": true"));
    let observations = parse_fred_observations(missing_fixture).expect("应可解析");
    assert_eq!(observations.len(), 2);
    for record in &observations {
        assert!(record.value.is_missing());
        assert_eq!(record.value.as_f64(), None);
    }
}

/// S-10：本仓 `src/` 下**每一个** `.rs` 文件均不含 URL 端点字面量与凭据读取入口；依赖为精确白名单。
#[test]
fn assert_no_network_and_no_coupling() {
    // ① 依赖集合必须是精确白名单：既挡住禁用依赖，也不接受任何未声明的依赖。
    let manifest = include_str!("../Cargo.toml");
    let deps = manifest
        .split("[dependencies]")
        .nth(1)
        .expect("Cargo.toml 须含 [dependencies]")
        .split("\n[")
        .next()
        .expect("依赖段存在");
    let mut names: Vec<&str> = deps
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .filter_map(|line| line.split('=').next())
        .map(str::trim)
        .collect();
    names.sort_unstable();
    assert_eq!(names, ["serde", "serde_json", "thiserror"]);
    assert!(!manifest.contains("path = \"../"));

    // ② 运行期递归枚举 `src/` 下全部 `.rs`：以 CARGO_MANIFEST_DIR 为根，**不依赖 cwd**、
    //    不用手写清单，因此新增模块文件（哪怕未接入模块树）也自动进入扫描面。
    let src_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_src_rs(&src_root, &mut files);
    files.sort();
    assert!(
        files.iter().any(|path| path.ends_with("lib.rs")),
        "枚举面须覆盖 src/lib.rs"
    );

    // ③ 覆盖完备性自证：`lib.rs` 里声明的每个 `pub mod <name>;` 都必须出现在枚举面内，
    //    否则「新增了模块文件」可能悄悄逃出扫描。
    let lib_text = std::fs::read_to_string(src_root.join("lib.rs")).expect("读取 src/lib.rs");
    let mut declared: Vec<String> = Vec::new();
    for line in lib_text.lines() {
        if let Some(rest) = line.trim().strip_prefix("pub mod ") {
            if let Some(name) = rest.strip_suffix(';') {
                declared.push(name.to_owned());
            }
        }
    }
    assert!(!declared.is_empty(), "lib.rs 须声明模块");
    for name in &declared {
        let expected = src_root.join(format!("{name}.rs"));
        assert!(
            files.contains(&expected),
            "枚举面漏掉 lib.rs 声明的模块文件：src/{name}.rs"
        );
    }

    // ④ 逐个文件断言：本仓 `src/` 下每一个 `.rs` 文件均不含 http/https 端点字面量，
    //    也不含环境变量 / 凭据读取入口。
    for path in &files {
        let text = std::fs::read_to_string(path).expect("读取 src 下 .rs 文件");
        for needle in ["http://", "https://"] {
            assert!(
                !text.contains(needle),
                "{} 含 URL 端点字面量 `{needle}`",
                path.display()
            );
        }
        for needle in ["env::var", "from_env"] {
            assert!(
                !text.contains(needle),
                "{} 含凭据读取入口 `{needle}`",
                path.display()
            );
        }
    }
}

/// S-11：验收命令与门禁面存在。
#[test]
fn assert_acceptance_surface() {
    let contributing = include_str!("../CONTRIBUTING.md");
    for gate in [
        "cargo fmt --all -- --check",
        "cargo clippy --all-targets --all-features -- -D warnings",
        "cargo test --all-features",
        "cargo package --no-verify",
    ] {
        assert!(contributing.contains(gate), "缺门禁命令：{gate}");
    }
    let readme = include_str!("../README.md");
    assert!(readme.contains("production_decision = NO-GO"));
    assert!(readme.contains("## 非目标"));
    assert!(readme.contains("## 门禁"));
}
