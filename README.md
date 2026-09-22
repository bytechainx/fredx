# fredx

`fredx` 是 FRED 源的**离线源事实库**：把 `specs/adapter/fred.md` 声明的采集范围落成代码 ——
series ID 常量与分组、源侧单位与频率、近义非同 ID 禁则、BAML 冻结令、写入主权、
曲线边界、publication 语义与一个只吃字符串的离线解析器。

- **不是联网采集器**：零 HTTP 客户端依赖、零端点字面量、零凭据读取
- **零内部耦合**：不依赖任何 `bytechainx/*` crate，公共形状与兄弟库各自实现一遍
- **零派生指标**：不做单位换算、不算净流动性 / 利差 / Credit Impulse / z-score
- 统一错误面：`FredErrorKind` 语义分类 + `FredError` / `FredResult<T>`
- fail-closed 授权判定：证据缺失 / 不明 / 过期 / 未覆盖一律拒绝

## 安装（引入方式）

本 crate **不发布到 crates.io**，以 git 依赖引入：

```toml
[dependencies]
fredx = { git = "https://github.com/bytechainx/fredx" }
```

本地开发可直接用路径依赖：

```toml
[dependencies]
fredx = { path = "../fredx" }
```

## 用法示例

```rust
use fredx::{parse_fred_observations, FredErrorKind};

let input = r#"{"_synthetic": true, "records": [
    {"series_id": "RRPONTSYD", "date": "2026-08-14", "value": 450.25,
     "unit": "Billions of U.S. Dollars", "frequency": "daily"}
]}"#;
let observations = parse_fred_observations(input)?;
assert_eq!(observations[0].series_id, "RRPONTSYD");

// 守卫：把 Billions 当 Millions 会被拒绝（本层不做换算）。
let wrong = r#"{"records": [
    {"series_id": "RRPONTSYD", "date": "2026-08-14", "value": 450.25,
     "unit": "Millions of U.S. Dollars", "frequency": "daily"}
]}"#;
assert_eq!(
    parse_fred_observations(wrong).unwrap_err().kind(),
    FredErrorKind::SemanticallyRejected
);
# Ok::<(), fredx::FredError>(())
```

### 跨源守卫

```rust
use fredx::{
    claim_authoritative_write, ensure_baml_freeze, ensure_not_silent_substitution, ensure_product_local,
    FredProduct,
};

// 权威写入只限本域的四条序列；曲线点与 BEA 表号一律拒绝。
assert!(claim_authoritative_write("WALCL").is_ok());
assert!(claim_authoritative_write("DGS10").is_err());
// 近义非同 ID 不得互换。
assert!(ensure_not_silent_substitution("DFF", "EFFR").is_err());
// BAML 冻结令。
assert!(ensure_baml_freeze("BAMLC0A0CM").is_err());
// 曲线构建归 yieldx。
assert!(ensure_product_local(FredProduct::YieldCurve).is_err());
```

## 同义非同 ID 与边界（摘要）

| 边界 | 处理 |
| --- | --- |
| `WALCL` / `WTREGEN` / `RRPONTSYD` / `WRESBAL` | 本域为权威 Observation **唯一写入方**；其它源只可对账告警 |
| `DGS*` / `T10Y*` | 可作**输入点**；曲线构建归 `yieldx`，本库拒绝 |
| 近义非同 ID（14 对） | 双向禁静默替换（`DFF`≠`EFFR`、`SP500`≠`SPY` …） |
| `BAMLH0A0HYM2` | 冻结令下唯一允许的 BAML 系列；`BAMLC0A0CM` 已 drop |
| `RRPONTSYD` 单位 | 源单位为 `Billions of U.S. Dollars`，与 `WALCL`/`WTREGEN` 的 Millions **量级不同**，本层不换算 |

## 非目标

- 不做联网采集：无 HTTP 客户端、无端点字面量、不读环境变量或凭据
- 不做派生指标（净流动性 / 利差 / Credit Impulse / z-score / Regime 状态归 analytics）
- 不做单位换算（归下游 Normalize）、不构建曲线（归 `yieldx`）
- 不做 live / ALFRED vintage、不做存储与分发
- 不宣称任何能力等级、SLA、新鲜度保证或生产就绪

## 门禁

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo package --no-verify
```

三大类测试随仓提供：`tests/tdd_contracts.rs`（TDD 行为契约）、`tests/sdd_spec.rs`
（与 `docs/标准.md` 章节 1:1）、`tests/aidd_boundary.rs`（对抗 / 边界用例）。
`tests/fixtures/` 下**全部为合成样本**，不是真实源数据，不构成任何证据。

**诚实边界**：`production_decision = NO-GO`；清单 COMPLETE ≠ ship；
authorization ≠ Production Ready。offline 范围被授权不代表 live 被授权。

## 许可

MIT OR Apache-2.0
