use crate::{PulseError, PulseResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GamificationState {
    pub score: u64,
    pub streak_green: u32,
    pub last_run_ms: u128,
    pub best_run_ms: Option<u128>,
    pub total_runs: u64,
    pub total_passed_specs: u64,
    pub total_failed_specs: u64,
    pub boss_hp: u32, // represents number of failing specs in last run
    pub last_updated: Option<DateTime<Utc>>,
}

impl GamificationState {
    pub fn update_after_run(&mut self, total_specs: usize, passed_specs: usize, duration_ms: u128) {
        let failed = total_specs.saturating_sub(passed_specs);
        self.total_runs += 1;
        self.total_passed_specs += passed_specs as u64;
        self.total_failed_specs += failed as u64;
        self.last_run_ms = duration_ms;
        self.best_run_ms = Some(self.best_run_ms.map_or(duration_ms, |best| best.min(duration_ms)));
        self.boss_hp = failed as u32;
        self.last_updated = Some(Utc::now());

        // Scoring: reward greens strongly, penalize failures lightly
        if failed == 0 {
            self.streak_green += 1;
            // Base 1000 points minus time factor (capped)
            let time_bonus = (1000u64).saturating_sub((duration_ms / 10) as u64).max(100);
            self.score = self.score.saturating_add(500 + time_bonus + (self.streak_green as u64 * 50));
        } else {
            // Break streak and apply small penalty
            self.streak_green = 0;
            let penalty = (failed as u64) * 50;
            self.score = self.score.saturating_sub(penalty.min(self.score));
        }
    }
}

pub fn load_gamification<P: AsRef<Path>>(base_dir: P) -> PulseResult<GamificationState> {
    let dir = base_dir.as_ref();
    let path = dir.join("gamification.json");
    if !dir.exists() {
        fs::create_dir_all(dir).map_err(|e| PulseError::fs_error(format!("Cannot create {}: {}", dir.display(), e), None::<String>))?;
    }
    if !path.exists() {
        return Ok(GamificationState::default());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| PulseError::fs_error(format!("Cannot read {}: {}", path.display(), e), None::<String>))?;
    let state: GamificationState = serde_json::from_str(&content)
        .map_err(|e| PulseError::config_error(format!("Invalid JSON in {}: {}", path.display(), e), None::<String>))?;
    Ok(state)
}

pub fn save_gamification<P: AsRef<Path>>(base_dir: P, state: &GamificationState) -> PulseResult<PathBuf> {
    let dir = base_dir.as_ref();
    if !dir.exists() {
        fs::create_dir_all(dir).map_err(|e| PulseError::fs_error(format!("Cannot create {}: {}", dir.display(), e), None::<String>))?;
    }
    let path = dir.join("gamification.json");
    let data = serde_json::to_string_pretty(state)
        .map_err(|e| PulseError::config_error(format!("Cannot serialize gamification state: {}", e), None::<String>))?;
    fs::write(&path, data)
        .map_err(|e| PulseError::fs_error(format!("Cannot write {}: {}", path.display(), e), None::<String>))?;
    Ok(path)
}

