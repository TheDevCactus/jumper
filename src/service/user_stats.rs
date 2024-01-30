use bevy::ecs::system::Resource;
use serde::{Deserialize, Serialize};

use super::constants::Constants;

#[derive(Serialize, Deserialize, Debug, Clone, Resource)]
pub struct LevelResult {
    pub level_id: String,
    pub time: usize,
    pub score: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserStats {
    pub level_results_points: Vec<LevelResult>,
    pub level_results_time: Vec<LevelResult>,
}

impl UserStats {
    pub fn load_from_file() -> Option<UserStats> {
        let constants = Constants::read_from_file();
        let raw =
            std::fs::read_to_string(format!("{}/user_stats.json", constants.path_to_player_data));
        if let Err(_) = raw {
            return None;
        }
        let mut trick_list = serde_json::from_str::<UserStats>(&raw.unwrap());
        if let Err(_) = trick_list {
            return None;
        }
        Some(trick_list.unwrap())
    }
    pub fn save_to_file(&self, file_path: String) {
        let serialized = serde_json::to_string(&self).unwrap();
        std::fs::write(file_path, serialized).unwrap();
    }
}

pub fn create_user_stats_file() {
    let user_stats = UserStats::default();
    user_stats.save_to_file("./player/user_stats.json".to_string());
}

pub fn record_level_result_to_user_stats(level_result: LevelResult) {
    let user_stats = UserStats::load_from_file();
    if let None = user_stats {
        return;
    }
    let mut user_stats = user_stats.unwrap();
    let time_last_entry = user_stats
        .level_results_time
        .iter_mut()
        .filter(|current_level_result| level_result.level_id == current_level_result.level_id)
        .map(|level_result| level_result.time)
        .min()
        .unwrap_or(0);
    let score_last_entry = user_stats
        .level_results_points
        .iter_mut()
        .filter(|current_level_result| level_result.level_id == current_level_result.level_id)
        .map(|level_result| level_result.score)
        .min()
        .unwrap_or(0);
    if time_last_entry == 0 || time_last_entry > level_result.time {
        user_stats.level_results_time.push(level_result.clone());
    }
    if score_last_entry == 0 || score_last_entry < level_result.score {
        user_stats.level_results_points.push(level_result);
    }
    user_stats.save_to_file("./player/user_stats.json".to_string());
}
