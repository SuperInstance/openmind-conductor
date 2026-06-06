use serde::{Deserialize, Serialize};

use crate::measure::Timing;

/// A condition for branching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    Gt(Reading, f64),
    Lt(Reading, f64),
    Eq(Reading, f64),
    Gte(Reading, f64),
    Lte(Reading, f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Reading {
    Last(String),     // last value from agent
    Average(String),  // average value from agent
    Count(String),    // number of readings from agent
}

/// A single step in the score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub agent: String,
    pub chord: String,
    pub args: Vec<String>,
    pub timing: Timing,
}

impl Step {
    pub fn new(agent: impl Into<String>, chord: impl Into<String>, args: Vec<&str>) -> Self {
        Step {
            agent: agent.into(),
            chord: chord.into(),
            args: args.into_iter().map(|s| s.to_string()).collect(),
            timing: Timing::Immediate,
        }
    }

    pub fn with_timing(mut self, timing: Timing) -> Self {
        self.timing = timing;
        self
    }
}

/// An instruction in the score — either a step or a branch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    Step(Step),
    Branch {
        condition: Condition,
        then_steps: Vec<Step>,
        else_steps: Vec<Step>,
    },
    Parallel(Vec<Step>),
    Fermata(crate::measure::Fermata),
}

/// A score — a pre-computed plan for the ensemble.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub instructions: Vec<Instruction>,
}

impl Score {
    pub fn builder() -> ScoreBuilder {
        ScoreBuilder {
            instructions: Vec::new(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

pub struct ScoreBuilder {
    instructions: Vec<Instruction>,
}

impl ScoreBuilder {
    pub fn step(mut self, agent: &str, chord: &str, args: Vec<&str>, timing: Timing) -> Self {
        self.instructions.push(Instruction::Step(
            Step::new(agent, chord, args).with_timing(timing),
        ));
        self
    }

    pub fn branch(
        mut self,
        condition: Condition,
        then_steps: Vec<Step>,
        else_steps: Vec<Step>,
    ) -> Self {
        self.instructions.push(Instruction::Branch {
            condition,
            then_steps,
            else_steps,
        });
        self
    }

    pub fn parallel(mut self, steps: Vec<Step>) -> Self {
        self.instructions.push(Instruction::Parallel(steps));
        self
    }

    pub fn fermata(mut self, fermata: crate::measure::Fermata) -> Self {
        self.instructions.push(Instruction::Fermata(fermata));
        self
    }

    pub fn build(self) -> Score {
        Score {
            instructions: self.instructions,
        }
    }
}
