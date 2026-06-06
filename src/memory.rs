use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::trit::Trit;

/// Simplified muscle memory — a JSON-loaded collection of learned patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuscleMemory {
    pub patterns: HashMap<String, Pattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub chord: String,
    pub trits: Vec<Trit>,
    pub confidence: f64,
}

impl MuscleMemory {
    pub fn new() -> Self {
        MuscleMemory {
            patterns: HashMap::new(),
        }
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    pub fn add_pattern(&mut self, name: impl Into<String>, chord: impl Into<String>, trits: Vec<Trit>, confidence: f64) {
        self.patterns.insert(name.into(), Pattern {
            chord: chord.into(),
            trits,
            confidence,
        });
    }

    pub fn get_pattern(&self, name: &str) -> Option<&Pattern> {
        self.patterns.get(name)
    }
}

impl Default for MuscleMemory {
    fn default() -> Self {
        Self::new()
    }
}
