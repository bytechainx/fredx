#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable
    )
)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]

//! # fredx —— FRED 源事实库（类型化源事实 + 离线解析 + 守卫 + fail-closed 授权判定）
//!
//! 本库把 `specs/adapter/fred.md` 声明的**源事实**落成代码：series ID 常量与分组、
//! 源侧单位与频率、近义非同 ID 禁则、BAML 冻结令、写入主权、曲线边界、
//! publication 语义三元组，以及一个只吃字符串的离线解析器。
//!
//! ## 能力
//!
//! | 能力 | 状态 |
//! | --- | --- |
//! | series ID 常量与分组（35 个纯 FRED ID + mapping-only 登记） | 已落（离线） |
//! | 源单位 / 频率一致性守卫（拒绝把 Billions 当 Millions） | 已落 |
//! | 权威写入主权判定与越权写入拒绝（`WALCL`/`WTREGEN`/`RRPONTSYD`/`WRESBAL`） | 已落（只读判定） |
//! | 近义非同 ID 守卫（涉 FRED 的 14 对） | 已落 |
//! | BAML 冻结令守卫 | 已落 |
//! | 曲线边界：曲线构建归 `yieldx` | 已落（路由拒绝） |
//! | publication 语义（`Date` + `Inferred` + 正式 PIT `NotEligible`） | 已落 |
//! | 授权判定（fail-closed，仅覆盖 offline 范围） | 已落 |
//! | 离线 JSON 解析（无网络参数） | 已落 |
//! | 联网采集 / live / ALFRED vintage | **未实现** |
//!
//! ## 责任边界
//!
//! 本库**做**：类型化源事实、离线解析与校验、跨源守卫的「自己那一侧」、只读授权判定。
//! 本库**不做**：联网采集、凭据处理、单位换算、派生指标、存储或分发。
//!
//! ## 非目标
//!
//! - 不是联网采集器：无 HTTP 客户端依赖、无端点字面量、不读环境变量或凭据
//! - 不实现派生指标（净流动性、利差、Credit Impulse、z-score、Regime 状态归 analytics）
//! - 不做单位换算（归下游 Normalize）；曲线构建归 `yieldx`
//! - 不新建共享 core crate：本库的公共形状是与兄弟库**各自实现一遍**的同义形状
//!
//! ## 诚实边界
//!
//! `production_decision = NO-GO`；清单 COMPLETE ≠ ship；authorization ≠ Production Ready。
//! offline 范围被授权不等于 live 被授权：证据的 `receipt` 同时登记 `live = no-go`。
//!
//! # 最小示例
//!
//! ```
//! use fredx::{
//!     parse_fred_observations, FredAuthorization, FredAccessMode, authorize_fred,
//!     documented_fred_evidence, fred_publication_semantics, is_formal_pit_eligible,
//! };
//!
//! let input = r#"{"_synthetic": true, "records": [
//!     {"series_id": "WALCL", "date": "2026-08-13", "value": 7000000.0,
//!      "unit": "Millions of U.S. Dollars", "frequency": "weekly"}
//! ]}"#;
//! let observations = parse_fred_observations(input)?;
//! assert_eq!(observations[0].series_id, "WALCL");
//!
//! // offline 范围被证据覆盖，live 不被覆盖。
//! let as_of = fredx::Date::new(2026, 8, 20)?;
//! let evidence = documented_fred_evidence();
//! assert!(matches!(
//!     authorize_fred(Some(&evidence), FredAccessMode::Offline, as_of),
//!     FredAuthorization::Authorized { .. }
//! ));
//! assert!(matches!(
//!     authorize_fred(Some(&evidence), FredAccessMode::Live, as_of),
//!     FredAuthorization::Denied { .. }
//! ));
//!
//! // 正式 PIT 资格恒为 false（推断层）。
//! assert!(!is_formal_pit_eligible());
//! assert_eq!(
//!     fred_publication_semantics().eligibility,
//!     fredx::PitEligibility::NotEligible
//! );
//! # Ok::<(), fredx::FredError>(())
//! ```

pub mod authz;
pub mod error;
pub mod parse;
pub mod pit;
pub mod routing;
pub mod series;
pub mod value;

pub use authz::{
    authorize_fred, documented_fred_evidence, mode_label, validate_authorization_evidence,
    FredAccessMode, FredAuthorization, FredAuthorizationEvidence, FRED_DECISION_ID, FRED_SIGNED_BY,
};
pub use error::{FredError, FredErrorKind, FredResult};
pub use parse::parse_fred_observations;
pub use pit::{
    fred_publication_semantics, is_formal_pit_eligible, AvailabilityEvidence,
    FredPublicationSemantics, PitEligibility, TimePrecision,
};
pub use routing::{
    claim_authoritative_write, ensure_baml_freeze, ensure_not_silent_substitution,
    ensure_product_local, is_bea_table_id, is_near_synonym_pair, FredProduct, BEA_TABLE_OWNER,
    CURVE_BUILD_OWNER, NEAR_SYNONYM_PAIRS,
};
pub use series::{
    ensure_known_series, ensure_source_frequency, ensure_source_unit, is_collected_series,
    is_curve_input_series, is_known_series, is_mapping_only_series, is_unique_write_authority,
    series_frequency, series_unit, ALL_SERIES, AUTHORITATIVE_WRITE_SERIES, BAML_ALLOWED_SERIES,
    BLS_FORWARD_SERIES, CURVE_SERIES, FUNDAMENTAL_SERIES, GLOBAL_REGIONAL_SERIES, P0_SERIES,
};
// series ID 常量：源事实的一等公民，直接暴露在门面上（也可经 `fredx::series` 访问）。
pub use series::{
    BAMLC0A0CM, BAMLH0A0HYM2, BOGZ1FL662090005Q, CES0500000003, CPHPTT01EZM659N, CPILFESL,
    DCOILWTICO, DEXCHUS, DEXJPUS, DEXUSEU, DFF, DFII10, DGS10, DGS2, FDHBFRBN, FYFSGDA188S, ICSA,
    IRLTLT01JPM156N, JPNCPIALLMINMEI, JTSJOL, NASDAQCOM, NIKKEI225, PAYEMS, PCEPILFE, PCOPPUSDM,
    RESPPLLOPNWW, RRPONTSYD, SOFR, SP500, T10Y2Y, T10YIE, T5YIFR, TEDRATE, UNRATE, VIXCLS, WALCL,
    WRESBAL, WTREGEN,
};
pub use value::{
    validate_date, validate_observation, validate_period, Date, FredMissingReason, FredObservation,
    FredUnit, FredValue, Frequency, Period, UNIT_BILLIONS_OF_USD, UNIT_MILLIONS_OF_USD,
};
