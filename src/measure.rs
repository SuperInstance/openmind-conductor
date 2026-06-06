use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Beats per minute — how often the conductor checks state.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bpm(pub f64);

impl Bpm {
    pub fn beat_duration(&self) -> Duration {
        Duration::from_secs_f64(60.0 / self.0)
    }
}

impl Default for Bpm {
    fn default() -> Self {
        Bpm(120.0) // 120 BPM default
    }
}

/// Timing for a step in the score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Timing {
    Immediate,
    After(u64), // milliseconds
    OnBeat(u32), // on specific beat number
}

/// A fermata — pause until a condition is met.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Fermata {
    UntilValueExceeds { agent: String, threshold: f64 },
    UntilValueBelow { agent: String, threshold: f64 },
    ForDuration(u64), // milliseconds
    UntilCount { agent: String, count: usize },
}

/// Measure — timing and synchronization manager.
pub struct Measure {
    pub bpm: Bpm,
    pub start: Instant,
    pub beat_count: u32,
}

impl Measure {
    pub fn new(bpm: Bpm) -> Self {
        Measure {
            bpm,
            start: Instant::now(),
            beat_count: 0,
        }
    }

    /// Wait for the next downbeat.
    pub async fn downbeat(&mut self) {
        let elapsed = self.start.elapsed();
        let beat_dur = self.bpm.beat_duration();
        let beats_elapsed = elapsed.as_secs_f64() / beat_dur.as_secs_f64();
        let next_beat = (beats_elapsed.floor() + 1.0) as u64;
        let wait = Duration::from_secs_f64(next_beat as f64 * beat_dur.as_secs_f64())
            .checked_sub(elapsed)
            .unwrap_or(Duration::ZERO);
        tokio::time::sleep(wait).await;
        self.beat_count += 1;
    }

    /// Swing — off-beat timing: wait half a beat.
    pub async fn swing(&self) {
        tokio::time::sleep(self.bpm.beat_duration() / 2).await;
    }

    /// Wait for a fermata condition.
    pub async fn fermata(&self, condition: Fermata, check: impl Fn(&str) -> Option<f64>) -> bool {
        match condition {
            Fermata::ForDuration(ms) => {
                tokio::time::sleep(Duration::from_millis(ms)).await;
                true
            }
            Fermata::UntilValueExceeds { ref agent, threshold } => {
                let deadline = Instant::now() + Duration::from_secs(30);
                loop {
                    if let Some(val) = check(agent) {
                        if val > threshold {
                            return true;
                        }
                    }
                    if Instant::now() > deadline {
                        return false;
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
            Fermata::UntilValueBelow { ref agent, threshold } => {
                let deadline = Instant::now() + Duration::from_secs(30);
                loop {
                    if let Some(val) = check(agent) {
                        if val < threshold {
                            return true;
                        }
                    }
                    if Instant::now() > deadline {
                        return false;
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
            Fermata::UntilCount { .. } => true,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}
