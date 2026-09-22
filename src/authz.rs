//! fredx 的授权判定（fail-closed）。
//!
//! 判定是**只读**结论：它不改写 `specs/adapter/fred.owner-approve.json` 的任何登记值，
//! 也不表示本库生产就绪（`production_decision = NO-GO` 与「该源的 offline 范围被授权」
//! 共存不矛盾）。
//!
//! 判定输入是调用方给出的证据描述与评估日期（**无墙钟**）；证据缺失 / 签核编号不明 /
//! 签署者不明 / 覆盖范围不明 / 已过期 → 一律 [`FredAuthorization::Denied`]。

use crate::error::FredResult;
use crate::value::{validate_date, Date};

/// Owner 签核编号（`specs/adapter/fred.owner-approve.json`）。
pub const FRED_DECISION_ID: &str = "FRED-PROD-2026-08-15-approve";

/// 签署者。
pub const FRED_SIGNED_BY: &str = "ZoneCNH";

/// 访问模式：本域证据只覆盖离线范围。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FredAccessMode {
    /// 离线解析与合成夹具。
    Offline,
    /// live 联网访问：本特性不实现，**且未被证据覆盖**。
    Live,
}

/// 授权判定结果。
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FredAuthorization {
    /// 证据有效且覆盖本次请求的范围与有效期。
    Authorized {
        /// 被覆盖的源 / 模式 / 用途 / 有效期。
        scope: String,
    },
    /// 证据缺失 / 过期 / 签署者不明 / 覆盖范围不明。
    Denied {
        /// 可读的拒绝理由。
        reason: String,
    },
}

/// 只读的授权证据登记。
///
/// 本结构**只承载证据描述**；它不能被用来「默认放行」——判定入口
/// [`authorize_fred`] 对缺失与不明一律拒绝。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FredAuthorizationEvidence {
    /// Owner 签核编号。
    pub decision_id: String,
    /// 签署者。
    pub signed_by: String,
    /// 签核时间（证据登记值）。
    pub signed_at: Option<Date>,
    /// 有效期上界；`None` 表示证据未声明有效期。
    pub valid_until: Option<Date>,
    /// 被覆盖的访问模式集合。
    pub authorized_modes: Vec<FredAccessMode>,
    /// 范围说明（人类可读）。
    pub scope_note: String,
}

/// 清单已登记的 fred 证据（`FRED-PROD-2026-08-15-approve`，offline 范围）。
///
/// 该证据的 `receipt` 同时登记 `live = no-go`，故 `authorized_modes` 只有
/// [`FredAccessMode::Offline`]。
#[must_use]
pub fn documented_fred_evidence() -> FredAuthorizationEvidence {
    FredAuthorizationEvidence {
        decision_id: FRED_DECISION_ID.to_owned(),
        signed_by: FRED_SIGNED_BY.to_owned(),
        signed_at: Date::new(2026, 8, 15).ok(),
        valid_until: None,
        authorized_modes: vec![FredAccessMode::Offline],
        scope_note: "FRED 域 offline 范围（live = no-go）".to_owned(),
    }
}

/// fail-closed 授权判定。
///
/// 判据（任一不满足即 `Denied`）：
///
/// 1. 证据存在且非空
/// 2. 签核编号非空（签署者不明）
/// 3. 签署者非空（签署者不明）
/// 4. 覆盖模式集合非空（覆盖范围不明）
/// 5. 证据未过期（`valid_until` 已声明时，`as_of` MUST NOT 晚于它）
/// 6. 请求模式 ∈ 覆盖模式集合
///
/// `as_of` 由调用方传入，本层不读取系统时间。
#[must_use]
pub fn authorize_fred(
    evidence: Option<&FredAuthorizationEvidence>,
    requested: FredAccessMode,
    as_of: Date,
) -> FredAuthorization {
    let Some(evidence) = evidence else {
        return FredAuthorization::Denied {
            reason: "缺少 Owner 签核证据".to_owned(),
        };
    };
    if evidence.decision_id.is_empty() {
        return FredAuthorization::Denied {
            reason: "签核编号不明".to_owned(),
        };
    }
    if evidence.signed_by.is_empty() {
        return FredAuthorization::Denied {
            reason: "签署者不明".to_owned(),
        };
    }
    if evidence.authorized_modes.is_empty() {
        return FredAuthorization::Denied {
            reason: "证据未声明任何被覆盖的范围".to_owned(),
        };
    }
    if let Some(valid_until) = evidence.valid_until {
        if as_of > valid_until {
            return FredAuthorization::Denied {
                reason: "证据已过期".to_owned(),
            };
        }
    }
    if !evidence.authorized_modes.contains(&requested) {
        return FredAuthorization::Denied {
            reason: format!("请求的访问模式未被证据覆盖（{}）", evidence.scope_note),
        };
    }
    FredAuthorization::Authorized {
        scope: format!(
            "{} / {} / {}",
            evidence.decision_id,
            evidence.scope_note,
            mode_label(requested)
        ),
    }
}

/// 模式的稳定记号。
#[must_use]
pub fn mode_label(mode: FredAccessMode) -> &'static str {
    match mode {
        FredAccessMode::Offline => "offline",
        FredAccessMode::Live => "live",
    }
}

/// 校验一份证据描述自身的形态（日期分量合法）。
///
/// # Errors
///
/// `signed_at` / `valid_until` 的日期分量非法时返回 [`crate::FredError::Invalid`]。
pub fn validate_authorization_evidence(evidence: &FredAuthorizationEvidence) -> FredResult<()> {
    if let Some(signed_at) = evidence.signed_at {
        validate_date(&signed_at)?;
    }
    if let Some(valid_until) = evidence.valid_until {
        validate_date(&valid_until)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn as_of() -> Date {
        Date::new(2026, 8, 20).expect("日期合法")
    }

    #[test]
    fn documented_evidence_authorizes_offline_only() {
        let evidence = documented_fred_evidence();
        assert!(validate_authorization_evidence(&evidence).is_ok());
        match authorize_fred(Some(&evidence), FredAccessMode::Offline, as_of()) {
            FredAuthorization::Authorized { scope } => {
                assert!(scope.contains(FRED_DECISION_ID));
                assert!(scope.contains("offline"));
            }
            other => panic!("应授权 offline，实得 {other:?}"),
        }
        assert!(matches!(
            authorize_fred(Some(&evidence), FredAccessMode::Live, as_of()),
            FredAuthorization::Denied { .. }
        ));
    }

    #[test]
    fn missing_evidence_is_denied() {
        match authorize_fred(None, FredAccessMode::Offline, as_of()) {
            FredAuthorization::Denied { reason } => {
                assert_eq!(reason, "缺少 Owner 签核证据");
            }
            other => panic!("应拒绝，实得 {other:?}"),
        }
    }

    #[test]
    fn unknown_scope_is_denied() {
        let mut evidence = documented_fred_evidence();
        evidence.authorized_modes.clear();
        assert!(matches!(
            authorize_fred(Some(&evidence), FredAccessMode::Offline, as_of()),
            FredAuthorization::Denied { .. }
        ));

        let mut evidence = documented_fred_evidence();
        evidence.decision_id = String::new();
        assert!(matches!(
            authorize_fred(Some(&evidence), FredAccessMode::Offline, as_of()),
            FredAuthorization::Denied { .. }
        ));

        let mut evidence = documented_fred_evidence();
        evidence.signed_by = String::new();
        assert!(matches!(
            authorize_fred(Some(&evidence), FredAccessMode::Offline, as_of()),
            FredAuthorization::Denied { .. }
        ));
    }

    #[test]
    fn expired_evidence_is_denied() {
        let mut evidence = documented_fred_evidence();
        evidence.valid_until = Date::new(2026, 8, 16).ok();
        match authorize_fred(Some(&evidence), FredAccessMode::Offline, as_of()) {
            FredAuthorization::Denied { reason } => assert_eq!(reason, "证据已过期"),
            other => panic!("应拒绝，实得 {other:?}"),
        }
        // 到期日当天仍算覆盖。
        assert!(matches!(
            authorize_fred(
                Some(&evidence),
                FredAccessMode::Offline,
                Date::new(2026, 8, 16).expect("日期合法")
            ),
            FredAuthorization::Authorized { .. }
        ));
    }

    #[test]
    fn invalid_evidence_dates_are_rejected() {
        let mut evidence = documented_fred_evidence();
        evidence.valid_until = Some(Date {
            year: 2026,
            month: 2,
            day: 30,
        });
        assert!(validate_authorization_evidence(&evidence).is_err());
    }
}
