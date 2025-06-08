use crate::menus::GameState;
use crate::theme::widget;
use bevy::prelude::*;
use bevy_jornet::Leaderboard;
use bevy_simple_text_input::TextInput;

pub struct MainMenuPlugin;
impl Plugin for MainMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::MainMenu), spawn_main_menu);
	}
}

fn spawn_main_menu(mut commands: Commands, leaderboard: ResMut<Leaderboard>) {
	leaderboard.refresh_leaderboard();
	println!("UWU");
	commands.spawn((Camera2d, StateScoped(GameState::MainMenu)));
	commands.spawn((
		widget::ui_root("Main Menu"),
		GlobalZIndex(2),
		StateScoped(GameState::MainMenu),
		children![
			widget::button("Play", enter_gameplay),
			widget::button("Tutorial", open_tutorial),
			widget::button("Settings", open_settings_menu),
			widget::button("Shop", open_shop_menu),
			widget::button("Leaderboard", open_leaderboard_menu),
		],
	));
}

fn enter_gameplay(
	_: Trigger<Pointer<Click>>,
	mut next_menu: ResMut<NextState<GameState>>,
) {
	next_menu.set(GameState::Game);
}

fn open_settings_menu(
	_: Trigger<Pointer<Click>>,
	mut next_menu: ResMut<NextState<GameState>>,
) {
	next_menu.set(GameState::Settings);
}

fn open_tutorial(
	_: Trigger<Pointer<Click>>,
	mut next_menu: ResMut<NextState<GameState>>,
) {
	next_menu.set(GameState::Tutorial);
}

fn open_shop_menu(
	_: Trigger<Pointer<Click>>,
	mut next_menu: ResMut<NextState<GameState>>,
) {
	next_menu.set(GameState::Shop);
}

fn open_leaderboard_menu(
	_: Trigger<Pointer<Click>>,
	mut next_menu: ResMut<NextState<GameState>>,
) {
	next_menu.set(GameState::Leaderboard);
}
