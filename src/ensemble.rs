use std::collections::HashMap;

use crate::baton::{FlexRequest, FlexResponse, LocalBaton};
use crate::memory::MuscleMemory;
use crate::score::{Condition, Instruction, Reading, Score};

/// An intent to conduct — a named task with a target value.
#[derive(Debug, Clone)]
pub struct ConductIntent {
    pub name: String,
    pub target: f64,
}

impl ConductIntent {
    pub fn new(name: impl Into<String>, target: f64) -> Self {
        ConductIntent {
            name: name.into(),
            target,
        }
    }
}

/// An agent in the ensemble — has a name and muscle memory.
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: String,
    pub memory: MuscleMemory,
    pub last_value: Option<f64>,
}

impl Agent {
    pub fn new(id: impl Into<String>, memory: MuscleMemory) -> Self {
        Agent {
            id: id.into(),
            memory,
            last_value: None,
        }
    }
}

/// The ensemble — a collection of agents coordinated by the conductor.
pub struct Ensemble {
    pub agents: HashMap<String, Agent>,
}

impl Ensemble {
    pub fn new() -> Self {
        Ensemble {
            agents: HashMap::new(),
        }
    }

    pub fn add_agent(&mut self, id: &str, memory: MuscleMemory) {
        self.agents.insert(id.to_string(), Agent::new(id, memory));
    }

    /// Get an agent by ID.
    pub fn get_agent(&self, id: &str) -> Option<&Agent> {
        self.agents.get(id)
    }

    pub fn get_agent_mut(&mut self, id: &str) -> Option<&mut Agent> {
        self.agents.get_mut(id)
    }

    /// Simple conduct: send an intent to all agents that have a matching pattern.
    pub fn conduct(&mut self, intent: &ConductIntent) -> Result<Vec<(&str, f64)>, String> {
        let mut results = Vec::new();
        for (id, agent) in &mut self.agents {
            if let Some(pattern) = agent.memory.get_pattern(&intent.name) {
                // Simple: use the pattern confidence scaled by target
                let value = intent.target * pattern.confidence;
                agent.last_value = Some(value);
                results.push((id.as_str(), value));
            }
        }
        Ok(results)
    }

    /// Execute a score directly (synchronous-style, no channels needed).
    /// Returns list of (agent_id, chord, value) for each executed step.
    pub fn execute_score_direct(&mut self, score: &Score) -> Result<Vec<(String, String, f64)>, String> {
        let mut results = Vec::new();

        for instruction in &score.instructions {
            match instruction {
                Instruction::Step(step) => {
                    let value = self.execute_step(&step.agent, &step.chord);
                    results.push((step.agent.clone(), step.chord.clone(), value));
                }
                Instruction::Branch { condition, then_steps, else_steps } => {
                    let cond_result = self.evaluate_condition(condition);
                    let steps = if cond_result { then_steps } else { else_steps };
                    for step in steps {
                        let value = self.execute_step(&step.agent, &step.chord);
                        results.push((step.agent.clone(), step.chord.clone(), value));
                    }
                }
                Instruction::Parallel(steps) => {
                    for step in steps {
                        let value = self.execute_step(&step.agent, &step.chord);
                        results.push((step.agent.clone(), step.chord.clone(), value));
                    }
                }
                Instruction::Fermata(_) => {
                    // Fermata handled separately via Measure
                }
            }
        }

        Ok(results)
    }

    fn execute_step(&mut self, agent_id: &str, chord: &str) -> f64 {
        if let Some(agent) = self.agents.get_mut(agent_id) {
            if let Some(pattern) = agent.memory.get_pattern(chord) {
                let value = agent.last_value.unwrap_or(0.0) * pattern.confidence;
                agent.last_value = Some(value);
                return value;
            }
            return agent.last_value.unwrap_or(0.0);
        }
        0.0
    }

    pub fn evaluate_condition(&self, condition: &Condition) -> bool {
        let (reading, threshold) = match condition {
            Condition::Gt(r, t) => (r, t),
            Condition::Lt(r, t) => (r, t),
            Condition::Eq(r, t) => (r, t),
            Condition::Gte(r, t) => (r, t),
            Condition::Lte(r, t) => (r, t),
        };

        let agent_id = match reading {
            Reading::Last(id) => id,
            Reading::Average(id) => id,
            Reading::Count(id) => id,
        };

        if let Some(agent) = self.agents.get(agent_id) {
            if let Some(val) = agent.last_value {
                return match condition {
                    Condition::Gt(_, t) => val > *t,
                    Condition::Lt(_, t) => val < *t,
                    Condition::Eq(_, t) => (val - *t).abs() < 0.001,
                    Condition::Gte(_, t) => val >= *t,
                    Condition::Lte(_, t) => val <= *t,
                };
            }
        }
        false
    }
}

impl Default for Ensemble {
    fn default() -> Self {
        Self::new()
    }
}
