//! fredx 的 publication 语义三元组（时间精度 + 可得性证据层 + 正式 PIT 资格）。
//!
//! FRED 标准 API 的 `realtime_start` 语义为**日期**而非时刻，离线 fixture 无发布时刻、
//! 无 ALFRED vintage，故 fred 的 publication 时刻属**推断层**：
//! 正式 PIT 路径须走 ALFRED vintage（live 阶段），本层 `NotEligible`。
//!
//! MUST NOT 补造 `00:00 UTC` 之类时刻把 `Date` 伪装成 `Instant`；日后若接入
//! ALFRED vintage 或已可证的 `available_at`，须先改清单与契约，**禁止静默升格**。

/// 时间精度：源只给日期还是给出时刻。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimePrecision {
    /// 只有日期。
    Date,
    /// 有明确时刻。
    Instant,
}

/// 可得性证据层：官方字段 > 日历 > 推断。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvailabilityEvidence {
    /// 官方字段。
    Official,
    /// 发布日历。
    Calendar,
    /// 推断。
    Inferred,
}

/// 正式 PIT 资格。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PitEligibility {
    /// 可进正式 PIT。
    Formal,
    /// 不可进正式 PIT。
    NotEligible,
}

/// fred 的 publication 语义三元组。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FredPublicationSemantics {
    /// 时间精度。
    pub time_precision: TimePrecision,
    /// 可得性证据层。
    pub availability: AvailabilityEvidence,
    /// 正式 PIT 资格。
    pub eligibility: PitEligibility,
}

/// 本源的 publication 语义：恒为 `(Date, Inferred, NotEligible)`。
#[must_use]
pub fn fred_publication_semantics() -> FredPublicationSemantics {
    FredPublicationSemantics {
        time_precision: TimePrecision::Date,
        availability: AvailabilityEvidence::Inferred,
        eligibility: PitEligibility::NotEligible,
    }
}

/// 正式 PIT 资格判定；本层**恒为 `false`**。
///
/// 对应清单的 `macro_fred::assemble_fred_observations`
/// （`is_formal_pit_eligible() == false`）口径。
#[must_use]
pub fn is_formal_pit_eligible() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publication_triple_is_date_inferred_not_eligible() {
        let semantics = fred_publication_semantics();
        assert_eq!(semantics.time_precision, TimePrecision::Date);
        assert_eq!(semantics.availability, AvailabilityEvidence::Inferred);
        assert_eq!(semantics.eligibility, PitEligibility::NotEligible);
    }

    #[test]
    fn formal_pit_is_pinned_to_not_eligible() {
        assert!(!is_formal_pit_eligible());
        assert_ne!(
            fred_publication_semantics().eligibility,
            PitEligibility::Formal
        );
    }

    #[test]
    fn precision_is_never_promoted_to_instant() {
        assert_ne!(
            fred_publication_semantics().time_precision,
            TimePrecision::Instant
        );
    }
}
