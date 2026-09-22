//! fredx 的离线解析器：自有输入形态 → 观测集合。
//!
//! 入口**只接受字符串**：MUST NOT 接受 URL、HTTP 客户端、认证信息或任何网络参数。
//!
//! 输入形态（本库自定，**不是** FRED API 响应的复制）：一个 JSON 对象
//! `{ "_synthetic"?: bool, "_note"?: string, "records": [ … ] }`，每条记录
//! `{ series_id, date, value, unit, frequency, vintage? }`。
//!
//! 语义：未知字段**原子失败**；重复身份（`series_id` + 期间 + `vintage`）**拒绝**且不去重。

use serde::Deserialize;

use crate::error::{FredError, FredResult};
use crate::routing::{ensure_baml_freeze, ensure_product_local, FredProduct};
use crate::series::{ensure_known_series, ensure_source_frequency, ensure_source_unit};
use crate::value::{
    validate_observation, Date, FredMissingReason, FredObservation, FredUnit, FredValue, Frequency,
    Period,
};

/// 输入顶层信封。`_synthetic` / `_note` 为合成样本标注，属**已知字段**。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEnvelope {
    #[serde(default)]
    _synthetic: Option<bool>,
    #[serde(default)]
    _note: Option<String>,
    records: Vec<RawRecord>,
}

/// 单条输入记录。字段名与清单声明的事实对齐（series ID / 期间 / 值 / 源单位 / 频率 / vintage）。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRecord {
    series_id: String,
    date: String,
    value: serde_json::Value,
    unit: String,
    frequency: String,
    #[serde(default)]
    vintage: Option<String>,
}

/// 解析 fredx 的离线输入，返回观测集合。
///
/// 逐条执行：清单范围守卫 → BAML 冻结令 → 频率记号 → 期间投影 → 值形态 →
/// 源单位 / 频率一致性 → 值对象校验。任一条失败即整体失败（原子）。
///
/// # Examples
///
/// ```
/// use fredx::parse_fred_observations;
///
/// let input = r#"{"records": [
///     {"series_id": "SOFR", "date": "2026-08-14", "value": 4.31,
///      "unit": "Percent", "frequency": "daily"}
/// ]}"#;
/// let observations = parse_fred_observations(input)?;
/// assert_eq!(observations[0].series_id, "SOFR");
/// # Ok::<(), fredx::FredError>(())
/// ```
///
/// # Errors
///
/// - 输入不是合法 JSON 对象 / 含未知字段 / 缺必需字段 → [`FredError::Invalid`]
/// - `series_id` 不在清单登记范围内、值形态非法、单位或频率与清单不符 → [`FredError::SemanticallyRejected`]
/// - 同一 `(series_id, 期间, vintage)` 出现两次 → [`FredError::SemanticallyRejected`]
pub fn parse_fred_observations(input: &str) -> FredResult<Vec<FredObservation>> {
    let envelope: RawEnvelope = serde_json::from_str(input).map_err(json_error)?;
    let mut observations = Vec::with_capacity(envelope.records.len());
    let mut seen: Vec<(String, Period, Option<Date>)> = Vec::with_capacity(envelope.records.len());
    for raw in &envelope.records {
        let observation = build_observation(raw)?;
        let identity = (
            observation.series_id.clone(),
            observation.period,
            observation.vintage,
        );
        if seen.contains(&identity) {
            return Err(FredError::SemanticallyRejected(format!(
                "重复身份（series_id + 期间 + vintage）：{}",
                observation.series_id
            )));
        }
        seen.push(identity);
        observations.push(observation);
    }
    Ok(observations)
}

/// 把一条原始记录转成观测对象，并逐层执行守卫。
fn build_observation(raw: &RawRecord) -> FredResult<FredObservation> {
    ensure_product_local(FredProduct::Observation)?;
    ensure_known_series(&raw.series_id)?;
    ensure_baml_freeze(&raw.series_id)?;
    let frequency = Frequency::parse(&raw.frequency)?;
    let date = Date::parse(&raw.date)?;
    let period = project_period(frequency, date)?;
    let observation = FredObservation {
        series_id: raw.series_id.clone(),
        period,
        value: parse_value(&raw.value)?,
        unit: FredUnit::new(&raw.unit)?,
        frequency,
        vintage: match raw.vintage.as_deref() {
            Some(text) => Some(Date::parse(text)?),
            None => None,
        },
    };
    validate_observation(&observation)?;
    ensure_source_unit(&observation)?;
    ensure_source_frequency(&observation)?;
    Ok(observation)
}

/// 按清单频率把源日期投影为业务期间。
///
/// 这是**期间投影**（源日期 → 该频率的期间身份），不是派生指标。
fn project_period(frequency: Frequency, date: Date) -> FredResult<Period> {
    match frequency {
        Frequency::Daily | Frequency::Weekly | Frequency::Irregular => Ok(Period::Day(date)),
        Frequency::Monthly => Ok(Period::Month {
            year: date.year,
            month: date.month,
        }),
        Frequency::Quarterly => Ok(Period::Quarter {
            year: date.year,
            quarter: quarter_of(date.month)?,
        }),
        Frequency::Annual => Ok(Period::Year(date.year)),
        Frequency::Event => Ok(Period::Event { date }),
    }
}

/// 月 → 季；月份越界时返回 [`FredError::Invalid`]。
fn quarter_of(month: u8) -> FredResult<u8> {
    if !(1..=12).contains(&month) {
        return Err(FredError::Invalid("月份须在 1..=12".into()));
    }
    Ok((month - 1) / 3 + 1)
}

/// 解析值形态：数值 → `Present`；`.` 或 `null` → 具名缺失。
fn parse_value(raw: &serde_json::Value) -> FredResult<FredValue> {
    match raw {
        serde_json::Value::Number(number) => match number.as_f64() {
            Some(v) => Ok(FredValue::Present(v)),
            None => Err(FredError::Invalid("观测值超出 f64 可表示范围".into())),
        },
        serde_json::Value::String(text) if text == "." => {
            Ok(FredValue::Missing(FredMissingReason::NoObservation))
        }
        serde_json::Value::Null => Ok(FredValue::Missing(FredMissingReason::NoObservation)),
        _ => Err(FredError::Invalid(
            "观测值只能为数值、`.` 或 null（MUST NOT 把缺失折算为 0）".into(),
        )),
    }
}

/// 把 JSON 解析错误映射为不泄漏输入的 [`FredError::Invalid`]。
fn json_error(err: serde_json::Error) -> FredError {
    FredError::Invalid(format!(
        "离线输入不是合法的 JSON 形态（第 {} 行第 {} 列）",
        err.line(),
        err.column()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const WELL_FORMED: &str = r#"{
        "_synthetic": true,
        "_note": "合成样本",
        "records": [
            {"series_id": "WALCL", "date": "2026-08-13", "value": 7000000.0,
             "unit": "Millions of U.S. Dollars", "frequency": "weekly"},
            {"series_id": "RRPONTSYD", "date": "2026-08-14", "value": 450.25,
             "unit": "Billions of U.S. Dollars", "frequency": "daily"}
        ]
    }"#;

    #[test]
    fn parses_well_formed_input() {
        let observations = parse_fred_observations(WELL_FORMED).expect("应可解析");
        assert_eq!(observations.len(), 2);
        assert_eq!(observations[0].series_id, "WALCL");
        assert_eq!(observations[0].frequency, Frequency::Weekly);
        assert_eq!(observations[1].value.as_f64(), Some(450.25));
        assert_eq!(
            observations[1].period,
            Period::Day(Date::new(2026, 8, 14).expect("日期合法"))
        );
        assert!(observations[0].vintage.is_none());
    }

    #[test]
    fn monthly_and_quarterly_records_project_to_periods() {
        let input = r#"{"records": [
            {"series_id": "PCEPILFE", "date": "2026-06-30", "value": 1.0,
             "unit": "Index", "frequency": "monthly"},
            {"series_id": "FYFSGDA188S", "date": "2026-06-30", "value": -6.5,
             "unit": "Percent", "frequency": "quarterly"},
            {"series_id": "BOGZ1FL662090005Q", "date": "2026-03-31", "value": 1.0,
             "unit": "Index", "frequency": "quarterly"}
        ]}"#;
        let observations = parse_fred_observations(input).expect("应可解析");
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
    }

    #[test]
    fn missing_value_is_named_and_not_zero() {
        let input = r#"{"records": [
            {"series_id": "SOFR", "date": "2026-08-14", "value": ".", "unit": "Percent",
             "frequency": "daily"},
            {"series_id": "DFF", "date": "2026-08-14", "value": null, "unit": "Percent",
             "frequency": "daily"}
        ]}"#;
        let observations = parse_fred_observations(input).expect("应可解析");
        for observation in &observations {
            assert!(observation.value.is_missing());
            assert_eq!(observation.value.as_f64(), None);
            assert_ne!(observation.value.as_f64(), Some(0.0));
        }
    }

    #[test]
    fn vintage_is_preserved_when_present() {
        let input = r#"{"records": [
            {"series_id": "UNRATE", "date": "2026-06-30", "value": 4.2, "unit": "Percent",
             "frequency": "monthly", "vintage": "2026-07-03"}
        ]}"#;
        let observations = parse_fred_observations(input).expect("应可解析");
        assert_eq!(
            observations[0].vintage,
            Some(Date::new(2026, 7, 3).expect("日期合法"))
        );
    }

    #[test]
    fn unknown_field_is_rejected_atomically() {
        let input = r#"{"records": [
            {"series_id": "SOFR", "date": "2026-08-14", "value": 1.0, "unit": "Percent",
             "frequency": "daily", "endpoint": "x"}
        ]}"#;
        assert!(parse_fred_observations(input).is_err());

        let top = r#"{"records": [], "token": "x"}"#;
        assert!(parse_fred_observations(top).is_err());
    }

    #[test]
    fn missing_required_field_is_rejected() {
        let input = r#"{"records": [
            {"date": "2026-08-14", "value": 1.0, "unit": "Percent", "frequency": "daily"}
        ]}"#;
        assert!(parse_fred_observations(input).is_err());
    }

    #[test]
    fn illegal_date_and_value_forms_are_rejected() {
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
            assert!(parse_fred_observations(&input).is_err(), "{bad_date}");
        }
        let bad_value = r#"{"records": [
            {"series_id": "SOFR", "date": "2026-08-14", "value": "n/a", "unit": "Percent",
             "frequency": "daily"}
        ]}"#;
        assert!(parse_fred_observations(bad_value).is_err());
    }

    #[test]
    fn duplicate_identity_is_rejected() {
        let input = r#"{"records": [
            {"series_id": "SOFR", "date": "2026-08-14", "value": 1.0, "unit": "Percent",
             "frequency": "daily"},
            {"series_id": "SOFR", "date": "2026-08-14", "value": 1.1, "unit": "Percent",
             "frequency": "daily"}
        ]}"#;
        let err = parse_fred_observations(input).expect_err("应拒绝重复身份");
        assert_eq!(err.kind(), crate::FredErrorKind::SemanticallyRejected);
    }

    #[test]
    fn out_of_scope_series_and_baml_violations_are_rejected() {
        let out_of_scope = r#"{"records": [
            {"series_id": "CPIAUCSL", "date": "2026-06-30", "value": 1.0, "unit": "Index",
             "frequency": "monthly"}
        ]}"#;
        assert!(parse_fred_observations(out_of_scope).is_err());

        let baml = r#"{"records": [
            {"series_id": "BAMLC0A0CM", "date": "2026-08-14", "value": 1.0, "unit": "Percent",
             "frequency": "daily"}
        ]}"#;
        assert!(parse_fred_observations(baml).is_err());
    }

    #[test]
    fn unit_and_frequency_drift_are_rejected() {
        let wrong_unit = r#"{"records": [
            {"series_id": "RRPONTSYD", "date": "2026-08-14", "value": 450.25,
             "unit": "Millions of U.S. Dollars", "frequency": "daily"}
        ]}"#;
        assert!(parse_fred_observations(wrong_unit).is_err());

        let wrong_frequency = r#"{"records": [
            {"series_id": "WALCL", "date": "2026-08-13", "value": 1.0,
             "unit": "Millions of U.S. Dollars", "frequency": "daily"}
        ]}"#;
        assert!(parse_fred_observations(wrong_frequency).is_err());
    }

    #[test]
    fn malformed_json_reports_position_without_echoing_input() {
        let err = parse_fred_observations("{ not json").expect_err("应拒绝");
        let shown = err.to_string();
        assert!(!shown.contains("not json"));
        assert!(shown.contains("第"));
    }

    #[test]
    fn empty_records_yield_empty_collection() {
        let observations = parse_fred_observations(r#"{"records": []}"#).expect("应可解析");
        assert!(observations.is_empty());
    }
}
