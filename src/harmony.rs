use std::collections::HashMap;

use crate::trit::Trit;

/// Voting strategy for conflict resolution.
#[derive(Debug, Clone)]
pub enum VotingStrategy {
    Majority,
    Consensus,
    Weighted(HashMap<String, f64>),
}

/// Result of a harmony vote.
#[derive(Debug, Clone)]
pub struct VoteResult {
    pub outcome: Trit,
    pub for_count: usize,
    pub against_count: usize,
    pub abstain_count: usize,
    pub dissenters: Vec<String>,
}

/// Harmony — conflict resolution when agents disagree.
pub struct Harmony {
    pub strategy: VotingStrategy,
    /// Track how often each agent is overruled.
    pub dissent_log: HashMap<String, u32>,
}

impl Harmony {
    pub fn new(strategy: VotingStrategy) -> Self {
        Harmony {
            strategy,
            dissent_log: HashMap::new(),
        }
    }

    /// Conduct a vote among agents.
    pub fn vote(&mut self, votes: HashMap<String, Trit>) -> VoteResult {
        let for_count = votes.values().filter(|v| **v == Trit::PlusOne).count();
        let against_count = votes.values().filter(|v| **v == Trit::MinusOne).count();
        let abstain_count = votes.values().filter(|v| **v == Trit::Zero).count();

        let outcome = match &self.strategy {
            VotingStrategy::Majority => {
                if for_count > against_count && for_count > abstain_count {
                    Trit::PlusOne
                } else if against_count > for_count {
                    Trit::MinusOne
                } else {
                    Trit::Zero
                }
            }
            VotingStrategy::Consensus => {
                if against_count > 0 || votes.is_empty() {
                    Trit::MinusOne // consensus blocked by any against
                } else if for_count == votes.len() {
                    Trit::PlusOne
                } else {
                    Trit::Zero
                }
            }
            VotingStrategy::Weighted(weights) => {
                let mut score = 0.0_f64;
                for (agent, vote) in &votes {
                    let w = weights.get(agent).copied().unwrap_or(1.0);
                    score += vote.value() as f64 * w;
                }
                if score > 0.0 {
                    Trit::PlusOne
                } else if score < 0.0 {
                    Trit::MinusOne
                } else {
                    Trit::Zero
                }
            }
        };

        // Track dissenters
        let dissenters: Vec<String> = votes
            .iter()
            .filter(|(_, v)| **v != outcome)
            .map(|(k, _)| k.clone())
            .collect();

        for d in &dissenters {
            *self.dissent_log.entry(d.clone()).or_insert(0) += 1;
        }

        VoteResult {
            outcome,
            for_count,
            against_count,
            abstain_count,
            dissenters,
        }
    }

    /// How many times an agent has been overruled.
    pub fn dissent_count(&self, agent: &str) -> u32 {
        self.dissent_log.get(agent).copied().unwrap_or(0)
    }

    /// Total dissent events.
    pub fn total_dissent(&self) -> u32 {
        self.dissent_log.values().sum()
    }
}
