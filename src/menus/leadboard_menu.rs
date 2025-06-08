use crate::menus::GameState;
use crate::theme::widget;
use bevy::prelude::*;
use bevy_jornet::{JornetPlugin, Leaderboard};
use bevy_simple_text_input::{TextInput, TextInputPlugin};
use std::env;

pub struct LeaderboardMenuPlugin;

impl Plugin for LeaderboardMenuPlugin {
	fn build(&self, app: &mut App) {
		dotenv::dotenv().ok();
		app.add_plugins(TextInputPlugin);
		let secret_key_1 = env::var("LEADERBOARD_ONE").unwrap();
		let secret_key_2 = env::var("LEADERBOARD_TWO").unwrap();
		app.add_plugins(JornetPlugin::with_leaderboard(
			&*secret_key_1,
			&*secret_key_2,
		));
		app.add_systems(OnEnter(GameState::Leaderboard), spawn_menu);
	}
}

fn spawn_menu(mut commands: Commands, leaderboard: Res<Leaderboard>) {
	commands.spawn((Camera2d, StateScoped(GameState::Leaderboard)));
	let mut children = vec![];

	let mut score = leaderboard.get_leaderboard();
	score.sort_by(|awa, uwu| uwu.score.partial_cmp(&awa.score).unwrap());

	for score in score {
		children.push(
			commands
				.spawn(widget::label(format!(
					"Name: {}, Score: {}, Timestamp: {}",
					score.player, score.score, score.timestamp
				)))
				.id(),
		);
	}
	commands
		.spawn((
			widget::ui_root("Leaderboard"),
			GlobalZIndex(2),
			StateScoped(GameState::Leaderboard),
			#[cfg(not(target_family = "wasm"))]
			children![widget::button("Back", back),],
		))
		.add_children(&children);
}

fn back(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<GameState>>) {
	next_menu.set(GameState::MainMenu);
}
