use bevy::{
    app::{App, Plugin, PreStartup},
    ecs::{schedule::ScheduleLabel, system::Commands},
    utils::intern::Interned,
};

use crate::{
    models::{TrickList, TrickListResource},
    service::{constants::Constants, user_stats},
};

fn initialize_constants(mut commands: Commands) {
    let raw = std::fs::read_to_string("./assets/constants.toml").unwrap();
    let constants = toml::from_str::<Constants>(&raw).unwrap();
    // print the constants to the console
    commands.insert_resource(constants);
}

fn initialize_trick_list(mut commands: Commands) {
    let raw = std::fs::read_to_string("./assets/trick_list.json").unwrap();
    let mut trick_list = serde_json::from_str::<TrickList>(&raw).unwrap();
    // handle longer tricks first so that we don't accidentally
    // trigger a shorter trick when we're trying to do a longer one
    trick_list
        .tricks
        .sort_by(|(keys_a, _), (keys_b, _)| keys_b.len().cmp(&keys_a.len()));
    commands.insert_resource(TrickListResource(trick_list));
}

pub struct ConfigLoader {
    pub pre_startup: Interned<dyn ScheduleLabel>,
}

fn write_data_files() {
    if let None = user_stats::UserStats::load_from_file() {
        user_stats::create_user_stats_file();
    }
}

impl Plugin for ConfigLoader {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreStartup,
            (
                initialize_trick_list,
                initialize_constants,
                write_data_files,
            ),
        );
    }
}
