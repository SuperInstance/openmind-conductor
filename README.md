# OpenMind Conductor

**OpenMind Conductor** is a Rust multi-agent orchestration framework with shared muscle memory — the prefrontal cortex for coordinated agent ensembles. It uses musical metaphors (scores, batons, measures, harmony) to model temporal coordination among autonomous agents.

## Why It Matters

Coordinating multiple AI agents requires three capabilities: (1) a shared plan (the score), (2) a communication channel (the baton), and (3) conflict resolution when agents disagree (harmony). OpenMind Conductor provides all three with a musical abstraction that maps naturally to real-time systems — beats synchronize timing, fermata pauses wait for conditions, and harmony voting resolves disagreements. The muscle memory layer persists learned patterns as JSON-serializable chord-to-trit mappings, enabling agents to load pre-trained coordination behaviors.

## How It Works

### Score-Based Orchestration

A `Score` is a sequence of `Instruction` variants:

```
Instruction::Step { agent, chord, args, timing }
Instruction::Branch { condition, then_steps, else_steps }
Instruction::Parallel { steps }    // fan-out
Instruction::Fermata { condition } // wait gate
```

Conditions compare `Reading` values (last, average, count) against thresholds using `Gt`, `Lt`, `Eq`, `Gte`, `Lte`. Scores serialize to JSON for persistence and inspection.

### Baton Communication

The `LocalBaton` uses tokio mpsc channels (capacity 256) for in-process agent communication:

```
register(agent_id) → Receiver<FlexRequest>
send(agent_id, request) → ()
recv() → FlexResponse
```

Message routing is **O(1)** via HashMap lookup. Each agent gets its own bounded mailbox, preventing memory exhaustion.

### Harmony Voting

When agents disagree, the `Harmony` module resolves conflicts via three strategies:

- **Majority**: Plurality of {+1, 0, -1} votes. Time: **O(N)** for N agents.
- **Consensus**: Any -1 vote blocks the decision. Strictest.
- **Weighted**: Each agent carries a weight w_i; outcome = sign(Σ w_i · v_i).

Dissenters are logged for post-hoc analysis of systematic disagreements.

### Muscle Memory

`MuscleMemory` stores named `Pattern`s — each containing a chord string, a sequence of `Trit`s {-1, 0, +1}, and a confidence score. Patterns are loaded from JSON:

```json
{ "move_forward": { "chord": "motor.go", "trits": [1,1,0], "confidence": 0.95 } }
```

Pattern lookup: **O(1)** via HashMap.

### Measure (Timing)

`Measure` synchronizes execution to BPM (beats per minute). At 120 BPM, each beat = 500ms. The `downbeat()` async function sleeps until the next beat boundary. `swing()` inserts an off-beat half-beat delay. Fermata conditions support `UntilValueExceeds`, `UntilValueBelow`, `ForDuration`, and `UntilCount`.

## Quick Start

```rust
use openmind_conductor::{ensemble::Ensemble, memory::MuscleMemory, score::Score};

let mut ensemble = Ensemble::new();
let memory = MuscleMemory::new();
ensemble.add_agent("alpha", memory);

let score = Score::builder()
    .then(agent="alpha", chord="move", args=["forward"])
    .build();

let results = ensemble.conduct(& ConductIntent::new("move", 1.0))?;
```

## API

| Module | Key Types |
|--------|-----------|
| `baton` | `LocalBaton`, `MockBaton`, `FlexRequest`, `FlexResponse` |
| `ensemble` | `Ensemble`, `Agent`, `ConductIntent` |
| `harmony` | `Harmony`, `VotingStrategy`, `VoteResult` |
| `memory` | `MuscleMemory`, `Pattern` |
| `score` | `Score`, `ScoreBuilder`, `Step`, `Instruction`, `Condition` |
| `measure` | `Measure`, `Bpm`, `Timing`, `Fermata` |
| `trit` | `Trit` (MinusOne, Zero, PlusOne) |

## Architecture Notes

OpenMind Conductor is the orchestration layer of the OpenMind system within SuperInstance. In γ + η = C, the conductor drives γ (growth — coordinating agents toward shared goals) while harmony voting provides η (avoidance — blocking harmful collective decisions). The `Trit` type directly implements the ternary {-1, 0, +1} conservation principle.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for the OpenMind architecture.

## References

1. Wooldridge, M. (2009). *An Introduction to MultiAgent Systems*, 2nd ed. Wiley.
2. Stone, P. & Veloso, M. (2000). "Multiagent Systems: A Survey from a Machine Learning Perspective." *Autonomous Robots*, 8(3), 345–383.
3. Dijkstra, E. W. (1965). "Solution of a Problem in Concurrent Programming Control." *Communications of the ACM*, 8(9), 569.

## License

MIT
