mod leadboard_menu;
mod main_menu;
mod pause_menu;
mod settings_menu;
mod shop_menu;

use crate::menus::leadboard_menu::LeaderboardMenuPlugin;
use crate::menus::main_menu::MainMenuPlugin;
use crate::menus::pause_menu::PauseMenuPlugin;
use crate::menus::settings_menu::SettingsMenuPlugin;
use crate::theme;
use bevy::prelude::*;

pub struct MenuPlugins;

impl Plugin for MenuPlugins {
	fn build(&self, app: &mut App) {
		app.init_state::<GameState>();
		app.init_state::<PauseMenu>();
		app.add_plugins(MainMenuPlugin);
		app.add_plugins(PauseMenuPlugin);
		app.add_plugins(LeaderboardMenuPlugin);
		app.add_plugins(SettingsMenuPlugin);
		app.add_plugins(theme::plugin);
	}
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub enum GameState {
	MainMenu,
	#[default]
	Settings,
	Game,
	Leaderboard,
	Shop,
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub enum PauseMenu {
	Paused,
	#[default]
	Unpaused,
}
