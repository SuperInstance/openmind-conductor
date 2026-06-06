# openmind-conductor

Multi-agent orchestration with shared muscle memory — the **prefrontal cortex** for coordinated agents.

## The Orchestra Metaphor

Think of a symphony orchestra. Each musician (agent) has their own instrument and muscle memory — years of practice that lets them play their part instinctively. But without a conductor, you get noise.

`openmind-conductor` is that conductor. It coordinates multiple agents, each carrying their own learned patterns, to work together on complex tasks.

### The Cast

| Component | Role | Analogy |
|-----------|------|---------|
| **Ensemble** | Collection of agents | The orchestra |
| **Score** | Pre-computed plan | The sheet music |
| **Baton** | Communication channel | The conductor's baton |
| **Measure** | Timing & sync | The tempo / BPM |
| **Harmony** | Conflict resolution | Voting on interpretation |
| **Trit** | Ternary digit (-1, 0, +1) | For / abstain / against |

## Quick Start

```rust
use openmind_conductor::*;

// Create an ensemble with agents that have muscle memory
let mut ensemble = ensemble::Ensemble::new();

let mut sensor_mem = memory::MuscleMemory::new();
sensor_mem.add_pattern("read_temperature", "temp", vec![trit::Trit::PlusOne], 1.0);
ensemble.add_agent("sensor", sensor_mem);

let mut motor_mem = memory::MuscleMemory::new();
motor_mem.add_pattern("fan_on", "motor", vec![trit::Trit::PlusOne], 1.0);
ensemble.add_agent("motor", motor_mem);

// Compose a score — the plan
let score = score::Score::builder()
    .step("sensor", "read_temperature", vec![], measure::Timing::Immediate)
    .branch(
        score::Condition::Gt(score::Reading::Last("sensor".into()), 25.0),
        vec![score::Step::new("motor", "fan_on", vec![])],
        vec![], // else: do nothing
    )
    .build();

// Execute
let results = ensemble.execute_score_direct(&score).unwrap();
```

## Architecture

### Ensemble

An `Ensemble` holds named agents, each with their own `MuscleMemory`. You can:
- `add_agent(id, memory)` — register an agent
- `conduct(intent)` — simple intent-based execution
- `execute_score_direct(score)` — score-based orchestration

### Score

A `Score` is a sequence of instructions:

- **Steps** — do something on an agent
- **Branches** — conditional logic (if/else)
- **Parallel** — concurrent execution groups
- **Fermata** — wait for a condition

Scores serialize to/from JSON for persistence and transfer.

### Baton

Communication channels between conductor and agents:

- `LocalBaton` — in-process (tokio mpsc channels)
- `MockBaton` — for testing (records sent messages)

### Measure

Timing and synchronization:

- **BPM** — how often the conductor checks state
- **Downbeat** — the main loop tick
- **Swing** — off-beat timing for async ops
- **Fermata** — pause until a condition is met

### Harmony

When agents disagree, `Harmony` resolves conflicts:

- **Majority** — most votes win
- **Consensus** — any dissent blocks
- **Weighted** — votes scaled by agent weight

Uses ternary voting: `Trit::PlusOne` (for), `Trit::Zero` (abstain), `Trit::MinusOne` (against). Tracks dissent — how often each agent is overruled.

## Trit

The ternary digit is a first-class type:

```rust
let t = trit::Trit::from(-1); // MinusOne
assert_eq!(t.value(), -1);
```

## Running Tests

```bash
cargo test
```

20 tests covering all modules: ensemble, score, baton, measure, harmony, and full integration.

## License

MIT
