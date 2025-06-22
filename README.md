<<<<<<< HEAD
# High-Frequency-Trading-Patterns
```markdown
# High-Frequency-Trading Patterns in Rust
=======
<<<<<<< HEAD
# High-Frequency-Trading
=======
# High-Frequency-Trading-Patterns
```markdown
# High-Frequency-Trading Patterns&nbsp;in Rust
>>>>>>> feb3bf6 (statistical arbitrage)

A hands-on playground that **implements canonical HFT play-books in pure Rust**, one pattern per crate (or binary).  
The goal is to let researchers, quants, and curious Rustaceans *clone → `cargo run` → inspect* each strategy with zero infrastructure pain.

---

## ✨ Patterns roadmap

| Stage | Pattern family | Rust crate/binary | Status |
|-------|---------------|-------------------|--------|
| **L2 Liquidity** | Passive / Inventory Market-Making | `hft-passive-mm` | **✅ Ready** |
| **L2 Liquidity** | Latency-Arb Liquidity Sniping | `hft-latency-arb` | ⏳ (next) |
| **L3 Arb** | Index / ETF Futures–Cash Arb | `hft-index-arb` | 📋 design |
| **L3 Stats** | Multi-asset StatArb | `hft-statarb` | 📋 design |
| **L4 Event** | Real-time NLP News Trader | `hft-news-algo` | 🧪 PoC |
| **L5 AI** | Order-Book Deep-Learning Micro-Predictor | `hft-lob-dl` | 🧪 PoC |
| **⚠︎ Manipulative** | Momentum Ignition, Spoofing, Quote-Stuffing | *educational-only* | 🔒 gated |

*This repository will never ship live-venue keys, credentials, or manipulative code activated by default.*

---

## 🏗️ Current focus — `hft-passive-mm`

*Single-binary* inventory-skew market maker + toy limit-order-book simulator.

```

├── Cargo.toml
├── Config.toml           # strategy knob file
└── src/
├── main.rs           # orchestration + Tokio runtime
├── config.rs         # Figment-backed loader
├── models.rs         # Tick / Order / Fill DTOs
├── exchange.rs       # async in-process book
├── strategy.rs       # inventory market-maker
└── risk.rs           # position + rate limits

````

### Quick start

```bash
git clone https://github.com/your-handle/hft-patterns-rust
cd hft-patterns-rust/hft-passive-mm
cargo run --release
````

Sample log:

```
FILL Buy  qty=1 px=99.92  inv= 1  pnl=-99.92
FILL Sell qty=1 px=100.18 inv= 0  pnl=-0.26
```

### Tuning knobs (`Config.toml`)

| Param             | Meaning                     | Default |
| ----------------- | --------------------------- | ------- |
| `half_spread`     | Base half-tick in contracts | `0.25`  |
| `size`            | Contracts per quote leg     | `1.0`   |
| `inv_limit`       | Max ± inventory             | `10.0`  |
| `inv_spread_mult` | Extra spread when at limit  | `2.0`   |
| `tick_ms`         | Sim market-data interval    | `50`    |

---

## 📐 Architecture snapshot

```text
┌──────────┐ ticks  ┌─────────────┐ orders ┌──────────┐ fills ┌─────────┐
│ exchange │──────▶│  strategy   │────────▶│  risk    │──────▶│ log/PnL │
└──────────┘        └─────────────┘        └──────────┘       └─────────┘
(random LOB)        (inventory MM)          (limits)
```

* All async channels are `tokio::mpsc`; no unsafe code required.
* Remove the included exchange sim and plug real websockets/REST for live trading.

---

## 🔌 Extending

1. **Multiple symbols** – spin one `InventoryMm` per symbol; share a single `Risk`.
2. **External feed** – swap `exchange.rs` for Binance/Polygon/Bybit WS adapter.
3. **Observability** – add `tracing-opentelemetry`, scrape with Prom/Grafana.
4. **Kubernetes** – containerise each pattern, then layer L4/L5 service-mesh patterns (circuit breakers, distributed tracing) as per your micro-services experiments.

---

## About

*No description, website, or topics provided.*
(Feel free to open a PR improving this section.)

---

## Resources

* **Readme** – this file
* **License** – MIT (see below)

---

## License

```text
MIT License

Copyright (c) 2025 Paul Seng

Permission is hereby granted, free of charge, to any person obtaining a copy
...
```

*FULL LICENSE TEXT in [`LICENSE`](LICENSE).*

---

## Activity (GitHub snapshot)

| Metric      | Value |
| ----------- | ----- |
| ⭐ Stars     | 0     |
| 👀 Watchers | 0     |
| 🍴 Forks    | 0     |

> Your ⭐️, watch, or fork will appear here automatically once the repository is public.

---

Happy market-making!  Pull requests, issue reports, and battle-stories are very welcome.

```
```
<<<<<<< HEAD
=======
>>>>>>> 051ca2b (Initial commit of HFT patterns project)
>>>>>>> feb3bf6 (statistical arbitrage)
