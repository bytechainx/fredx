# 变更记录

本文件记录 `fredx` 的可见变更。格式对齐 Keep a Changelog，
版本号遵循 `docs/versioning.md`；本仓库是库 crate，`Cargo.lock` 不入库。

## [Unreleased]

### 新增

- 新增 E2E target `tests/e2e_fred.rs`：把本仓**核对器口径内的全部公开条目**逐条真实执行 ——
  权威公开面派生自 `cargo +nightly public-api --simplified`，核对器（带 `llvm-cov`）退出码 **0**，
  权威 **171** / 声明 **171** / `公开 fn 执行 38/38`（分项 `type` 18 / `variant` 44 / `field` 18 /
  `const` 53 / `fn` 38）；单一 `#[test] e2e_fred_all_public_api`，`[dev-dependencies]` 仍为空。
  **纯测试新增，不改公开 API、不升版本**；口径边界（两级嵌套字段未登记 ⇒ 三层判据不保护、
  derive/auto impl 不计入等）与核对命令见 `AGENTS.md`「E2E 公开面覆盖核对（核对器口径）」。

## [0.1.1] - 2026-09-23

### 修正

- 修正授权元数据与日期有效区间校验，恢复 fail-closed 契约

## [0.1.0] - 2026-09-22

### 新增

- 源事实常量：35 个纯 FRED series ID（P0 13 / 利率与曲线 4 / 基本面 4 / 全球区域 12 /
  BLS 转发闭合 2）与分组常量集、`WRESBAL`（mapping-only）、`BAMLC0A0CM`（已 drop）、
  `RESPPLLOPNWW`（归属未决）登记。
- 值对象与校验：`Date`（严格 ISO + 闰年）、`Period`、`Frequency`、`FredUnit`、`FredValue`
  （具名缺失）、`FredObservation`，以及 `validate_date` / `validate_period` / `validate_observation`。
- 跨源守卫（本域自己那一侧）：14 对近义非同 ID 双向禁静默替换、BAML 冻结令、
  权威写入主权判定与越权写入拒绝、曲线构建路由 `yieldx`。
- 源单位一致性守卫：`WALCL`/`WTREGEN`（Millions）与 `RRPONTSYD`（Billions）**只拒绝、不换算**。
- publication 语义三元组：恒为 `(Date, Inferred, NotEligible)`；`is_formal_pit_eligible()` 恒为 `false`。
- fail-closed 授权判定：`FredAuthorization` / `authorize_fred`；offline 范围被 `FRED-PROD-2026-08-15-approve`
  覆盖，live 不被覆盖。
- 离线解析：`parse_fred_observations`（自有 JSON 形态，未知字段原子失败、重复身份拒绝）。
- 三类测试（`tests/tdd_contracts.rs` / `tests/sdd_spec.rs` / `tests/aidd_boundary.rs`）、
  合成夹具、手写微基准与文档面（README / `docs/API.md` / `docs/标准.md` / `CONTEXT.md`）。

### 边界

- `production_decision = NO-GO`：本版本不含联网采集、live、官方 PIT、单位换算或派生指标。
