//! fredx 的源事实值对象：日期、期间、频率、源侧单位、观测值形态。
//!
//! 本模块只表达源事实，**不做**任何派生计算（净流动性、利差、Credit Impulse、
//! z-score、Regime 状态等一律归 analytics），也**不做**单位换算（归下游 Normalize）。

use crate::error::{FredError, FredResult};

/// 严格 ISO 日期（`YYYY-MM-DD`）。
///
/// 字段公开以便构造与模式匹配，但**直接构造不校验**；构造入口用 [`Date::new`] /
/// [`Date::parse`]，完整性校验用 [`validate_date`]。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Date {
    /// 年（本库只接受 1..=9999）。
    pub year: i16,
    /// 月（1..=12）。
    pub month: u8,
    /// 日（按月份与闰年校验）。
    pub day: u8,
}

impl Date {
    /// 构造并校验一个日期。
    ///
    /// # Errors
    ///
    /// 年份越界、月份不在 1..=12、或日超出该月实际天数时返回 [`FredError::Invalid`]。
    pub fn new(year: i16, month: u8, day: u8) -> FredResult<Self> {
        let date = Self { year, month, day };
        validate_date(&date)?;
        Ok(date)
    }

    /// 按严格 ISO 形态 `YYYY-MM-DD` 解析。
    ///
    /// 只接受四位年 + 两位月 + 两位日、以 `-` 分隔且长度恰为 10 的形态。
    /// 拒绝 `2026-2-3`（未补零）、`2026/02/03`（分隔符不符）与带时间部分者。
    ///
    /// # Errors
    ///
    /// 形态不符或取值非法时返回 [`FredError::Invalid`]。
    pub fn parse(text: &str) -> FredResult<Self> {
        let bytes = text.as_bytes();
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return Err(FredError::Invalid(
                "日期须为严格的 YYYY-MM-DD（月/日两位补零）".into(),
            ));
        }
        let year = parse_digits(&bytes[0..4])
            .ok_or_else(|| FredError::Invalid("年份须为四位数字".into()))?;
        let month = parse_digits(&bytes[5..7])
            .ok_or_else(|| FredError::Invalid("月份须为两位数字".into()))?;
        let day = parse_digits(&bytes[8..10])
            .ok_or_else(|| FredError::Invalid("日期须为两位数字".into()))?;
        let year = i16::try_from(year).map_err(|_| FredError::Invalid("年份越界".into()))?;
        let month = u8::try_from(month).map_err(|_| FredError::Invalid("月份越界".into()))?;
        let day = u8::try_from(day).map_err(|_| FredError::Invalid("日期越界".into()))?;
        Self::new(year, month, day)
    }

    /// 该年是否为闰年。
    #[must_use]
    pub const fn is_leap_year(year: i16) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    /// 该年该月的天数；月份非法时返回 `0`（不 panic）。
    #[must_use]
    pub const fn days_in_month(year: i16, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 0,
        }
    }
}

/// 解析定长十进制串；出现非数字字符时返回 `None`。
fn parse_digits(bytes: &[u8]) -> Option<u32> {
    let mut acc: u32 = 0;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return None;
        }
        acc = acc * 10 + u32::from(b - b'0');
    }
    Some(acc)
}

/// 校验日期：年份 `1..=9999`、月 `1..=12`、日不超过该月实际天数。
///
/// # Errors
///
/// 任一项越界时返回 [`FredError::Invalid`]。
pub fn validate_date(date: &Date) -> FredResult<()> {
    if date.year < 1 || date.year > 9999 {
        return Err(FredError::Invalid("年份须在 1..=9999".into()));
    }
    if date.month < 1 || date.month > 12 {
        return Err(FredError::Invalid("月份须在 1..=12".into()));
    }
    let limit = Date::days_in_month(date.year, date.month);
    if date.day < 1 || date.day > limit {
        return Err(FredError::Invalid(format!(
            "日期须在 1..={limit}（{}-{:02}）",
            date.year, date.month
        )));
    }
    Ok(())
}

/// 业务期间。MUST NOT 用裸字符串顶替。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Period {
    /// 日频观测。
    Day(Date),
    /// 月频观测。
    Month {
        /// 年。
        year: i16,
        /// 月（1..=12）。
        month: u8,
    },
    /// 季频观测。
    Quarter {
        /// 年。
        year: i16,
        /// 季（1..=4）。
        quarter: u8,
    },
    /// 年频观测。
    Year(i16),
    /// 事件型观测（以日期定位）。
    Event {
        /// 事件日期。
        date: Date,
    },
}

/// 校验期间：日期分量合法、月 `1..=12`、季 `1..=4`。
///
/// # Errors
///
/// 任一分量越界时返回 [`FredError::Invalid`]。
pub fn validate_period(period: &Period) -> FredResult<()> {
    match *period {
        Period::Day(date) | Period::Event { date } => validate_date(&date),
        Period::Month { year, month } => {
            if !(1..=9999).contains(&year) {
                return Err(FredError::Invalid("年份须在 1..=9999".into()));
            }
            if !(1..=12).contains(&month) {
                return Err(FredError::Invalid("月份须在 1..=12".into()));
            }
            Ok(())
        }
        Period::Quarter { year, quarter } => {
            if !(1..=9999).contains(&year) {
                return Err(FredError::Invalid("年份须在 1..=9999".into()));
            }
            if !(1..=4).contains(&quarter) {
                return Err(FredError::Invalid("季度须在 1..=4".into()));
            }
            Ok(())
        }
        Period::Year(year) => {
            if !(1..=9999).contains(&year) {
                return Err(FredError::Invalid("年份须在 1..=9999".into()));
            }
            Ok(())
        }
    }
}

/// 源侧频率。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Frequency {
    /// 日频。
    Daily,
    /// 周频。
    Weekly,
    /// 月频。
    Monthly,
    /// 季频。
    Quarterly,
    /// 年频。
    Annual,
    /// 事件型（不定期、由事件触发）。
    Event,
    /// 不规则。
    Irregular,
}

impl Frequency {
    /// 本库离线输入使用的稳定记号。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Quarterly => "quarterly",
            Self::Annual => "annual",
            Self::Event => "event",
            Self::Irregular => "irregular",
        }
    }

    /// 解析本库离线输入使用的频率记号（**只接受小写**）。
    ///
    /// # Errors
    ///
    /// 记号不在白名单内时返回 [`FredError::Invalid`]。
    pub fn parse(text: &str) -> FredResult<Self> {
        match text {
            "daily" => Ok(Self::Daily),
            "weekly" => Ok(Self::Weekly),
            "monthly" => Ok(Self::Monthly),
            "quarterly" => Ok(Self::Quarterly),
            "annual" => Ok(Self::Annual),
            "event" => Ok(Self::Event),
            "irregular" => Ok(Self::Irregular),
            _ => Err(FredError::Invalid(format!(
                "频率记号不在白名单内（{text}）"
            ))),
        }
    }
}

/// 源侧单位。
///
/// **保留源单位**；单位换算 MUST NOT 在源层发生（归下游 Normalize）。
/// 采用开放 newtype 而非封闭枚举：清单只对部分序列声明了源单位，
/// 封闭枚举会迫使实现方为其余序列**编造**单位取值。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FredUnit(String);

/// `Millions of U.S. Dollars`：`WALCL` / `WTREGEN` 的源单位。
pub const UNIT_MILLIONS_OF_USD: &str = "Millions of U.S. Dollars";

/// `Billions of U.S. Dollars`：`RRPONTSYD` 的源单位（与上者**不同量级**）。
pub const UNIT_BILLIONS_OF_USD: &str = "Billions of U.S. Dollars";

impl FredUnit {
    /// 构造源侧单位：非空、无首尾空白、不含控制字符。
    ///
    /// # Errors
    ///
    /// 上述任一条件不满足时返回 [`FredError::Invalid`]。
    pub fn new(text: &str) -> FredResult<Self> {
        if text.is_empty() || text.trim() != text {
            return Err(FredError::Invalid("源单位须为非空且无首尾空白".into()));
        }
        if text.chars().any(char::is_control) {
            return Err(FredError::Invalid("源单位不得含控制字符".into()));
        }
        Ok(Self(text.to_owned()))
    }

    /// 源单位字面量。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 缺失原因。MUST NOT 把缺失静默折算为 `0`。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FredMissingReason {
    /// 源以缺省标记表达「该期间无观测」。
    NoObservation,
}

/// 观测值：要么是数值，要么是**具名**缺失原因。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FredValue {
    /// 已发布的数值。
    Present(f64),
    /// 缺失，附具名原因。
    Missing(FredMissingReason),
}

impl FredValue {
    /// 数值（缺失时为 `None`）。
    #[must_use]
    pub fn as_f64(self) -> Option<f64> {
        match self {
            Self::Present(v) => Some(v),
            Self::Missing(_) => None,
        }
    }

    /// 是否缺失。
    #[must_use]
    pub fn is_missing(self) -> bool {
        matches!(self, Self::Missing(_))
    }
}

/// 一条 FRED 源事实观测。
///
/// 身份 = `series_id` + `period` + `vintage`（`vintage` 无官方面时为 `None`）。
#[derive(Debug, Clone, PartialEq)]
pub struct FredObservation {
    /// 源条目标识（FRED series ID），如 `WALCL`。
    pub series_id: String,
    /// 业务期间。
    pub period: Period,
    /// 观测值。
    pub value: FredValue,
    /// 源侧单位（保留原样）。
    pub unit: FredUnit,
    /// 源侧频率。
    pub frequency: Frequency,
    /// 修订标识；本层无官方 vintage 面时为 `None`，MUST NOT 伪造。
    pub vintage: Option<Date>,
}

/// 校验一条观测：标识非空、期间合法、数值有限、修订日期合法。
///
/// 单位与频率的**跨表一致性**由 [`crate::series`] 的守卫负责（见
/// [`crate::series::ensure_source_unit`] / [`crate::series::ensure_source_frequency`]），
/// 因为那需要清单侧序列表。
///
/// # Errors
///
/// 任一条件不满足时返回 [`FredError::Invalid`]。
pub fn validate_observation(observation: &FredObservation) -> FredResult<()> {
    if observation.series_id.is_empty() || observation.series_id.trim() != observation.series_id {
        return Err(FredError::Invalid("series_id 须为非空且无首尾空白".into()));
    }
    validate_period(&observation.period)?;
    if let FredValue::Present(v) = observation.value {
        if !v.is_finite() {
            return Err(FredError::Invalid(
                "观测值须为有限数（NaN / 无穷不得静默通过）".into(),
            ));
        }
    }
    if let Some(vintage) = observation.vintage {
        validate_date(&vintage)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit() -> FredUnit {
        FredUnit::new("Index").expect("单位合法")
    }

    fn observation(series_id: &str) -> FredObservation {
        FredObservation {
            series_id: series_id.to_owned(),
            period: Period::Day(Date::new(2026, 8, 15).expect("日期合法")),
            value: FredValue::Present(1.5),
            unit: unit(),
            frequency: Frequency::Daily,
            vintage: None,
        }
    }

    #[test]
    fn date_parse_accepts_strict_iso() {
        let date = Date::parse("2026-08-15").expect("应可解析");
        assert_eq!(
            date,
            Date {
                year: 2026,
                month: 8,
                day: 15
            }
        );
    }

    #[test]
    fn date_parse_rejects_non_strict_forms() {
        for bad in [
            "2026-2-3",
            "2026/02/03",
            "2026-02-03T00:00:00Z",
            "2026-02-03 00:00",
            "2026-02-3",
            "26-02-03",
            "2026-02-03extra",
            "",
        ] {
            assert!(Date::parse(bad).is_err(), "应拒绝：{bad}");
        }
    }

    #[test]
    fn date_rejects_impossible_calendar_days() {
        assert!(Date::parse("2026-13-01").is_err());
        assert!(Date::parse("2026-00-10").is_err());
        assert!(Date::parse("2026-02-30").is_err());
        assert!(Date::parse("2026-04-31").is_err());
        assert!(Date::parse("2025-02-29").is_err());
        assert!(Date::parse("2024-02-29").is_ok());
        assert!(Date::parse("0000-01-01").is_err());
    }

    #[test]
    fn leap_year_rule_matches_gregorian() {
        assert!(Date::is_leap_year(2024));
        assert!(!Date::is_leap_year(2025));
        assert!(!Date::is_leap_year(1900));
        assert!(Date::is_leap_year(2000));
    }

    #[test]
    fn days_in_month_never_panics_on_bad_month() {
        assert_eq!(Date::days_in_month(2026, 2), 28);
        assert_eq!(Date::days_in_month(2024, 2), 29);
        assert_eq!(Date::days_in_month(2026, 12), 31);
        assert_eq!(Date::days_in_month(2026, 13), 0);
        assert_eq!(Date::days_in_month(2026, 0), 0);
    }

    #[test]
    fn period_validation_covers_every_variant() {
        let day = Date::new(2026, 8, 15).expect("日期合法");
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

    #[test]
    fn frequency_tokens_round_trip_and_reject_unknown() {
        for freq in [
            Frequency::Daily,
            Frequency::Weekly,
            Frequency::Monthly,
            Frequency::Quarterly,
            Frequency::Annual,
            Frequency::Event,
            Frequency::Irregular,
        ] {
            assert_eq!(Frequency::parse(freq.as_str()).expect("回环"), freq);
        }
        assert!(Frequency::parse("Daily").is_err());
        assert!(Frequency::parse("").is_err());
    }

    #[test]
    fn unit_rejects_blank_and_control_chars() {
        assert_eq!(
            FredUnit::new(UNIT_BILLIONS_OF_USD).expect("合法").as_str(),
            UNIT_BILLIONS_OF_USD
        );
        assert!(FredUnit::new("").is_err());
        assert!(FredUnit::new(" Index").is_err());
        assert!(FredUnit::new("Index\n").is_err());
        // 内部（非首尾）控制字符也必须被拒绝：它不会被 trim 捕获。
        assert!(FredUnit::new("Per\u{7}cent").is_err());
        assert!(FredUnit::new("Per\u{0}cent").is_err());
    }

    #[test]
    fn missing_value_is_named_and_never_zero() {
        let missing = FredValue::Missing(FredMissingReason::NoObservation);
        assert!(missing.is_missing());
        assert_eq!(missing.as_f64(), None);
        assert_eq!(FredValue::Present(0.0).as_f64(), Some(0.0));
    }

    #[test]
    fn validate_observation_accepts_well_formed_record() {
        assert!(validate_observation(&observation("WALCL")).is_ok());
    }

    #[test]
    fn validate_observation_rejects_bad_series_id_and_values() {
        let mut bad = observation(" WALCL");
        assert!(validate_observation(&bad).is_err());

        bad = observation("");
        assert!(validate_observation(&bad).is_err());

        bad = observation("WALCL");
        bad.value = FredValue::Present(f64::NAN);
        assert!(validate_observation(&bad).is_err());

        bad.value = FredValue::Present(f64::INFINITY);
        assert!(validate_observation(&bad).is_err());

        bad = observation("WALCL");
        bad.period = Period::Month {
            year: 2026,
            month: 0,
        };
        assert!(validate_observation(&bad).is_err());

        bad = observation("WALCL");
        bad.vintage = Some(Date {
            year: 2026,
            month: 2,
            day: 30,
        });
        assert!(validate_observation(&bad).is_err());
    }
}
