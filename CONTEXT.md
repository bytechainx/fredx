# fredx 上下文

本文件定义 `fredx` 与其使用方共享的词汇与边界。它记录领域含义与能力边界，
不记录具体实现细节、部署决定或存储方案。

## 角色与边界

**源事实库**：把 `specs/adapter/fred.md`（source_id = `fred`）声明的 FRED 采集范围
落成类型化常量与守卫的离线库。
_Avoid_: 采集器 / 抓取器（本库不含任何网络能力）

**离线解析**：只吃字符串的解析入口，把自有 JSON 形态转成观测集合。
_Avoid_: API 客户端（本库不接受 URL、认证信息或任何网络参数）

**守卫**：跨源约束在本库内**自己那一侧**的落点（禁静默替换、冻结令、写入主权、曲线边界）。
跨源整体语义归 `specs/features/005-macro-data-source-crates/contracts/cross-source-routing.md`。
_Avoid_: 权威裁定者（本库 MUST NOT 重新裁定任何未决项）

**共享形状**：错误面、值对象与 publication 三元组在各数据源库中**各自实现一遍**，
以文档 `contracts/source-library-contract.md` 冻结形状。
_Avoid_: 公共 core crate（宪章原则 III 禁止第二个共享契约 crate）

## 源事实语义

**权威写入主权**（`AUTHORITATIVE_WRITE_SERIES`）：`WALCL` / `WTREGEN` / `RRPONTSYD` / `WRESBAL`
四条序列的权威 Observation 唯一写入方是本域（Owner 联裁 `WALCL-AUTHORITY-2026-08-20-A`）。
其它源只可**对账告警**，MUST NOT 覆盖权威值或双写 canonical key。
_Avoid_: 校验面（`treasuryx` 的 FD01–FD04 是永久校验面，只告警不写入）

**曲线输入点**：`DGS*` / `T10Y*` 是曲线**输入**，本库可持有其源观测；
**曲线构建**（期限结构对象）归 `yieldx`。
_Avoid_: 曲线权威（本域是源，不是曲线 owner）

**近义非同 ID**：语义不同、MUST NOT 互为别名的 ID 对（`DFF`≠`EFFR`、`SP500`≠`SPY` …）。
本库登记 `cross-source-routing.md` §4 中涉及 fred 的 14 对。
_Avoid_: 别名表（别名会把两个不同语义压成一个身份）

**源单位保留**：观测携带源侧单位字面量，本层**不做换算**。
`RRPONTSYD` 是 Billions、`WALCL` / `WTREGEN` 是 Millions，两者量级不同。
_Avoid_: 规范化单位（Canonical 转换归下游 Normalize）

**具名缺失**：缺失以 `FredMissingReason` 表达，MUST NOT 静默折算为 `0`。
_Avoid_: 零值（0 是一个合法观测，与缺失不是同一件事）

**推断层 publication**：FRED 标准 API 的 `realtime_start` 是日期而非时刻，离线层无
ALFRED vintage，故 publication 时刻属推断层，正式 PIT 资格为 `NotEligible`。
_Avoid_: 正式 PIT（须走 ALFRED vintage，属 live 阶段）

## 授权语义

**授权判定**：一次 fail-closed 的只读结论（offline / live 是否被证据覆盖）。
它**不改写** `specs/adapter/fred.owner-approve.json` 的登记值，也不表示本库生产就绪。
_Avoid_: 生产就绪（`production_decision` 恒为 `NO-GO`，与 offline 范围被授权共存不矛盾）

**已登记证据**：`FRED-PROD-2026-08-15-approve`（签署者 `ZoneCNH`），覆盖 offline 范围；
同一 receipt 登记 `live = no-go`。
_Avoid_: live 授权（清单未授予，本库 MUST NOT 据此放行）

## 非目标（明确排除）

- 联网采集 / live / 官方 PIT（ALFRED vintage）
- 派生指标（净流动性、利差、Credit Impulse、z-score、Regime 状态）
- 单位换算、曲线构建、存储与分发
- 任何能力等级、SLA、新鲜度保证或生产就绪宣称

## `rust-version` 推导

规则：`rust-version` = 依赖图中所有依赖所声明 `rust_version` 的**最大值**
（`cargo metadata --format-version 1` 的 `rust_version` 字段）。

本 crate 的直接依赖与其 `rust_version`：

| 依赖 | 版本 | `rust_version` |
| --- | --- | --- |
| `thiserror` | 2.0.20 | 1.71 |
| `serde` | 1.0.229 | 1.56 |
| `serde_json` | 1.0.151 | 1.71 |

传递依赖：`itoa` 1.0.18（1.68）、`memchr` 2.8.3（1.61）、`proc-macro2` 1.0.107（1.71）、
`quote` 1.0.47（1.71）、`serde_core` 1.0.229（1.56）、`serde_derive` 1.0.229（1.71）、
`syn` 3.0.6（1.71）、`thiserror-impl` 2.0.20（1.71）、`unicode-ident` 1.0.26（1.71）、
`zmij` 1.0.23（1.71）。

最大值 = **1.71**，故 `rust-version = "1.71"`（2026-09-22 由 `cargo metadata` 实读推导）。
所用语言特性（`let … else`、内联格式参数、`const fn` 常量求值）下界均不高于 1.71。

## 已知缺口

| # | 缺口 | 说明 |
| --- | --- | --- |
| G1 | 无联网采集 | 本特性不实现；`production_decision = NO-GO` |
| G2 | 无官方 PIT / ALFRED vintage | publication 为推断层；`is_formal_pit_eligible()` 恒 `false` |
| G3 | 部分序列无源单位声明 | 清单只声明了 `WALCL` / `WTREGEN` / `RRPONTSYD` 的源单位，其余序列 `series_unit` 返回 `None`，不做单位判定 |
| G4 | `FDHBFRBN`「等」的全集未钉死 | 本库 MUST NOT 自行扩集（`cross-source-routing.md` §8 P5） |
| G5 | `GDPNow` 载体 `absent` | 载体待 Owner 钉死；MUST NOT 用 `GDP` 顶替（§8 P6） |
| G6 | `RESPPLLOPNWW` 归属未决 | 与 `WRESBAL` 的身份与权威归属待裁；本库一律拒绝主张权威写入 |
| G7 | 夹具为合成样本 | `tests/fixtures/` 全部自拟，不是真实源数据，不构成证据 |
