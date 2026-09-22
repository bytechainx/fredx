//! fredx 侧的跨源守卫：近义非同 ID、BAML 冻结令、写入主权、曲线边界。
//!
//! 本模块只实现 `cross-source-routing.md` §7 分配给 `fredx` 的**自己那一侧**。
//! 跨源整体语义归该契约文档，MUST NOT 在本库内重新裁定任何未决项。

use crate::error::{FredError, FredResult};
use crate::series::{
    is_curve_input_series, is_unique_write_authority, BAMLC0A0CM, BAMLH0A0HYM2,
    BAML_ALLOWED_SERIES, CPILFESL, DCOILWTICO, DEXCHUS, DFF, DGS10, NASDAQCOM, NIKKEI225, PCEPILFE,
    PCOPPUSDM, RESPPLLOPNWW, SP500, WRESBAL,
};

/// 曲线构建的归属方（`cross-source-routing.md` §2）。
pub const CURVE_BUILD_OWNER: &str = "yieldx";

/// BEA 表号（形如 `T10101`）的归属方（`cross-source-routing.md` §7）。
pub const BEA_TABLE_OWNER: &str = "beax";

/// 近义非同 ID 全表中**涉及 `fredx` 的 14 对**（清单逐条钉死，MUST NOT 互为别名）。
///
/// 左列与右列语义不同，MUST NOT 静默替换；顺序对其判定无影响。
pub const NEAR_SYNONYM_PAIRS: &[(&str, &str)] = &[
    (DFF, "EFFR"),
    (SP500, "SPY"),
    (DCOILWTICO, "CL=F"),
    (PCOPPUSDM, "HG=F"),
    ("GDPNow", "GDP"),
    (PCEPILFE, "PCEPI"),
    (CPILFESL, "CPIAUCSL"),
    (NASDAQCOM, "QQQ"),
    (NIKKEI225, "EWJ"),
    (DEXCHUS, "USDCNH"),
    ("DX-Y.NYB", "DTWEXBGS"),
    ("T20100", PCEPILFE),
    ("^TNX", DGS10),
    (RESPPLLOPNWW, WRESBAL),
];

/// 本库可受理的产品种类。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FredProduct {
    /// 源序列观测（本库职责）。
    Observation,
    /// 收益率曲线（期限结构）：归 [`CURVE_BUILD_OWNER`]。
    YieldCurve,
}

/// 该对 ID 是否属于清单钉死的「近义非同 ID」。
#[must_use]
pub fn is_near_synonym_pair(left: &str, right: &str) -> bool {
    NEAR_SYNONYM_PAIRS
        .iter()
        .any(|(a, b)| (left == *a && right == *b) || (left == *b && right == *a))
}

/// 禁静默替换守卫：被钉死的近义对 MUST NOT 互换或互为别名。
///
/// # Errors
///
/// 该对属于 [`NEAR_SYNONYM_PAIRS`] 时返回 [`FredError::SemanticallyRejected`]。
pub fn ensure_not_silent_substitution(left: &str, right: &str) -> FredResult<()> {
    if is_near_synonym_pair(left, right) {
        return Err(FredError::SemanticallyRejected(format!(
            "{left} 与 {right} 语义不同，不得互为别名或静默替换"
        )));
    }
    Ok(())
}

/// 该 ID 是否为 BEA 表号形态（`T` + 5 位数字，如 `T10101`）。
#[must_use]
pub fn is_bea_table_id(candidate: &str) -> bool {
    let bytes = candidate.as_bytes();
    bytes.len() == 6 && bytes[0] == b'T' && bytes[1..].iter().all(u8::is_ascii_digit)
}

/// BAML 冻结令守卫：本库只允许 [`BAMLH0A0HYM2`] 一个 BAML 系列。
///
/// `BAMLC0A0CM` 曾违反冻结令，**已 drop**，恢复须 Owner 书面豁免。
///
/// # Errors
///
/// 输入为其它 BAML 系列时返回 [`FredError::SemanticallyRejected`]。
pub fn ensure_baml_freeze(series_id: &str) -> FredResult<()> {
    if !series_id.starts_with("BAML") || BAML_ALLOWED_SERIES.contains(&series_id) {
        return Ok(());
    }
    if series_id == BAMLC0A0CM {
        return Err(FredError::SemanticallyRejected(
            "BAML 冻结令：BAMLC0A0CM 已 drop，恢复须 Owner 书面豁免".into(),
        ));
    }
    Err(FredError::SemanticallyRejected(format!(
        "BAML 冻结令：本库只允许 {BAMLH0A0HYM2}，不得使用 {series_id}"
    )))
}

/// 越权写入守卫。
///
/// 仅当本域（`fredx`）为该 series 的权威 Observation 唯一写入方时返回 `Ok`；
/// 曲线点权威归[`CURVE_BUILD_OWNER`]、BEA 表号归[`BEA_TABLE_OWNER`]、
/// 归属未决的候选序列一律拒绝。
///
/// 本函数是**只读判定**，MUST NOT 改写任何授权或主权登记值。
///
/// # Errors
///
/// 本域不主张该 ID 的权威写入时返回 [`FredError::WriteAuthorityDenied`]。
pub fn claim_authoritative_write(series_id: &str) -> FredResult<()> {
    if is_unique_write_authority(series_id) {
        return Ok(());
    }
    if is_curve_input_series(series_id) {
        return Err(FredError::WriteAuthorityDenied(format!(
            "{series_id}：曲线点权威写入归 {CURVE_BUILD_OWNER}"
        )));
    }
    if is_bea_table_id(series_id) {
        return Err(FredError::WriteAuthorityDenied(format!(
            "{series_id}：BEA 表号归 {BEA_TABLE_OWNER}"
        )));
    }
    if series_id == RESPPLLOPNWW {
        return Err(FredError::WriteAuthorityDenied(format!(
            "{series_id}：身份与权威归属未决，双方均不得主张"
        )));
    }
    Err(FredError::WriteAuthorityDenied(format!(
        "{series_id}：本域不主张该 series 的权威写入"
    )))
}

/// 产品归属守卫：本库只受理源序列观测；曲线构建归 [`CURVE_BUILD_OWNER`]。
///
/// # Errors
///
/// 产品为 [`FredProduct::YieldCurve`] 时返回 [`FredError::RoutedElsewhere`]。
pub fn ensure_product_local(product: FredProduct) -> FredResult<()> {
    match product {
        FredProduct::Observation => Ok(()),
        FredProduct::YieldCurve => Err(FredError::RoutedElsewhere(format!(
            "曲线构建归 {CURVE_BUILD_OWNER}；本域只提供曲线输入点"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::FredErrorKind;
    use crate::series::{RRPONTSYD, WALCL, WTREGEN};

    #[test]
    fn every_documented_pair_is_detected_in_both_directions() {
        assert_eq!(NEAR_SYNONYM_PAIRS.len(), 14);
        for (left, right) in NEAR_SYNONYM_PAIRS {
            assert!(is_near_synonym_pair(left, right), "{left}/{right}");
            assert!(is_near_synonym_pair(right, left), "{right}/{left}");
            assert_eq!(
                ensure_not_silent_substitution(left, right)
                    .expect_err("应拒绝")
                    .kind(),
                FredErrorKind::SemanticallyRejected
            );
        }
    }

    #[test]
    fn unrelated_pairs_are_allowed() {
        assert!(ensure_not_silent_substitution(SP500, DGS10).is_ok());
        assert!(ensure_not_silent_substitution(WALCL, RRPONTSYD).is_ok());
        assert!(!is_near_synonym_pair("WALCL", "RRPONTSYD"));
    }

    #[test]
    fn baml_freeze_allows_only_the_frozen_series() {
        assert!(ensure_baml_freeze(BAMLH0A0HYM2).is_ok());
        assert!(ensure_baml_freeze(WALCL).is_ok());
        let dropped = ensure_baml_freeze(BAMLC0A0CM).expect_err("应拒绝");
        assert_eq!(dropped.kind(), FredErrorKind::SemanticallyRejected);
        assert!(dropped.to_string().contains("书面豁免"));
        assert!(ensure_baml_freeze("BAMLH0A3HYC").is_err());
    }

    #[test]
    fn authoritative_write_is_limited_to_four_series() {
        for id in [WALCL, WTREGEN, RRPONTSYD, WRESBAL] {
            assert!(claim_authoritative_write(id).is_ok(), "{id}");
        }
        assert_eq!(crate::series::AUTHORITATIVE_WRITE_SERIES.len(), 4);
    }

    #[test]
    fn foreign_write_claims_are_denied_with_distinct_reasons() {
        let curve = claim_authoritative_write(DGS10).expect_err("曲线点");
        assert_eq!(curve.kind(), FredErrorKind::WriteAuthorityDenied);
        assert!(curve.to_string().contains(CURVE_BUILD_OWNER));

        let table = claim_authoritative_write("T20100").expect_err("BEA 表号");
        assert_eq!(table.kind(), FredErrorKind::WriteAuthorityDenied);
        assert!(table.to_string().contains(BEA_TABLE_OWNER));

        let pending = claim_authoritative_write(RESPPLLOPNWW).expect_err("未决");
        assert_eq!(pending.kind(), FredErrorKind::WriteAuthorityDenied);

        let other = claim_authoritative_write(SP500).expect_err("非主权序列");
        assert_eq!(other.kind(), FredErrorKind::WriteAuthorityDenied);
    }

    #[test]
    fn bea_table_id_shape_is_exact() {
        assert!(is_bea_table_id("T10101"));
        assert!(is_bea_table_id("T20100"));
        assert!(!is_bea_table_id("T10Y2Y"));
        assert!(!is_bea_table_id("T1010"));
        assert!(!is_bea_table_id("t10101"));
        assert!(!is_bea_table_id("NIPA"));
    }

    #[test]
    fn curve_products_are_routed_away_while_observations_are_local() {
        assert!(ensure_product_local(FredProduct::Observation).is_ok());
        let err = ensure_product_local(FredProduct::YieldCurve).expect_err("应路由");
        assert_eq!(err.kind(), FredErrorKind::RoutedElsewhere);
        assert!(err.to_string().contains(CURVE_BUILD_OWNER));
    }
}
