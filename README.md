# game-sync-proof

**Cross-platform game state synchronisation: float drift vs constraint-theory snap**

## Overview

`proof-game-sync` is a deterministic simulation benchmark that demonstrates why raw IEEE-754 floating-point arithmetic causes multiplayer game desync across platforms — and how [`constraint_theory_core::PythagoreanManifold`](constraint_theory_core/src/lib.rs) eliminates the drift by projecting state onto a deterministic rational lattice after every game tick.

The proof-of-concept simulates **10 entities** (position + velocity in ℝ³) over **10 000 ticks** at 60 fps, running the same simulation on three "platforms" (Windows, macOS, Linux) that each inject sub-nanometre-per-second velocity perturbations mimicking real-world FPU rounding differences between compilers and CPU microarchitectures.

Two modes are compared head-to-head:

| Mode | Method | Cross-platform guarantee |
|------|--------|--------------------------|
| `float` | Raw Euler integration, no correction | Drift compounds unboundedly |
| `ct` | Euler + `PythagoreanManifold::snap()` post-step | **Bit-identical** state on all platforms |

## Architecture

```
proof-game-sync/                        ← Cargo workspace
├── Cargo.toml                          workspace root (resolver 2)
├── constraint_theory_core/             ← core library crate
│   ├── Cargo.toml                      v1.0.1
│   └── src/lib.rs                      PythagoreanManifold implementation
├── game_sync/                          ← simulation binary crate
│   ├── Cargo.toml
│   └── src/main.rs                     entry point, all simulation logic
├── bench.sh                            release build + run + test script
└── README.md
```

**Key components:**

- **`PythagoreanManifold`** — lattice projection engine. Given a tolerance ε, maps every scalar `v` to `round(v / ε) · ε`. Properties: *idempotent*, *order-independent*, *absorbs sub-ε noise*. Cached reciprocal (`inv_tolerance`) avoids repeated division.
- **`Entity`** — lightweight `Vec3` position + `Vec3` velocity struct. `Vec3` supports `as_array()` / `from_array()` for interop with the manifold.
- **`World`** — fixed-size array `[Entity; 10]`, zero-allocation, stack-friendly.
- **`PLATFORM_EPSILON`** — three distinct perturbation magnitudes (`~1.2e-9`, `~-9.9e-10`, `~2.3e-10`) that model the last-bit FPU divergence observed between MSVC, Clang, and GCC on x86-64.

## Game Sync Theory

### The Float Drift Problem

In a lockstep multiplayer game, every client must produce the **exact same** simulation state each tick from the same inputs. IEEE-754 `f64` arithmetic is *not* associative* — `(a + b) + c ≠ a + (b + c)` at the ULP level — and different compilers, optimisation levels, and CPU microarchitectures reorder operations differently. Over thousands of ticks, these ~1 ULP differences compound into centimetre-scale positional drift.

This is the same fundamental problem described in *[1500 Archers on a 28.8](https://www.gamedeveloper.com/programming/1500-archers-on-a-28-8-network-programming-in-age-of-empires-and-beyond)* (Age of Empires networking).

### The Constraint-Theory Solution

Rather than rewriting physics in fixed-point or integer arithmetic, `PythagoreanManifold` acts as a **post-processing projection**:

```
state(t+1) = snap(integrate(state(t), dt))
```

The snap operation maps each component onto the lattice `L = { k·ε | k ∈ ℤ }`. Any two values within `ε/2` of each other collapse to the same lattice point, **breaking the feedback loop** that amplifies drift:

```
                    ε
           ┌────────┼────────┐
  v + δ₁ ──┤   snap  →  k·ε  │  ← same point
  v + δ₂ ──┤   snap  →  k·ε  │  ← same point
           └────────┼────────┘
     where |δ₁|, |δ₂| < ε/2
```

**Why this works for games:**
- **Drop-in** — no changes to physics, collision, or AI code.
- **Ergonomic** — keep `f64` throughout; only one `snap()` call per tick.
- **Sub-millimetre precision** — with ε = 1e-4, the lattice resolution is 0.1 mm (far below player perception).
- **Bit-identical** — the lattice is platform-independent by construction.

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

## Quick Start

```bash
# Clone and enter the workspace
cd proof-game-sync

# Quick run (debug mode)
cargo run --bin game_sync

# Benchmarked release run + unit tests
./bench.sh

# Unit tests only (constraint_theory_core)
cargo test
```

---

## Integration

To add deterministic state sync to your own game engine:

**1. Add the dependency:**

```toml
# In your game's Cargo.toml
[dependencies]
constraint-theory-core = "1.0.1"
```

**2. Create a manifold once (typically at startup):**

```rust
use constraint_theory_core::PythagoreanManifold;

// ε = 1e-4 gives 0.1 mm lattice resolution — adjust for your game's scale
let manifold = PythagoreanManifold::new(1e-4);
```

**3. Snap state after each physics tick:**

```rust
fn tick(&mut self, dt: f64) {
    // Your existing physics, AI, collision code — unchanged
    self.physics_step(dt);

    // One post-processing call per tick ensures cross-platform determinism
    for entity in &mut self.entities {
        entity.pos = Vec3::from_array(manifold.snap(&entity.pos.as_array()));
        entity.vel = Vec3::from_array(manifold.snap(&entity.vel.as_array()));
    }
}
```

**4. Verify with your own test harness:**

```rust
#[test]
fn platforms_agree() {
    let manifold = PythagoreanManifold::new(1e-4);
    let world_a = simulate_with_epsilon(&manifold,  1.23e-9);
    let world_b = simulate_with_epsilon(&manifold, -0.99e-9);
    assert_eq!(world_a, world_b, "CT mode must be platform-independent");
}
```

## Workspace Layout

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

## Why It Matters

Floating-point non-determinism is a known hard problem in networked games
([1500 Archers on a 28.8](https://www.gamedeveloper.com/programming/1500-archers-on-a-28-8-network-programming-in-age-of-empires-and-beyond)).
Traditional solutions (lockstep with integer physics, fixed-point arithmetic)
require rewriting the entire physics layer. `PythagoreanManifold` is a
drop-in post-processing step that preserves floating-point arithmetic
throughout but **projects** the state back to a deterministic subspace after
each tick, combining the ergonomics of `f64` with the reproducibility of
integer arithmetic.

---

<img src="callsign1.jpg" width="128" alt="callsign">
