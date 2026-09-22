# fredx 公开 API

本文对应 `fredx 0.1.0` 的离线公开消费面。全部入口**不触网**、不读凭据、不做单位换算。

## 值对象与校验

| 类型 / 入口 | 语义 |
| --- | --- |
| `Date` | 严格 ISO 日期 `YYYY-MM-DD`（`year` / `month` / `day` 公开字段） |
| `Date::new` | 构造并校验日期（月份范围 + 实际天数 + 闰年） |
| `Date::parse` | 按严格 ISO 解析；拒绝未补零、非 `-` 分隔、带时间部分者 |
| `Date::is_leap_year` | 格里高利闰年判定 |
| `Date::days_in_month` | 该年该月天数；月份非法返回 `0`（不 panic） |
| `Period` | 业务期间：`Day` / `Month` / `Quarter` / `Year` / `Event` |
| `Frequency` | 源侧频率七值；`Frequency::parse` / `Frequency::as_str` 为稳定记号 |
| `FredUnit` | 源侧单位（开放 newtype）；`FredUnit::new` / `FredUnit::as_str` |
| `FredMissingReason` | 具名缺失原因（`NoObservation`） |
| `FredValue` | `Present(f64)` / `Missing(FredMissingReason)`；`as_f64` / `is_missing` |
| `FredObservation` | 一条源事实观测：`series_id` + `period` + `value` + `unit` + `frequency` + `vintage` |
| `validate_date` / `validate_period` / `validate_observation` | 值对象完整性校验，返回 `FredResult<()>` |

## 源事实常量

| 入口 | 语义 |
| --- | --- |
| 35 个 series ID 常量 | 清单明确列出的纯 FRED ID（`fredx::WALCL`、`fredx::DFII10` …） |
| `WRESBAL` / `BAMLC0A0CM` / `RESPPLLOPNWW` | 非采集登记：mapping-only / 已 drop / 归属未决 |
| `P0_SERIES` `CURVE_SERIES` `FUNDAMENTAL_SERIES` `GLOBAL_REGIONAL_SERIES` `BLS_FORWARD_SERIES` | 分组常量集 |
| `ALL_SERIES` | 采集集合全集（35 个） |
| `AUTHORITATIVE_WRITE_SERIES` | 权威写入唯一归本域的四条序列 |
| `BAML_ALLOWED_SERIES` | 冻结令下允许的唯一 BAML 系列 |
| `UNIT_MILLIONS_OF_USD` / `UNIT_BILLIONS_OF_USD` | 清单声明的两个源单位字面量 |
| `is_collected_series` / `is_mapping_only_series` / `is_known_series` | 范围判定 |
| `is_curve_input_series` | `DGS*` / `T10Y*` 曲线输入点判定 |
| `series_unit` / `series_frequency` | 清单声明的源单位 / 频率（未声明返回 `None`） |
| `is_unique_write_authority` | 该 series 的权威写入是否唯一归本域 |

## 校验与守卫

| 入口 | 语义 |
| --- | --- |
| `ensure_known_series` | 拒绝清单登记范围外的 series ID |
| `ensure_source_unit` | 源单位一致性（只拒绝、不换算） |
| `ensure_source_frequency` | 清单频率一致性 |
| `NEAR_SYNONYM_PAIRS` / `is_near_synonym_pair` | 涉 FRED 的 14 对近义非同 ID |
| `ensure_not_silent_substitution` | 双向禁静默替换 |
| `ensure_baml_freeze` | BAML 冻结令（只允许 `BAMLH0A0HYM2`） |
| `claim_authoritative_write` | 越权写入拒绝（只读判定） |
| `is_bea_table_id` / `BEA_TABLE_OWNER` | BEA 表号形态与归属方 |
| `FredProduct` / `ensure_product_local` / `CURVE_BUILD_OWNER` | 曲线构建归 `yieldx` |

## publication 语义

| 入口 | 语义 |
| --- | --- |
| `TimePrecision` / `AvailabilityEvidence` / `PitEligibility` | 三元组枚举 |
| `FredPublicationSemantics` / `fred_publication_semantics` | 恒为 `(Date, Inferred, NotEligible)` |
| `is_formal_pit_eligible` | 恒为 `false` |

## 授权判定

| 入口 | 语义 |
| --- | --- |
| `FredAccessMode` | `Offline` / `Live` |
| `FredAuthorization` | `Authorized { scope }` / `Denied { reason }` |
| `FredAuthorizationEvidence` | 只读证据登记（编号 / 签署者 / 日期 / 覆盖模式 / 范围说明） |
| `documented_fred_evidence` | 已登记证据 `FRED-PROD-2026-08-15-approve`（仅 offline） |
| `authorize_fred` | fail-closed 判定（缺失 / 不明 / 过期 / 未覆盖一律拒绝） |
| `validate_authorization_evidence` | 证据日期分量校验 |
| `mode_label` | 访问模式的稳定记号 |
| `FRED_DECISION_ID` / `FRED_SIGNED_BY` | 已登记签核编号与签署者 |

## 离线解析

| 入口 | 语义 |
| --- | --- |
| `parse_fred_observations` | 自有 JSON 形态 → `Vec<FredObservation>`；未知字段原子失败、重复身份拒绝 |

## 错误面

| 入口 | 语义 |
| --- | --- |
| `FredErrorKind` | 八类语义分类（`Invalid` / `Missing` / `AuthorizationDenied` / `RoutedElsewhere` / `WriteAuthorityDenied` / `SemanticallyRejected` / `NotApplicable` / `Invariant`） |
| `FredError` / `FredError::kind` / `FredError::is_retryable` | 错误类型与分类/重试判定（仅 `Invariant` 具重试标记） |
| `FredResult<T>` | 统一结果别名 |
