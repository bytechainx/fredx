//! fredx 的源事实常量表：FRED series ID 全集、分组、源单位与频率。
//!
//! 本模块是 `specs/adapter/fred.md` 的**范围对齐落点**。它只登记清单明确列出的
//! series ID、频率与源单位；清单未列出的 ID MUST NOT 在此新增（尤其
//! `FDHBFRBN`「等」的全集未钉死，见 `cross-source-routing.md` §8 P5）。
//!
//! 边界：`WRESBAL` 在本库为 **mapping-only**（不进采集集合），仅因权威写入主权
//! （Owner 联裁 `WALCL-AUTHORITY-2026-08-20-A`）而被登记。

use crate::error::{FredError, FredResult};
use crate::value::{FredObservation, Frequency, UNIT_BILLIONS_OF_USD, UNIT_MILLIONS_OF_USD};

// ---------------------------------------------------------------------------
// 一、核心与流动性相关采集范围（P0 组）
// ---------------------------------------------------------------------------

/// 10Y 实际利率（TIPS）。
pub const DFII10: &str = "DFII10";
/// 10Y 盈亏平衡通胀。
pub const T10YIE: &str = "T10YIE";
/// 高收益债利差 HY OAS（**已冻结**：只允许本 ID，禁其他 BAML 系列）。
pub const BAMLH0A0HYM2: &str = "BAMLH0A0HYM2";
/// 2s10s 收益率曲线利差。
pub const T10Y2Y: &str = "T10Y2Y";
/// 初请失业金（周频）。
pub const ICSA: &str = "ICSA";
/// Fed 总资产源序列（**必须采集**，源单位 Millions）。
pub const WALCL: &str = "WALCL";
/// 财政部 TGA 源序列（**必须采集**，源单位 Millions）。
pub const WTREGEN: &str = "WTREGEN";
/// 隔夜逆回购 RRP（**必须采集**，源单位 Billions，与上两者**不同量级**）。
pub const RRPONTSYD: &str = "RRPONTSYD";
/// SOFR 担保隔夜融资利率。
pub const SOFR: &str = "SOFR";
/// 联邦基金有效利率（逐日；**≠ `EFFR`**）。
pub const DFF: &str = "DFF";
/// VIX 波动率主源（R1 升主源）。
pub const VIXCLS: &str = "VIXCLS";
/// 标普 500 指数（R2 主源；**≠ `SPY`**）。
pub const SP500: &str = "SP500";
/// WTI 现货价格（R2 近义；**≠ `CL=F`**）。
pub const DCOILWTICO: &str = "DCOILWTICO";

// ---------------------------------------------------------------------------
// 二、利率与曲线组（曲线输入点）
// ---------------------------------------------------------------------------

/// 2Y 国债收益率（曲线输入）。
pub const DGS2: &str = "DGS2";
/// 10Y 名义收益率（BE 交叉验证输入）。
pub const DGS10: &str = "DGS10";
/// 5Y/5Y 远期盈亏平衡。
pub const T5YIFR: &str = "T5YIFR";
/// TED Spread（信用压力辅助）。
pub const TEDRATE: &str = "TEDRATE";

// ---------------------------------------------------------------------------
// 三、经济基本面组
// ---------------------------------------------------------------------------

/// 核心 PCE 价格指数（通胀政策锚；**≠ `PCEPI`**，采集主责在本域 `fred-forward`）。
pub const PCEPILFE: &str = "PCEPILFE";
/// 民用失业率。
pub const UNRATE: &str = "UNRATE";
/// 非农就业。
pub const PAYEMS: &str = "PAYEMS";
/// 平均时薪。
pub const CES0500000003: &str = "CES0500000003";

// ---------------------------------------------------------------------------
// 四、全球与区域组
// ---------------------------------------------------------------------------

/// USD/JPY（R1 独占主源，无备源）。
pub const DEXJPUS: &str = "DEXJPUS";
/// JGB 10Y 收益率（月频）。
pub const IRLTLT01JPM156N: &str = "IRLTLT01JPM156N";
/// 日本 CPI（月频）。
pub const JPNCPIALLMINMEI: &str = "JPNCPIALLMINMEI";
/// EURUSD。
pub const DEXUSEU: &str = "DEXUSEU";
/// 欧元区 HICP（月频）。
pub const CPHPTT01EZM659N: &str = "CPHPTT01EZM659N";
/// 美国赤字/GDP（季频）。
pub const FYFSGDA188S: &str = "FYFSGDA188S";
/// Treasury 净发行相关（月频；「等」的全集未钉死，MUST NOT 自行扩集）。
pub const FDHBFRBN: &str = "FDHBFRBN";
/// Z.1 源侧交叉验证序列（季频）。
pub const BOGZ1FL662090005Q: &str = "BOGZ1FL662090005Q";
/// 纳斯达克综合（R2；**≠ `QQQ`**）。
pub const NASDAQCOM: &str = "NASDAQCOM";
/// 日经 225（R2；**≠ `EWJ`**）。
pub const NIKKEI225: &str = "NIKKEI225";
/// 全球铜价（R2 月频近义；**≠ COMEX `HG=F` 日频**）。
pub const PCOPPUSDM: &str = "PCOPPUSDM";
/// USD/CNY 在岸（R2 近义；**CNY ≠ CNH**）。
pub const DEXCHUS: &str = "DEXCHUS";

// ---------------------------------------------------------------------------
// 五、BLS 转发缺口闭合（核心 CPI + JOLTS）
// ---------------------------------------------------------------------------

/// 核心 CPI（**≠ `CPIAUCSL` 总 CPI**）。
pub const CPILFESL: &str = "CPILFESL";
/// JOLTS 职位空缺（月频）。
pub const JTSJOL: &str = "JTSJOL";

// ---------------------------------------------------------------------------
// 六、非采集登记：mapping-only、冻结令 drop 项、未决候选
// ---------------------------------------------------------------------------

/// 准备金余额（**mapping-only**：接受映射，不进采集集合；权威写入仍在本域）。
pub const WRESBAL: &str = "WRESBAL";
/// 公司债期权调整利差（**已 drop**）：曾违反 BAML 冻结令，恢复须 Owner 书面豁免。
pub const BAMLC0A0CM: &str = "BAMLC0A0CM";
/// 候选序列（身份与权威归属**未决**）：本域 MUST NOT 主张权威写入。
pub const RESPPLLOPNWW: &str = "RESPPLLOPNWW";

// ---------------------------------------------------------------------------
// 分组与集合
// ---------------------------------------------------------------------------

/// P0 组：核心与流动性相关采集范围（13 个）。
pub const P0_SERIES: &[&str] = &[
    DFII10,
    T10YIE,
    BAMLH0A0HYM2,
    T10Y2Y,
    ICSA,
    WALCL,
    WTREGEN,
    RRPONTSYD,
    SOFR,
    DFF,
    VIXCLS,
    SP500,
    DCOILWTICO,
];

/// 利率与曲线组：曲线输入点（4 个）。
pub const CURVE_SERIES: &[&str] = &[DGS2, DGS10, T5YIFR, TEDRATE];

/// 经济基本面组（4 个）。
pub const FUNDAMENTAL_SERIES: &[&str] = &[PCEPILFE, UNRATE, PAYEMS, CES0500000003];

/// 全球与区域组（12 个）。
pub const GLOBAL_REGIONAL_SERIES: &[&str] = &[
    DEXJPUS,
    IRLTLT01JPM156N,
    JPNCPIALLMINMEI,
    DEXUSEU,
    CPHPTT01EZM659N,
    FYFSGDA188S,
    FDHBFRBN,
    BOGZ1FL662090005Q,
    NASDAQCOM,
    NIKKEI225,
    PCOPPUSDM,
    DEXCHUS,
];

/// BLS 转发闭合组（2 个）。
pub const BLS_FORWARD_SERIES: &[&str] = &[CPILFESL, JTSJOL];

/// 采集集合全集（35 个纯 FRED series ID，来自清单明确列出的行）。
pub const ALL_SERIES: &[&str] = &[
    DFII10,
    T10YIE,
    BAMLH0A0HYM2,
    T10Y2Y,
    ICSA,
    WALCL,
    WTREGEN,
    RRPONTSYD,
    SOFR,
    DFF,
    VIXCLS,
    SP500,
    DCOILWTICO,
    DGS2,
    DGS10,
    T5YIFR,
    TEDRATE,
    PCEPILFE,
    UNRATE,
    PAYEMS,
    CES0500000003,
    DEXJPUS,
    IRLTLT01JPM156N,
    JPNCPIALLMINMEI,
    DEXUSEU,
    CPHPTT01EZM659N,
    FYFSGDA188S,
    FDHBFRBN,
    BOGZ1FL662090005Q,
    NASDAQCOM,
    NIKKEI225,
    PCOPPUSDM,
    DEXCHUS,
    CPILFESL,
    JTSJOL,
];

/// 本域为**权威 Observation 唯一写入方**的 series（Owner 联裁
/// `WALCL-AUTHORITY-2026-08-20-A`，选项 A）。
///
/// 其它源对这四个序列只可对账告警，MUST NOT 覆盖权威值、MUST NOT 双写 canonical key。
pub const AUTHORITATIVE_WRITE_SERIES: &[&str] = &[WALCL, WTREGEN, RRPONTSYD, WRESBAL];

/// BAML 冻结令允许的**唯一** BAML 系列。
pub const BAML_ALLOWED_SERIES: &[&str] = &[BAMLH0A0HYM2];

/// 该 ID 是否在**采集集合**内（35 个纯 FRED ID）。
#[must_use]
pub fn is_collected_series(series_id: &str) -> bool {
    ALL_SERIES.contains(&series_id)
}

/// 该 ID 是否为本库的 **mapping-only** 序列（接受映射、不进采集集合）。
#[must_use]
pub fn is_mapping_only_series(series_id: &str) -> bool {
    series_id == WRESBAL
}

/// 该 ID 是否为本库**已登记**的序列（采集集合 ∪ mapping-only）。
#[must_use]
pub fn is_known_series(series_id: &str) -> bool {
    is_collected_series(series_id) || is_mapping_only_series(series_id)
}

/// 该 ID 是否为曲线输入点：按 `cross-source-routing.md` §2 的字面前缀
/// `DGS*` / `T10Y*` 判定。
///
/// 曲线点**可作输入**，但曲线构建归 `yieldx`（见
/// [`crate::routing::ensure_curve_build_routed`]）。
#[must_use]
pub fn is_curve_input_series(series_id: &str) -> bool {
    series_id.starts_with("DGS") || series_id.starts_with("T10Y")
}

/// 清单明确声明过的**源单位**；未声明的序列返回 `None`。
///
/// 本库**保留源单位**，不做换算。`RRPONTSYD`（Billions）与
/// `WALCL` / `WTREGEN`（Millions）量级不同，MUST NOT 视为同一单位。
#[must_use]
pub fn series_unit(series_id: &str) -> Option<&'static str> {
    match series_id {
        WALCL | WTREGEN => Some(UNIT_MILLIONS_OF_USD),
        RRPONTSYD => Some(UNIT_BILLIONS_OF_USD),
        _ => None,
    }
}

/// 清单声明的**源侧频率**；未登记的序列返回 `None`。
#[must_use]
pub fn series_frequency(series_id: &str) -> Option<Frequency> {
    let frequency = match series_id {
        ICSA | WALCL | WTREGEN => Frequency::Weekly,
        PCEPILFE | UNRATE | PAYEMS | CES0500000003 | IRLTLT01JPM156N | JPNCPIALLMINMEI
        | CPHPTT01EZM659N | FDHBFRBN | PCOPPUSDM | CPILFESL | JTSJOL => Frequency::Monthly,
        FYFSGDA188S | BOGZ1FL662090005Q => Frequency::Quarterly,
        DFII10 | T10YIE | BAMLH0A0HYM2 | T10Y2Y | RRPONTSYD | SOFR | DFF | VIXCLS | SP500
        | DCOILWTICO | DGS2 | DGS10 | T5YIFR | TEDRATE | DEXJPUS | DEXUSEU | NASDAQCOM
        | NIKKEI225 | DEXCHUS => Frequency::Daily,
        _ => return None,
    };
    Some(frequency)
}

/// 该 series 的权威 Observation 唯一写入方是否为本域（`fredx`）。
#[must_use]
pub fn is_unique_write_authority(series_id: &str) -> bool {
    AUTHORITATIVE_WRITE_SERIES.contains(&series_id)
}

/// 拒绝清单范围外的 series ID（原子失败，MUST NOT 静默接受）。
///
/// # Errors
///
/// 该 ID 未在清单登记（既非采集集合、亦非 mapping-only）时返回
/// [`FredError::SemanticallyRejected`]。
pub fn ensure_known_series(series_id: &str) -> FredResult<()> {
    if is_known_series(series_id) {
        return Ok(());
    }
    Err(FredError::SemanticallyRejected(format!(
        "series_id 不在清单登记范围内：{series_id}"
    )))
}

/// 源单位一致性守卫：清单声明过源单位的序列，其观测单位 MUST 等于源单位。
///
/// 这条守卫**不换算**，只拒绝把 `Billions` 与 `Millions` 混为一谈的输入。
///
/// # Errors
///
/// 单位与清单声明不符时返回 [`FredError::SemanticallyRejected`]。
pub fn ensure_source_unit(observation: &FredObservation) -> FredResult<()> {
    if let Some(expected) = series_unit(&observation.series_id) {
        if observation.unit.as_str() != expected {
            return Err(FredError::SemanticallyRejected(format!(
                "{} 的源单位为 {expected}，输入为 {}；本层不做换算",
                observation.series_id,
                observation.unit.as_str()
            )));
        }
    }
    Ok(())
}

/// 频率一致性守卫：清单声明过频率的序列，其观测频率 MUST 等于清单值。
///
/// # Errors
///
/// 频率与清单声明不符时返回 [`FredError::SemanticallyRejected`]。
pub fn ensure_source_frequency(observation: &FredObservation) -> FredResult<()> {
    if let Some(expected) = series_frequency(&observation.series_id) {
        if observation.frequency != expected {
            return Err(FredError::SemanticallyRejected(format!(
                "{} 的清单频率为 {}，输入为 {}",
                observation.series_id,
                expected.as_str(),
                observation.frequency.as_str()
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{Date, FredUnit, FredValue, Period};

    #[test]
    fn collection_set_is_the_manifest_scope() {
        // 35 = 13（P0）+ 4（曲线）+ 4（基本面）+ 12（全球区域）+ 2（BLS 转发）。
        assert_eq!(ALL_SERIES.len(), 35);
        assert_eq!(
            P0_SERIES.len()
                + CURVE_SERIES.len()
                + FUNDAMENTAL_SERIES.len()
                + GLOBAL_REGIONAL_SERIES.len()
                + BLS_FORWARD_SERIES.len(),
            ALL_SERIES.len()
        );
        // 每个分组项都在全集中，且全集无重复。
        for group in [
            P0_SERIES,
            CURVE_SERIES,
            FUNDAMENTAL_SERIES,
            GLOBAL_REGIONAL_SERIES,
            BLS_FORWARD_SERIES,
        ] {
            for id in group {
                assert!(is_collected_series(id), "分组项不在全集：{id}");
            }
        }
        let mut sorted = ALL_SERIES.to_vec();
        sorted.sort_unstable();
        let before = sorted.len();
        sorted.dedup();
        assert_eq!(before, sorted.len(), "全集出现重复 ID");
    }

    #[test]
    fn mapping_only_and_dropped_ids_are_not_collected() {
        assert!(!is_collected_series(WRESBAL));
        assert!(is_mapping_only_series(WRESBAL));
        assert!(is_known_series(WRESBAL));
        assert!(!is_known_series(BAMLC0A0CM));
        assert!(!is_known_series("NOT_A_FRED_ID"));
    }

    #[test]
    fn curve_input_matches_documented_prefixes() {
        assert!(is_curve_input_series(DGS2));
        assert!(is_curve_input_series(DGS10));
        assert!(is_curve_input_series(T10Y2Y));
        assert!(is_curve_input_series(T10YIE));
        assert!(!is_curve_input_series(BAMLH0A0HYM2));
        assert!(!is_curve_input_series(T5YIFR));
    }

    #[test]
    fn authoritative_write_set_is_exactly_four_series() {
        assert_eq!(AUTHORITATIVE_WRITE_SERIES.len(), 4);
        for id in AUTHORITATIVE_WRITE_SERIES {
            assert!(is_unique_write_authority(id));
        }
        assert!(!is_unique_write_authority(SP500));
        assert!(!is_unique_write_authority(RESPPLLOPNWW));
    }

    #[test]
    fn source_units_differ_in_magnitude_and_are_not_converted() {
        assert_eq!(series_unit(WALCL), Some(UNIT_MILLIONS_OF_USD));
        assert_eq!(series_unit(WTREGEN), Some(UNIT_MILLIONS_OF_USD));
        assert_eq!(series_unit(RRPONTSYD), Some(UNIT_BILLIONS_OF_USD));
        assert_ne!(series_unit(WALCL), series_unit(RRPONTSYD));
        assert_eq!(series_unit(SP500), None);
    }

    #[test]
    fn source_frequency_covers_every_collected_series() {
        for id in ALL_SERIES {
            assert!(series_frequency(id).is_some(), "缺频率声明：{id}");
        }
        assert_eq!(series_frequency(ICSA), Some(Frequency::Weekly));
        assert_eq!(series_frequency(FYFSGDA188S), Some(Frequency::Quarterly));
        assert_eq!(series_frequency(PCEPILFE), Some(Frequency::Monthly));
        assert_eq!(series_frequency(WRESBAL), None);
    }

    #[test]
    fn ensure_known_series_rejects_out_of_scope_ids() {
        assert!(ensure_known_series(WALCL).is_ok());
        assert!(ensure_known_series(WRESBAL).is_ok());
        assert!(ensure_known_series("CPIAUCSL").is_err());
    }

    fn observation(series_id: &str, unit: &str, frequency: Frequency) -> FredObservation {
        FredObservation {
            series_id: series_id.to_owned(),
            period: Period::Day(Date::new(2026, 8, 15).expect("日期合法")),
            value: FredValue::Present(1.0),
            unit: FredUnit::new(unit).expect("单位合法"),
            frequency,
            vintage: None,
        }
    }

    #[test]
    fn unit_guard_rejects_billions_as_millions() {
        let wrong = observation(RRPONTSYD, UNIT_MILLIONS_OF_USD, Frequency::Daily);
        let err = ensure_source_unit(&wrong).expect_err("应拒绝");
        assert_eq!(err.kind(), crate::FredErrorKind::SemanticallyRejected);

        let right = observation(RRPONTSYD, UNIT_BILLIONS_OF_USD, Frequency::Daily);
        assert!(ensure_source_unit(&right).is_ok());

        // 未声明源单位的序列不做该判定。
        assert!(ensure_source_unit(&observation(SP500, "Index", Frequency::Daily)).is_ok());
    }

    #[test]
    fn frequency_guard_rejects_manifest_drift() {
        let wrong = observation(WALCL, UNIT_MILLIONS_OF_USD, Frequency::Monthly);
        assert!(ensure_source_frequency(&wrong).is_err());
        let right = observation(WALCL, UNIT_MILLIONS_OF_USD, Frequency::Weekly);
        assert!(ensure_source_frequency(&right).is_ok());
    }
}
