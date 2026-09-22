#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable
)]
//! TDD 行为契约（特性 005 · `fredx`）。
//!
//! 入口集合 = 本 crate 全部公开入口（类型 / 方法 / 判定函数 / 守卫 / 解析器 / 源事实常量）。
//! 下表每个入口先在 `/tmp` 变异副本上观测应红、再在本树观测绿；每条变异与红绿结果
//! 由 `scripts` 侧的等价脚本逐条实跑（见 PR 描述），红==绿是标准形态
//! （同一用例在变异副本上失败、在原树上通过）。
//!
//! // TDD-PROBE: FredErrorKind | 变异：把 `RoutedElsewhere` 的分类改映射为 `Invalid` | 红=error_kind_mapping_and_retryable | 绿=error_kind_mapping_and_retryable
//! // TDD-PROBE: FredError | 变异：`Invalid` 变体的 Display 模板改为 `{0}`（丢中文前缀）| 红=error_display_is_chinese_and_does_not_echo_input | 绿=error_display_is_chinese_and_does_not_echo_input
//! // TDD-PROBE: FredError::kind | 变异：`Missing` 与 `Invalid` 分类互换 | 红=error_kind_mapping_and_retryable | 绿=error_kind_mapping_and_retryable
//! // TDD-PROBE: FredError::is_retryable | 变异：改为对全部变体返回 `true` | 红=error_kind_mapping_and_retryable | 绿=error_kind_mapping_and_retryable
//! // TDD-PROBE: FredResult | 变异：`validate_observation` 丢弃 `validate_period` 的错误 | 红=observation_validation_rules | 绿=observation_validation_rules
//! // TDD-PROBE: Date | 变异：构造时把 `month` 与 `day` 写反 | 红=date_construction_and_calendar_rules | 绿=date_construction_and_calendar_rules
//! // TDD-PROBE: Date::new | 变异：年份下界校验被删除（`Date::new(0,1,1)` 通过） | 红=date_construction_and_calendar_rules | 绿=date_construction_and_calendar_rules
//! // TDD-PROBE: Date::parse | 变异：长度校验放宽为 `< 10`（接受带时间部分者） | 红=date_construction_and_calendar_rules | 绿=date_construction_and_calendar_rules
//! // TDD-PROBE: Date::is_leap_year | 变异：删除 `% 100` 项（1900 被判为闰年） | 红=date_construction_and_calendar_rules | 绿=date_construction_and_calendar_rules
//! // TDD-PROBE: Date::days_in_month | 变异：非法月返回 `31`（`_ => 0` 改为 `31`） | 红=date_construction_and_calendar_rules | 绿=date_construction_and_calendar_rules
//! // TDD-PROBE: validate_date | 变异：日范围校验改为 `day > 31` 才拒绝 | 红=date_construction_and_calendar_rules | 绿=date_construction_and_calendar_rules
//! // TDD-PROBE: Period | 变异：`Quarter` 的季范围上界由 `4` 改为 `5` | 红=period_validation_covers_every_variant | 绿=period_validation_covers_every_variant
//! // TDD-PROBE: validate_period | 变异：`Year(0)` 被接受 | 红=period_validation_covers_every_variant | 绿=period_validation_covers_every_variant
//! // TDD-PROBE: Frequency | 变异：`monthly` 记号映射到 `Quarterly` | 红=frequency_token_and_label_rules | 绿=frequency_token_and_label_rules
//! // TDD-PROBE: Frequency::parse | 变异：未知记号放行（`_` 分支返回 `Ok(Daily)`） | 红=frequency_token_and_label_rules | 绿=frequency_token_and_label_rules
//! // TDD-PROBE: Frequency::as_str | 变异：`Weekly` 记号改为 `week` | 红=frequency_token_and_label_rules | 绿=frequency_token_and_label_rules
//! // TDD-PROBE: FredUnit | 变异：去掉控制字符校验 | 红=unit_constructor_rules | 绿=unit_constructor_rules
//! // TDD-PROBE: FredUnit::new | 变异：允许空串与首尾空白 | 红=unit_constructor_rules | 绿=unit_constructor_rules
//! // TDD-PROBE: FredUnit::as_str | 变异：恒返回固定串 | 红=unit_constructor_rules | 绿=unit_constructor_rules
//! // TDD-PROBE: UNIT_MILLIONS_OF_USD | 变异：字面量改为 `Billions of U.S. Dollars` | 红=unit_constructor_rules | 绿=unit_constructor_rules
//! // TDD-PROBE: UNIT_BILLIONS_OF_USD | 变异：字面量改为 `Millions of U.S. Dollars` | 红=unit_constructor_rules | 绿=unit_constructor_rules
//! // TDD-PROBE: FredMissingReason | 变异：`.` 占位被解析成 `Present(0.0)` | 红=parse_keeps_missing_named | 绿=parse_keeps_missing_named
//! // TDD-PROBE: FredValue | 变异：缺失被折算为 0（`as_f64` 返回 `Some(0.0)`） | 红=value_missing_is_named_never_zero | 绿=value_missing_is_named_never_zero
//! // TDD-PROBE: FredValue::as_f64 | 变异：缺失时返回 `Some(0.0)` | 红=value_missing_is_named_never_zero | 绿=value_missing_is_named_never_zero
//! // TDD-PROBE: FredValue::is_missing | 变异：语义反转（`Present` 返回 `true`） | 红=value_missing_is_named_never_zero | 绿=value_missing_is_named_never_zero
//! // TDD-PROBE: FredObservation | 变异：`vintage` 恒被填为固定日期 | 红=parse_preserves_vintage | 绿=parse_preserves_vintage
//! // TDD-PROBE: validate_observation | 变异：删除有限数校验（NaN 通过） | 红=observation_validation_rules | 绿=observation_validation_rules
//! // TDD-PROBE: parse_fred_observations | 变异：跳过 `ensure_source_unit` 调用 | 红=parse_rejects_unit_drift | 绿=parse_rejects_unit_drift
//! // TDD-PROBE: DFII10（常量） | 变异：字面量改为 `DFII10_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: T10YIE（常量） | 变异：字面量改为 `T10YIE_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: BAMLH0A0HYM2（常量） | 变异：字面量改为 `BAMLH0A0HYM2_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: T10Y2Y（常量） | 变异：字面量改为 `T10Y2Y_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: ICSA（常量） | 变异：字面量改为 `ICSA_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: WALCL（常量） | 变异：字面量改为 `WALCL_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: WTREGEN（常量） | 变异：字面量改为 `WTREGEN_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: RRPONTSYD（常量） | 变异：字面量改为 `RRPONTSYD_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: SOFR（常量） | 变异：字面量改为 `SOFR_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: DFF（常量） | 变异：字面量改为 `DFF_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: VIXCLS（常量） | 变异：字面量改为 `VIXCLS_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: SP500（常量） | 变异：字面量改为 `SP500_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: DCOILWTICO（常量） | 变异：字面量改为 `DCOILWTICO_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: DGS2（常量） | 变异：字面量改为 `DGS2_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: DGS10（常量） | 变异：字面量改为 `DGS10_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: T5YIFR（常量） | 变异：字面量改为 `T5YIFR_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: TEDRATE（常量） | 变异：字面量改为 `TEDRATE_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: PCEPILFE（常量） | 变异：字面量改为 `PCEPILFE_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: UNRATE（常量） | 变异：字面量改为 `UNRATE_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: PAYEMS（常量） | 变异：字面量改为 `PAYEMS_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: CES0500000003（常量） | 变异：字面量改为 `CES0500000003_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: DEXJPUS（常量） | 变异：字面量改为 `DEXJPUS_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: IRLTLT01JPM156N（常量） | 变异：字面量改为 `IRLTLT01JPM156N_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: JPNCPIALLMINMEI（常量） | 变异：字面量改为 `JPNCPIALLMINMEI_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: DEXUSEU（常量） | 变异：字面量改为 `DEXUSEU_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: CPHPTT01EZM659N（常量） | 变异：字面量改为 `CPHPTT01EZM659N_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: FYFSGDA188S（常量） | 变异：字面量改为 `FYFSGDA188S_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: FDHBFRBN（常量） | 变异：字面量改为 `FDHBFRBN_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: BOGZ1FL662090005Q（常量） | 变异：字面量改为 `BOGZ1FL662090005Q_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: NASDAQCOM（常量） | 变异：字面量改为 `NASDAQCOM_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: NIKKEI225（常量） | 变异：字面量改为 `NIKKEI225_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: PCOPPUSDM（常量） | 变异：字面量改为 `PCOPPUSDM_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: DEXCHUS（常量） | 变异：字面量改为 `DEXCHUS_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: CPILFESL（常量） | 变异：字面量改为 `CPILFESL_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: JTSJOL（常量） | 变异：字面量改为 `JTSJOL_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: WRESBAL（常量） | 变异：字面量改为 `WRESBAL_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: BAMLC0A0CM（常量） | 变异：字面量改为 `BAMLC0A0CM_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: RESPPLLOPNWW（常量） | 变异：字面量改为 `RESPPLLOPNWW_Z` | 红=series_id_literals_match_the_manifest | 绿=series_id_literals_match_the_manifest
//! // TDD-PROBE: ALL_SERIES | 变异：从全集中删除 `CPILFESL` | 红=series_scope_predicates | 绿=series_scope_predicates
//! // TDD-PROBE: P0_SERIES | 变异：从 P0 组删除 `DFII10` | 红=series_scope_predicates | 绿=series_scope_predicates
//! // TDD-PROBE: CURVE_SERIES | 变异：把 `WALCL` 加入曲线组 | 红=series_scope_predicates | 绿=series_scope_predicates
//! // TDD-PROBE: FUNDAMENTAL_SERIES | 变异：从基本面组删除 `PCEPILFE` | 红=series_scope_predicates | 绿=series_scope_predicates
//! // TDD-PROBE: GLOBAL_REGIONAL_SERIES | 变异：把 `WALCL` 加入全球区域组 | 红=series_scope_predicates | 绿=series_scope_predicates
//! // TDD-PROBE: BLS_FORWARD_SERIES | 变异：从 BLS 转发组删除 `JTSJOL` | 红=series_scope_predicates | 绿=series_scope_predicates
//! // TDD-PROBE: AUTHORITATIVE_WRITE_SERIES | 变异：把 `SP500` 加入权威写入集合 | 红=authoritative_write_and_denials | 绿=authoritative_write_and_denials
//! // TDD-PROBE: BAML_ALLOWED_SERIES | 变异：把 `BAMLC0A0CM` 加入允许集合 | 红=parse_rejects_baml_other_series | 绿=parse_rejects_baml_other_series
//! // TDD-PROBE: is_collected_series | 变异：改为前缀匹配（`WALCLX` 被判为采集序列） | 红=series_scope_predicates | 绿=series_scope_predicates
//! // TDD-PROBE: is_mapping_only_series | 变异：改为返回 `true` 对任意 ID | 红=series_scope_predicates | 绿=series_scope_predicates
//! // TDD-PROBE: is_known_series | 变异：把 `BAMLC0A0CM`（已 drop）判为已登记 | 红=series_scope_predicates | 绿=series_scope_predicates
//! // TDD-PROBE: is_curve_input_series | 变异：改为仅匹配 `DGS` 前缀（`T10Y2Y` 漏判） | 红=curve_input_prefix_rules | 绿=curve_input_prefix_rules
//! // TDD-PROBE: series_unit | 变异：把 `WALCL` 的源单位改为 Billions | 红=parse_rejects_unit_drift | 绿=parse_rejects_unit_drift
//! // TDD-PROBE: series_frequency | 变异：把 `WALCL` 的清单频率改为 `daily` | 红=parse_rejects_frequency_drift | 绿=parse_rejects_frequency_drift
//! // TDD-PROBE: is_unique_write_authority | 变异：改为对全部 ID 返回 `true` | 红=authoritative_write_and_denials | 绿=authoritative_write_and_denials
//! // TDD-PROBE: ensure_known_series | 变异：改为对未登记 ID 返回 `Ok(())` | 红=parse_rejects_out_of_scope_series | 绿=parse_rejects_out_of_scope_series
//! // TDD-PROBE: ensure_source_unit | 变异：改为对不符单位返回 `Ok(())` | 红=parse_rejects_unit_drift | 绿=parse_rejects_unit_drift
//! // TDD-PROBE: ensure_source_frequency | 变异：改为对不符频率返回 `Ok(())` | 红=parse_rejects_frequency_drift | 绿=parse_rejects_frequency_drift
//! // TDD-PROBE: NEAR_SYNONYM_PAIRS | 变异：删除 `(DFF, EFFR)` 一对 | 红=near_synonym_pairs_are_enforced | 绿=near_synonym_pairs_are_enforced
//! // TDD-PROBE: is_near_synonym_pair | 变异：改为单向匹配（仅左→右） | 红=near_synonym_pairs_are_enforced | 绿=near_synonym_pairs_are_enforced
//! // TDD-PROBE: ensure_not_silent_substitution | 变异：命中后返回 `Ok(())` | 红=near_synonym_pairs_are_enforced | 绿=near_synonym_pairs_are_enforced
//! // TDD-PROBE: ensure_baml_freeze | 变异：对全部 `BAML*` 放行 | 红=parse_rejects_baml_other_series | 绿=parse_rejects_baml_other_series
//! // TDD-PROBE: is_bea_table_id | 变异：改为只看首字母 `T`（`T10Y2Y` 被判为表号） | 红=bea_table_id_shape | 绿=bea_table_id_shape
//! // TDD-PROBE: BEA_TABLE_OWNER | 变异：归属方改为 `fredx` | 红=bea_table_id_shape | 绿=bea_table_id_shape
//! // TDD-PROBE: CURVE_BUILD_OWNER | 变异：归属方改为 `fredx` | 红=curve_products_are_routed_not_local | 绿=curve_products_are_routed_not_local
//! // TDD-PROBE: FredProduct | 变异：`YieldCurve` 变体改为与 `Observation` 等价 | 红=curve_products_are_routed_not_local | 绿=curve_products_are_routed_not_local
//! // TDD-PROBE: ensure_product_local | 变异：对 `YieldCurve` 返回 `Ok(())` | 红=curve_products_are_routed_not_local | 绿=curve_products_are_routed_not_local
//! // TDD-PROBE: claim_authoritative_write | 变异：对曲线点返回 `Ok(())` | 红=authoritative_write_and_denials | 绿=authoritative_write_and_denials
//! // TDD-PROBE: TimePrecision | 变异：`Date` 变体与 `Instant` 对调 | 红=publication_semantics_triple | 绿=publication_semantics_triple
//! // TDD-PROBE: AvailabilityEvidence | 变异：`Inferred` 与 `Official` 对调 | 红=publication_semantics_triple | 绿=publication_semantics_triple
//! // TDD-PROBE: PitEligibility | 变异：`NotEligible` 与 `Formal` 对调 | 红=publication_semantics_triple | 绿=publication_semantics_triple
//! // TDD-PROBE: FredPublicationSemantics | 变异：构造时把 evidence 填为 `Official` | 红=publication_semantics_triple | 绿=publication_semantics_triple
//! // TDD-PROBE: fred_publication_semantics | 变异：返回 `PitEligibility::Formal` | 红=publication_semantics_triple | 绿=publication_semantics_triple
//! // TDD-PROBE: is_formal_pit_eligible | 变异：改为返回 `true` | 红=formal_pit_is_never_eligible | 绿=formal_pit_is_never_eligible
//! // TDD-PROBE: FredAccessMode | 变异：`Offline` 与 `Live` 对调 | 红=authz_documented_evidence_is_offline_only | 绿=authz_documented_evidence_is_offline_only
//! // TDD-PROBE: FredAuthorization | 变异：证据缺失分支改为返回 `Authorized` | 红=authz_fail_closed_paths | 绿=authz_fail_closed_paths
//! // TDD-PROBE: FredAuthorizationEvidence | 变异：证据的 `signed_by` 被清空 | 红=authz_documented_evidence_is_offline_only | 绿=authz_documented_evidence_is_offline_only
//! // TDD-PROBE: FRED_DECISION_ID | 变异：编号改为 `FRED-PROD-2026-07-28-reject`（已被替换的旧值） | 红=authz_documented_evidence_is_offline_only | 绿=authz_documented_evidence_is_offline_only
//! // TDD-PROBE: FRED_SIGNED_BY | 变异：签署者改为空串 | 红=authz_documented_evidence_is_offline_only | 绿=authz_documented_evidence_is_offline_only
//! // TDD-PROBE: documented_fred_evidence | 变异：`authorized_modes` 填为 `[Offline, Live]` | 红=authz_documented_evidence_is_offline_only | 绿=authz_documented_evidence_is_offline_only
//! // TDD-PROBE: authorize_fred | 变异：`Live` 未被覆盖的判定失效（`contains` 检查删除） | 红=authz_documented_evidence_is_offline_only | 绿=authz_documented_evidence_is_offline_only
//! // TDD-PROBE: validate_authorization_evidence | 变异：跳过 `valid_until` 的日期校验 | 红=authz_evidence_date_validation | 绿=authz_evidence_date_validation
//! // TDD-PROBE: mode_label | 变异：`Offline` 的记号改为 `live` | 红=mode_labels_are_stable | 绿=mode_labels_are_stable

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
    AUTHORITATIVE_WRITE_SERIES, BAML_ALLOWED_SERIES, BEA_TABLE_OWNER, BLS_FORWARD_SERIES,
    CURVE_BUILD_OWNER, CURVE_SERIES, FRED_DECISION_ID, FRED_SIGNED_BY, FUNDAMENTAL_SERIES,
    GLOBAL_REGIONAL_SERIES, NEAR_SYNONYM_PAIRS, P0_SERIES, UNIT_BILLIONS_OF_USD,
    UNIT_MILLIONS_OF_USD,
};

const FIXTURE: &str = include_str!("fixtures/fred_observations_synthetic.json");
const MISSING_FIXTURE: &str = include_str!("fixtures/fred_missing_value_synthetic.json");

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

fn record(series_id: &str, date: &str, value: &str, unit: &str, frequency: &str) -> String {
    format!(
        r#"{{"records": [{{"series_id": "{series_id}", "date": "{date}", "value": {value},
            "unit": "{unit}", "frequency": "{frequency}"}}]}}"#
    )
}

/// 35 个 series ID 常量逐字等于清单登记值（源事实防漂），且与分组集一致。
#[test]
fn series_id_literals_match_the_manifest() {
    let expected: &[(&str, &str)] = &[
        (fredx::DFII10, "DFII10"),
        (fredx::T10YIE, "T10YIE"),
        (fredx::BAMLH0A0HYM2, "BAMLH0A0HYM2"),
        (fredx::T10Y2Y, "T10Y2Y"),
        (fredx::ICSA, "ICSA"),
        (fredx::WALCL, "WALCL"),
        (fredx::WTREGEN, "WTREGEN"),
        (fredx::RRPONTSYD, "RRPONTSYD"),
        (fredx::SOFR, "SOFR"),
        (fredx::DFF, "DFF"),
        (fredx::VIXCLS, "VIXCLS"),
        (fredx::SP500, "SP500"),
        (fredx::DCOILWTICO, "DCOILWTICO"),
        (fredx::DGS2, "DGS2"),
        (fredx::DGS10, "DGS10"),
        (fredx::T5YIFR, "T5YIFR"),
        (fredx::TEDRATE, "TEDRATE"),
        (fredx::PCEPILFE, "PCEPILFE"),
        (fredx::UNRATE, "UNRATE"),
        (fredx::PAYEMS, "PAYEMS"),
        (fredx::CES0500000003, "CES0500000003"),
        (fredx::DEXJPUS, "DEXJPUS"),
        (fredx::IRLTLT01JPM156N, "IRLTLT01JPM156N"),
        (fredx::JPNCPIALLMINMEI, "JPNCPIALLMINMEI"),
        (fredx::DEXUSEU, "DEXUSEU"),
        (fredx::CPHPTT01EZM659N, "CPHPTT01EZM659N"),
        (fredx::FYFSGDA188S, "FYFSGDA188S"),
        (fredx::FDHBFRBN, "FDHBFRBN"),
        (fredx::BOGZ1FL662090005Q, "BOGZ1FL662090005Q"),
        (fredx::NASDAQCOM, "NASDAQCOM"),
        (fredx::NIKKEI225, "NIKKEI225"),
        (fredx::PCOPPUSDM, "PCOPPUSDM"),
        (fredx::DEXCHUS, "DEXCHUS"),
        (fredx::CPILFESL, "CPILFESL"),
        (fredx::JTSJOL, "JTSJOL"),
    ];
    assert_eq!(expected.len(), 35);
    for (actual, manifest) in expected {
        assert_eq!(*actual, *manifest);
    }
    assert_eq!(fredx::WRESBAL, "WRESBAL");
    assert_eq!(fredx::BAMLC0A0CM, "BAMLC0A0CM");
    assert_eq!(fredx::RESPPLLOPNWW, "RESPPLLOPNWW");
    for (id, _) in expected {
        assert!(ALL_SERIES.contains(id), "全集缺 {id}");
    }
    assert_eq!(ALL_SERIES.len(), expected.len());
}

/// Date / validate_date：严格 ISO 形态、月日范围、闰年。
#[test]
fn date_construction_and_calendar_rules() {
    assert_eq!(
        Date::parse("2026-08-14").expect("应可解析"),
        Date::new(2026, 8, 14).expect("应可构造")
    );
    for bad in [
        "2026-2-3",
        "2026/02/03",
        "2026-02-30",
        "2026-13-01",
        "10000-01-01",
        "2026-08-14T00:00:00Z",
    ] {
        assert!(Date::parse(bad).is_err(), "应拒绝 {bad}");
    }
    assert!(Date::new(0, 1, 1).is_err());
    assert!(Date::is_leap_year(2024));
    assert!(!Date::is_leap_year(1900));
    assert!(Date::is_leap_year(2000));
    assert_eq!(Date::days_in_month(2024, 2), 29);
    assert_eq!(Date::days_in_month(2025, 2), 28);
    assert_eq!(Date::days_in_month(2026, 13), 0);
    assert!(validate_date(&Date {
        year: 2026,
        month: 4,
        day: 31
    })
    .is_err());
}

/// Period / validate_period：五个变体的分量范围。
#[test]
fn period_validation_covers_every_variant() {
    let day = Date::new(2026, 8, 14).expect("日期合法");
    assert!(validate_period(&Period::Day(day)).is_ok());
    assert!(validate_period(&Period::Event { date: day }).is_ok());
    assert!(validate_period(&Period::Month {
        year: 2026,
        month: 12
    })
    .is_ok());
    assert!(validate_period(&Period::Month {
        year: 2026,
        month: 13
    })
    .is_err());
    assert!(validate_period(&Period::Quarter {
        year: 2026,
        quarter: 4
    })
    .is_ok());
    assert!(validate_period(&Period::Quarter {
        year: 2026,
        quarter: 5
    })
    .is_err());
    assert!(validate_period(&Period::Year(2026)).is_ok());
    assert!(validate_period(&Period::Year(0)).is_err());
}

/// Frequency：七值记号回环，且只接受小写。
#[test]
fn frequency_token_and_label_rules() {
    let all = [
        (Frequency::Daily, "daily"),
        (Frequency::Weekly, "weekly"),
        (Frequency::Monthly, "monthly"),
        (Frequency::Quarterly, "quarterly"),
        (Frequency::Annual, "annual"),
        (Frequency::Event, "event"),
        (Frequency::Irregular, "irregular"),
    ];
    for (frequency, token) in all {
        assert_eq!(frequency.as_str(), token);
        assert_eq!(Frequency::parse(token).expect("应可解析"), frequency);
    }
    assert!(Frequency::parse("Monthly").is_err());
    assert!(Frequency::parse("week").is_err());
}

/// FredUnit / UNIT_*：开放 newtype 的构造与字面量。
#[test]
fn unit_constructor_rules() {
    assert_eq!(UNIT_MILLIONS_OF_USD, "Millions of U.S. Dollars");
    assert_eq!(UNIT_BILLIONS_OF_USD, "Billions of U.S. Dollars");
    assert_ne!(UNIT_MILLIONS_OF_USD, UNIT_BILLIONS_OF_USD);
    assert_eq!(
        FredUnit::new(UNIT_BILLIONS_OF_USD)
            .expect("应可构造")
            .as_str(),
        UNIT_BILLIONS_OF_USD
    );
    assert!(FredUnit::new("").is_err());
    assert!(FredUnit::new(" Percent").is_err());
    assert!(FredUnit::new("Percent\n").is_err());
    // 内部（非首尾）控制字符也必须被拒绝：它不会被 trim 捕获。
    assert!(FredUnit::new("Per\u{7}cent").is_err());
    assert!(FredUnit::new("Per\u{0}cent").is_err());
}

/// FredValue / FredMissingReason：缺失具名且绝不折算为 0。
#[test]
fn value_missing_is_named_never_zero() {
    let missing = FredValue::Missing(FredMissingReason::NoObservation);
    assert!(missing.is_missing());
    assert_eq!(missing.as_f64(), None);
    assert_ne!(missing, FredValue::Present(0.0));
    let present = FredValue::Present(0.0);
    assert!(!present.is_missing());
    assert_eq!(present.as_f64(), Some(0.0));
}

/// FredObservation / validate_observation：标识、期间、有限数、修订日期。
#[test]
fn observation_validation_rules() {
    let good = observation("WALCL", UNIT_MILLIONS_OF_USD, Frequency::Weekly);
    assert!(validate_observation(&good).is_ok());

    let mut bad = good.clone();
    bad.series_id = " WALCL".to_owned();
    assert!(validate_observation(&bad).is_err());

    bad = good.clone();
    bad.value = FredValue::Present(f64::NAN);
    assert!(validate_observation(&bad).is_err());

    bad = good.clone();
    bad.value = FredValue::Present(f64::INFINITY);
    assert!(validate_observation(&bad).is_err());

    bad = good.clone();
    bad.period = Period::Year(0);
    assert!(validate_observation(&bad).is_err());

    bad = good.clone();
    bad.vintage = Some(Date {
        year: 2026,
        month: 2,
        day: 30,
    });
    assert!(validate_observation(&bad).is_err());
}

/// parse_fred_observations：合成夹具整体可解析。
#[test]
fn parse_accepts_synthetic_fixture() {
    let observations = parse_fred_observations(FIXTURE).expect("应可解析");
    assert_eq!(observations.len(), 10);
    assert!(observations.iter().any(|o| o.series_id == fredx::RRPONTSYD));
    assert!(observations.iter().all(|o| validate_observation(o).is_ok()));
}

/// 期间投影：月 / 季 / 年 / 周 按清单频率落到对应 `Period`。
#[test]
fn parse_projects_period_by_frequency() {
    let input = format!(
        r#"{{"records": [
            {},
            {},
            {},
            {}
        ]}}"#,
        r#"{"series_id": "PCEPILFE", "date": "2026-06-30", "value": 128.4, "unit": "Index", "frequency": "monthly"}"#,
        r#"{"series_id": "FYFSGDA188S", "date": "2026-06-30", "value": -6.5, "unit": "Percent of GDP", "frequency": "quarterly"}"#,
        r#"{"series_id": "BOGZ1FL662090005Q", "date": "2026-03-31", "value": 1.0, "unit": "Index", "frequency": "quarterly"}"#,
        r#"{"series_id": "ICSA", "date": "2026-08-08", "value": 221000.0, "unit": "Persons", "frequency": "weekly"}"#
    );
    let observations = parse_fred_observations(&input).expect("应可解析");
    assert_eq!(
        observations[0].period,
        Period::Month {
            year: 2026,
            month: 6
        }
    );
    assert_eq!(
        observations[1].period,
        Period::Quarter {
            year: 2026,
            quarter: 2
        }
    );
    assert_eq!(
        observations[2].period,
        Period::Quarter {
            year: 2026,
            quarter: 1
        }
    );
    assert!(matches!(observations[3].period, Period::Day(_)));
}

/// 修订标识：给出时保留，未给出时为 `None`（不伪造）。
#[test]
fn parse_preserves_vintage() {
    let with_vintage = parse_fred_observations(FIXTURE).expect("应可解析");
    let vintage = with_vintage
        .iter()
        .find(|o| o.series_id == fredx::UNRATE)
        .expect("夹具含 UNRATE");
    assert_eq!(
        vintage.vintage,
        Some(Date::new(2026, 7, 3).expect("日期合法"))
    );
    assert!(with_vintage
        .iter()
        .filter(|o| o.series_id != fredx::UNRATE)
        .all(|o| o.vintage.is_none()));
}

/// 未知字段（记录层与顶层）原子失败。
#[test]
fn parse_rejects_unknown_field() {
    assert!(parse_fred_observations(
        r#"{"records": [{"series_id": "SOFR", "date": "2026-08-14", "value": 1.0,
            "unit": "Percent", "frequency": "daily", "endpoint": "x"}]}"#
    )
    .is_err());
    assert!(parse_fred_observations(r#"{"records": [], "token": "x"}"#).is_err());
}

/// 缺必需字段原子失败。
#[test]
fn parse_rejects_missing_field() {
    assert!(parse_fred_observations(
        r#"{"records": [{"date": "2026-08-14", "value": 1.0, "unit": "Percent",
            "frequency": "daily"}]}"#
    )
    .is_err());
    assert!(parse_fred_observations(r#"{"items": []}"#).is_err());
}

/// 非法日期形态原子失败。
#[test]
fn parse_rejects_illegal_date() {
    for bad in [
        "2026-2-3",
        "2026/02/03",
        "2026-02-30",
        "2026-08-14T00:00:00Z",
    ] {
        let input = record("SOFR", bad, "1.0", "Percent", "daily");
        assert!(parse_fred_observations(&input).is_err(), "{bad}");
    }
}

/// 重复身份被拒绝（选择「拒绝」而非去重）。
#[test]
fn parse_rejects_duplicate_identity() {
    let input = r#"{"records": [
        {"series_id": "SOFR", "date": "2026-08-14", "value": 1.0, "unit": "Percent", "frequency": "daily"},
        {"series_id": "SOFR", "date": "2026-08-14", "value": 1.1, "unit": "Percent", "frequency": "daily"}
    ]}"#;
    let err = parse_fred_observations(input).expect_err("应拒绝");
    assert_eq!(err.kind(), FredErrorKind::SemanticallyRejected);
}

/// `.` 与 null 两种缺失形态都映射为具名缺失。
#[test]
fn parse_keeps_missing_named() {
    let observations = parse_fred_observations(MISSING_FIXTURE).expect("应可解析");
    assert_eq!(observations.len(), 2);
    for record in &observations {
        assert_eq!(
            record.value,
            FredValue::Missing(FredMissingReason::NoObservation)
        );
        assert_eq!(record.value.as_f64(), None);
    }
    assert!(
        parse_fred_observations(&record("SOFR", "2026-08-14", "0.0", "Percent", "daily")).is_ok()
    );
}

/// 清单范围外 series ID 被拒绝。
#[test]
fn parse_rejects_out_of_scope_series() {
    assert!(ensure_known_series("WALCL").is_ok());
    assert!(ensure_known_series("WRESBAL").is_ok());
    for out_of_scope in ["CPIAUCSL", "BAMLC0A0CM", "RESPPLLOPNWW", "GDPNow"] {
        assert!(ensure_known_series(out_of_scope).is_err(), "{out_of_scope}");
    }
    assert!(
        parse_fred_observations(&record("CPIAUCSL", "2026-06-30", "1.0", "Index", "monthly"))
            .is_err()
    );
}

/// 源单位：`RRPONTSYD` 为 Billions，标成 Millions 必须被拒且值不被改写。
#[test]
fn parse_rejects_unit_drift() {
    assert_eq!(series_unit("WALCL"), Some(UNIT_MILLIONS_OF_USD));
    assert_eq!(series_unit("RRPONTSYD"), Some(UNIT_BILLIONS_OF_USD));
    assert_eq!(series_unit("SP500"), None);

    let wrong = observation("RRPONTSYD", UNIT_MILLIONS_OF_USD, Frequency::Daily);
    assert_eq!(
        ensure_source_unit(&wrong).expect_err("应拒绝").kind(),
        FredErrorKind::SemanticallyRejected
    );
    assert_eq!(wrong.value.as_f64(), Some(1.0), "拒绝时不得换算");

    let right = observation("RRPONTSYD", UNIT_BILLIONS_OF_USD, Frequency::Daily);
    assert!(ensure_source_unit(&right).is_ok());
    assert!(parse_fred_observations(&record(
        "RRPONTSYD",
        "2026-08-14",
        "450.25",
        UNIT_MILLIONS_OF_USD,
        "daily"
    ))
    .is_err());
}

/// 频率：清单声明为周频的序列标成 daily 必须被拒。
#[test]
fn parse_rejects_frequency_drift() {
    assert_eq!(series_frequency("ICSA"), Some(Frequency::Weekly));
    assert_eq!(series_frequency("FYFSGDA188S"), Some(Frequency::Quarterly));
    let wrong = observation("ICSA", "Persons", Frequency::Daily);
    assert!(ensure_source_frequency(&wrong).is_err());
    assert!(parse_fred_observations(&record(
        "ICSA",
        "2026-08-08",
        "221000.0",
        "Persons",
        "daily"
    ))
    .is_err());
    assert!(parse_fred_observations(&record(
        "ICSA",
        "2026-08-08",
        "221000.0",
        "Persons",
        "weekly"
    ))
    .is_ok());
}

/// BAML 冻结令：只允许 `BAMLH0A0HYM2`。
#[test]
fn parse_rejects_baml_other_series() {
    assert_eq!(BAML_ALLOWED_SERIES.len(), 1);
    assert_eq!(BAML_ALLOWED_SERIES[0], fredx::BAMLH0A0HYM2);
    assert!(ensure_baml_freeze("BAMLH0A0HYM2").is_ok());
    assert!(ensure_baml_freeze("WALCL").is_ok());
    assert!(ensure_baml_freeze("BAMLC0A0CM").is_err());
    assert!(ensure_baml_freeze("BAMLH0A3HYC").is_err());
    assert!(parse_fred_observations(&record(
        "BAMLH0A0HYM2",
        "2026-08-14",
        "3.12",
        "Percent",
        "daily"
    ))
    .is_ok());
    assert!(parse_fred_observations(&record(
        "BAMLH0A0CM",
        "2026-08-14",
        "1.0",
        "Percent",
        "daily"
    ))
    .is_err());
}

/// 值形态白名单：只接受数值、`.`、null。
#[test]
fn parse_rejects_non_numeric_value() {
    for bad in ["\"n/a\"", "true", "[1]", "{}"] {
        assert!(
            parse_fred_observations(&record("SOFR", "2026-08-14", bad, "Percent", "daily"))
                .is_err(),
            "{bad}"
        );
    }
}

/// 范围判定谓词与分组集合的内部一致性。
#[test]
fn series_scope_predicates() {
    assert_eq!(ALL_SERIES.len(), 35);
    assert_eq!(
        P0_SERIES.len()
            + CURVE_SERIES.len()
            + FUNDAMENTAL_SERIES.len()
            + GLOBAL_REGIONAL_SERIES.len()
            + BLS_FORWARD_SERIES.len(),
        ALL_SERIES.len()
    );
    assert_eq!(P0_SERIES.len(), 13);
    assert_eq!(CURVE_SERIES.len(), 4);
    assert_eq!(FUNDAMENTAL_SERIES.len(), 4);
    assert_eq!(GLOBAL_REGIONAL_SERIES.len(), 12);
    assert_eq!(BLS_FORWARD_SERIES.len(), 2);

    assert!(is_collected_series("WALCL"));
    assert!(!is_collected_series("WALCLX"));
    assert!(is_mapping_only_series("WRESBAL"));
    assert!(!is_mapping_only_series("WALCL"));
    assert!(is_known_series("WRESBAL"));
    assert!(!is_known_series("BAMLC0A0CM"));
}

/// 曲线输入点按 `DGS*` / `T10Y*` 前缀判定。
#[test]
fn curve_input_prefix_rules() {
    for id in ["DGS2", "DGS10", "T10Y2Y", "T10YIE"] {
        assert!(is_curve_input_series(id), "{id}");
    }
    for id in ["WALCL", "T5YIFR", "TEDRATE", "BAMLH0A0HYM2"] {
        assert!(!is_curve_input_series(id), "{id}");
    }
}

/// 权威写入主权与越权拒绝。
#[test]
fn authoritative_write_and_denials() {
    assert_eq!(AUTHORITATIVE_WRITE_SERIES.len(), 4);
    for id in ["WALCL", "WTREGEN", "RRPONTSYD", "WRESBAL"] {
        assert!(is_unique_write_authority(id));
        assert!(claim_authoritative_write(id).is_ok());
    }
    assert!(!is_unique_write_authority("SP500"));
    for denied in ["DGS10", "T20100", "RESPPLLOPNWW", "SP500"] {
        assert_eq!(
            claim_authoritative_write(denied)
                .expect_err("应拒绝")
                .kind(),
            FredErrorKind::WriteAuthorityDenied,
            "{denied}"
        );
    }
    assert!(claim_authoritative_write("DGS10")
        .expect_err("曲线点")
        .to_string()
        .contains(CURVE_BUILD_OWNER));
    assert!(claim_authoritative_write("T20100")
        .expect_err("BEA 表号")
        .to_string()
        .contains(BEA_TABLE_OWNER));
}

/// BEA 表号形态与归属方。
#[test]
fn bea_table_id_shape() {
    assert!(is_bea_table_id("T10101"));
    assert!(is_bea_table_id("T20100"));
    assert!(!is_bea_table_id("T10Y2Y"));
    assert!(!is_bea_table_id("t10101"));
    assert!(!is_bea_table_id("NIPA"));
    assert_eq!(BEA_TABLE_OWNER, "beax");
}

/// 14 对近义非同 ID 双向禁止。
#[test]
fn near_synonym_pairs_are_enforced() {
    assert_eq!(NEAR_SYNONYM_PAIRS.len(), 14);
    for (left, right) in NEAR_SYNONYM_PAIRS {
        assert!(is_near_synonym_pair(left, right));
        assert!(is_near_synonym_pair(right, left));
        assert!(ensure_not_silent_substitution(left, right).is_err());
        assert!(ensure_not_silent_substitution(right, left).is_err());
    }
    assert!(ensure_not_silent_substitution("WALCL", "RRPONTSYD").is_ok());
    assert!(is_near_synonym_pair("DFF", "EFFR"));
}

/// 曲线构建不在本域。
#[test]
fn curve_products_are_routed_not_local() {
    assert_eq!(CURVE_BUILD_OWNER, "yieldx");
    assert!(ensure_product_local(FredProduct::Observation).is_ok());
    let err = ensure_product_local(FredProduct::YieldCurve).expect_err("应路由");
    assert_eq!(err.kind(), FredErrorKind::RoutedElsewhere);
    assert!(err.to_string().contains(CURVE_BUILD_OWNER));
}

/// publication 三元组恒为 `(Date, Inferred, NotEligible)`。
#[test]
fn publication_semantics_triple() {
    let semantics: FredPublicationSemantics = fred_publication_semantics();
    assert_eq!(semantics.time_precision, TimePrecision::Date);
    assert_eq!(semantics.availability, AvailabilityEvidence::Inferred);
    assert_eq!(semantics.eligibility, PitEligibility::NotEligible);
    assert_ne!(semantics.time_precision, TimePrecision::Instant);
    assert_ne!(semantics.eligibility, PitEligibility::Formal);
}

/// 正式 PIT 资格恒为 `false`。
#[test]
fn formal_pit_is_never_eligible() {
    assert!(!is_formal_pit_eligible());
    assert_eq!(
        fred_publication_semantics().eligibility,
        PitEligibility::NotEligible
    );
}

/// 已登记证据覆盖 offline、不覆盖 live。
#[test]
fn authz_documented_evidence_is_offline_only() {
    assert_eq!(FRED_DECISION_ID, "FRED-PROD-2026-08-15-approve");
    assert_eq!(FRED_SIGNED_BY, "ZoneCNH");
    let evidence = documented_fred_evidence();
    assert_eq!(evidence.decision_id, FRED_DECISION_ID);
    assert_eq!(evidence.signed_by, FRED_SIGNED_BY);
    assert_eq!(evidence.authorized_modes, vec![FredAccessMode::Offline]);
    let as_of = Date::new(2026, 8, 20).expect("日期合法");
    match authorize_fred(Some(&evidence), FredAccessMode::Offline, as_of) {
        FredAuthorization::Authorized { scope } => {
            assert!(scope.contains(FRED_DECISION_ID));
            assert!(scope.contains("offline"));
        }
        other => panic!("应授权 offline，实得 {other:?}"),
    }
    match authorize_fred(Some(&evidence), FredAccessMode::Live, as_of) {
        FredAuthorization::Denied { reason } => assert!(!reason.is_empty()),
        other => panic!("应拒绝 live，实得 {other:?}"),
    }
}

/// fail-closed：缺失 / 编号不明 / 签署者不明 / 范围不明 / 过期。
#[test]
fn authz_fail_closed_paths() {
    let as_of = Date::new(2026, 8, 20).expect("日期合法");
    assert!(matches!(
        authorize_fred(None, FredAccessMode::Offline, as_of),
        FredAuthorization::Denied { .. }
    ));

    let mut evidence = documented_fred_evidence();
    evidence.decision_id = String::new();
    assert!(matches!(
        authorize_fred(Some(&evidence), FredAccessMode::Offline, as_of),
        FredAuthorization::Denied { .. }
    ));

    let mut evidence = documented_fred_evidence();
    evidence.signed_by = String::new();
    assert!(matches!(
        authorize_fred(Some(&evidence), FredAccessMode::Offline, as_of),
        FredAuthorization::Denied { .. }
    ));

    let mut evidence = documented_fred_evidence();
    evidence.authorized_modes.clear();
    assert!(matches!(
        authorize_fred(Some(&evidence), FredAccessMode::Offline, as_of),
        FredAuthorization::Denied { .. }
    ));

    let mut evidence = documented_fred_evidence();
    evidence.valid_until = Date::new(2026, 8, 16).ok();
    match authorize_fred(Some(&evidence), FredAccessMode::Offline, as_of) {
        FredAuthorization::Denied { reason } => assert_eq!(reason, "证据已过期"),
        other => panic!("应拒绝过期证据，实得 {other:?}"),
    }
}

/// 证据描述的日期分量校验。
#[test]
fn authz_evidence_date_validation() {
    let valid: FredAuthorizationEvidence = documented_fred_evidence();
    assert!(validate_authorization_evidence(&valid).is_ok());

    let mut invalid = documented_fred_evidence();
    invalid.signed_at = Some(Date {
        year: 2026,
        month: 13,
        day: 1,
    });
    assert!(validate_authorization_evidence(&invalid).is_err());

    let mut invalid = documented_fred_evidence();
    invalid.valid_until = Some(Date {
        year: 2026,
        month: 2,
        day: 30,
    });
    assert!(validate_authorization_evidence(&invalid).is_err());
}

/// 访问模式记号稳定。
#[test]
fn mode_labels_are_stable() {
    assert_eq!(mode_label(FredAccessMode::Offline), "offline");
    assert_eq!(mode_label(FredAccessMode::Live), "live");
}

/// 错误分类与重试判定。
#[test]
fn error_kind_mapping_and_retryable() {
    let cases = [
        (FredError::Invalid("x".into()), FredErrorKind::Invalid),
        (FredError::Missing("x".into()), FredErrorKind::Missing),
        (
            FredError::AuthorizationDenied("x".into()),
            FredErrorKind::AuthorizationDenied,
        ),
        (
            FredError::RoutedElsewhere("x".into()),
            FredErrorKind::RoutedElsewhere,
        ),
        (
            FredError::WriteAuthorityDenied("x".into()),
            FredErrorKind::WriteAuthorityDenied,
        ),
        (
            FredError::SemanticallyRejected("x".into()),
            FredErrorKind::SemanticallyRejected,
        ),
        (
            FredError::NotApplicable("x".into()),
            FredErrorKind::NotApplicable,
        ),
        (FredError::Invariant("x".into()), FredErrorKind::Invariant),
    ];
    for (err, kind) in cases {
        assert_eq!(err.kind(), kind);
        assert_eq!(err.is_retryable(), kind == FredErrorKind::Invariant);
    }
    let result: FredResult<()> = ensure_known_series("WALCL");
    assert!(result.is_ok());
    let failed: FredResult<()> = ensure_known_series("CPIAUCSL");
    assert_eq!(
        failed.expect_err("范围外").kind(),
        FredErrorKind::SemanticallyRejected
    );
}

/// 错误消息为中文，且不回声输入内容。
#[test]
fn error_display_is_chinese_and_does_not_echo_input() {
    let err = parse_fred_observations("{ oops").expect_err("非法 JSON");
    let shown = err.to_string();
    assert!(shown.contains("输入非法"));
    assert!(!shown.contains("oops"));

    let err = FredError::RoutedElsewhere("曲线构建归 yieldx".into());
    assert!(err.to_string().contains("已路由他处"));
    assert!(!err.to_string().contains("http"));
}
