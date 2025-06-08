mod leadboard_menu;
mod main_menu;
mod pause_menu;
mod settings_menu;
pub mod shop_menu;

use crate::menus::leadboard_menu::LeaderboardMenuPlugin;
use crate::menus::main_menu::MainMenuPlugin;
use crate::menus::pause_menu::PauseMenuPlugin;
use crate::menus::settings_menu::SettingsMenuPlugin;
use crate::menus::shop_menu::ShopMenuPlugin;
use crate::theme;
use crate::tutorial_section::TutorialPlugin;
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
		app.add_plugins(TutorialPlugin);
		app.add_plugins(ShopMenuPlugin);
		app.add_plugins(theme::plugin);
	}
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub enum GameState {
	MainMenu,
	Settings,
	Game,
	Leaderboard,
	Shop,
	#[default]
	Tutorial,
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub enum PauseMenu {
	Paused,
	#[default]
	Unpaused,
}
