use crate::Score;
use crate::menus::GameState;
use crate::theme::widget;
use bevy::prelude::*;
use bevy_jornet::{JornetPlugin, Leaderboard};
use bevy_simple_text_input::{TextInput, TextInputPlugin, TextInputValue};
use std::env;

pub struct SettingsMenuPlugin;

impl Plugin for SettingsMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::Settings), spawn_menu);
		app.add_systems(OnExit(GameState::Settings), set_username);
		app.add_systems(OnExit(GameState::Game), send_score);
	}
}

fn send_score(leaderboard: ResMut<Leaderboard>, score: ResMut<Score>) {
	leaderboard.send_score(score.0 as f32);
}

fn set_username(
	mut commands: Commands,
	text_input: Single<&TextInputValue>,
	mut leaderboard: ResMut<Leaderboard>,
) {
	commands.insert_resource(Username(text_input.0.clone()));
	leaderboard.create_player(Some(&text_input.0));
}

#[derive(Resource)]
pub struct Username(pub String);

fn spawn_menu(mut commands: Commands, possible_username: Option<Res<Username>>) {
	commands.spawn((Camera2d, StateScoped(GameState::Settings)));
	let text_input = commands
		.spawn((
			TextInput,
			Node {
				padding: UiRect::all(Val::Px(5.0)),
				border: UiRect::all(Val::Px(2.0)),
				..default()
			},
			match possible_username {
				Some(awa) => TextInputValue(awa.0.clone()),
				None => TextInputValue("".to_string()),
			},
			BorderColor(Color::BLACK),
		))
		.id();
	commands
		.spawn((
			widget::ui_root("Leaderboard"),
			GlobalZIndex(2),
			StateScoped(GameState::Settings),
			children![
				widget::button("Main Menu", back),
				Text("Username for Leaderboard".to_string()),
			],
		))
		.add_child(text_input);
}

fn back(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<GameState>>) {
	next_menu.set(GameState::MainMenu);
}
