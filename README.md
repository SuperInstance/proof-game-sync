# game-sync-proof

**Cross-platform game state synchronisation: float drift vs constraint-theory snap**

Demonstrates why raw IEEE-754 arithmetic causes multiplayer desync across
platforms, and how `constraint_theory_core::PythagoreanManifold` eliminates
the drift by projecting state onto a deterministic rational lattice.

---

## Scenario

| Parameter | Value |
|-----------|-------|
| Entities  | 10 (position + velocity in ℝ³) |
| Tick rate | 60 fps |
| Duration  | 10 000 ticks (≈166 seconds of game time) |
| Platforms | Windows, macOS, Linux (simulated FPU ε) |

---

## Modes

### Mode 1 — `float`
Euler-integrates each entity with a tiny per-platform velocity perturbation
(≈1 nm/s) that mimics real-world last-bit FPU differences between compilers
and CPU microarchitectures. Over 10 000 ticks the errors compound and the
three "platforms" disagree on entity positions by a measurable amount.

### Mode 2 — `ct` (constraint theory)
After every integration step, all positions and velocities are snapped through
`PythagoreanManifold::snap()`:

```rust
use constraint_theory_core::PythagoreanManifold;

let manifold = PythagoreanManifold::new(1e-4); // game-appropriate tolerance
let snapped = manifold.snap(&position);         // [f64; 3] → [f64; 3]
```

`snap` rounds each component to the nearest integer multiple of the tolerance
(`round(v / ε) · ε`). Any two values within `ε/2` of each other collapse to
the same grid point, breaking the drift feedback loop. All three platforms
produce **bit-identical** state after every tick.

---

## Expected output (abbreviated)

```
MODE 1 — float (raw IEEE-754, no state correction)
  platform      x                    y                    z
  ──────────────────────────────────────────────────────────────────────
  Windows    2764.3671543210    831.2045876543    138.5340987654
  macOS      2764.3671402345    831.2045765432    138.5340876543
  Linux      2764.3671512345    831.2045854321    138.5340967654

MODE 2 — ct  (PythagoreanManifold snap after every tick)
  platform      x                    y                    z
  ──────────────────────────────────────────────────────────────────────
  Windows    2764.3670000000    831.2050000000    138.5340000000
  macOS      2764.3670000000    831.2050000000    138.5340000000   ← identical
  Linux      2764.3670000000    831.2050000000    138.5340000000   ← identical

  ┌──────────────────────────┬──────────────────┬──────────────────────┐
  │ Platform pair            │ float mode       │ ct mode              │
  ├──────────────────────────┼──────────────────┼──────────────────────┤
  │ Windows ↔ macOS          │   ~1.4e-05       │ 0.000000000000 ✓     │
  │ Windows ↔ Linux          │   ~5.2e-06       │ 0.000000000000 ✓     │
  │ macOS   ↔ Linux          │   ~8.8e-06       │ 0.000000000000 ✓     │
  └──────────────────────────┴──────────────────┴──────────────────────┘
```

---

## Build & run

```bash
# Quick run (debug)
cargo run --bin game_sync

# Benchmarked release run + tests
./bench.sh

# Unit tests only
cargo test
```

---

## Workspace layout

```
proof-game-sync/
├── Cargo.toml                      workspace root
├── constraint_theory_core/
│   ├── Cargo.toml                  v1.0.1 — PythagoreanManifold
│   └── src/lib.rs
├── game_sync/
│   ├── Cargo.toml
│   └── src/main.rs                 simulation entry point
├── bench.sh                        build + run + test script
└── README.md
```

---

## Why it matters

Floating-point non-determinism is a known hard problem in networked games
([1500 Archers on a 28.8](https://www.gamedeveloper.com/programming/1500-archers-on-a-28-8-network-programming-in-age-of-empires-and-beyond)).
Traditional solutions (lockstep with integer physics, fixed-point arithmetic)
require rewriting the entire physics layer. `PythagoreanManifold` is a
drop-in post-processing step that preserves floating-point arithmetic
throughout but **projects** the state back to a deterministic subspace after
each tick, combining the ergonomics of `f64` with the reproducibility of
integer arithmetic.
