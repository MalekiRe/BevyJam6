use crate::menus::GameState;
use crate::theme::widget;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_jornet::{JornetPlugin, Leaderboard};
use bevy_simple_text_input::{TextInput, TextInputPlugin};
use std::env;

pub struct LeaderboardMenuPlugin;

impl Plugin for LeaderboardMenuPlugin {
	fn build(&self, app: &mut App) {
		dotenv::dotenv().ok();
		app.add_plugins(TextInputPlugin);
		let secret_key_1 = env!("LEADERBOARD_ONE");
		let secret_key_2 = env!("LEADERBOARD_TWO");
		app.add_plugins(JornetPlugin::with_leaderboard(
			&*secret_key_1,
			&*secret_key_2,
		));
		app.add_systems(
			Update,
			update_scroll_position.run_if(in_state(GameState::Leaderboard)),
		);
		app.add_systems(OnEnter(GameState::Leaderboard), spawn_menu);
	}
}

fn spawn_menu(
	mut commands: Commands,
	leaderboard: Res<Leaderboard>,
	window: Single<&Window, With<PrimaryWindow>>,
) {
	commands.spawn((
		Camera2d,
		StateScoped(GameState::Leaderboard),
		IsDefaultUiCamera,
	));
	let mut children = vec![];

	let mut score = leaderboard.get_leaderboard();
	score.sort_by(|awa, uwu| uwu.score.partial_cmp(&awa.score).unwrap());

	for score in score {
		children.push(
			commands
				.spawn((
					widget::label(format!(
						"Name: {}, Score: {}, Timestamp: {}",
						score.player, score.score, score.timestamp
					)),
					Pickable::IGNORE,
				))
				.id(),
		);
	}
	/*let p = commands
	.spawn((
		widget::ui_root("Leaderboard"),
		GlobalZIndex(2),
		StateScoped(GameState::Leaderboard),
	))
	.id();*/
	let e = commands
		.spawn((
			StateScoped(GameState::Leaderboard),
			Node {
				flex_direction: FlexDirection::Column,
				align_self: AlignSelf::Auto,
				justify_content: JustifyContent::Start,
				align_items: AlignItems::Center,
				position_type: PositionType::Absolute,
				width: Val::Percent(100.0),
				height: Val::Percent(100.0),
				//top: Val::Px(window.height() / 2.0),
				overflow: Overflow::scroll_y(),
				..default()
			},
			//Transform::from_translation(Vec3::new(0.0, window.height(), 0.0)),
			children![widget::button("Back", back),],
			Pickable {
				should_block_lower: false,
				..default()
			},
		))
		.id();
	commands.entity(e).add_children(&children);
	//commands.entity(p).add_child(e);
}
const FONT_SIZE: f32 = 20.;
const LINE_HEIGHT: f32 = 21.;

pub fn update_scroll_position(
	mut mouse_wheel_events: EventReader<MouseWheel>,
	hover_map: Res<HoverMap>,
	mut scrolled_node_query: Query<&mut ScrollPosition>,
) {
	for mouse_wheel_event in mouse_wheel_events.read() {
		let (mut dx, mut dy) = match mouse_wheel_event.unit {
			MouseScrollUnit::Line => (
				mouse_wheel_event.x * LINE_HEIGHT,
				mouse_wheel_event.y * LINE_HEIGHT,
			),
			MouseScrollUnit::Pixel => (mouse_wheel_event.x, mouse_wheel_event.y),
		};

		for (_pointer, pointer_map) in hover_map.iter() {
			for (entity, _hit) in pointer_map.iter() {
				if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
					scroll_position.offset_x -= dx;
					scroll_position.offset_y -= dy;
				}
			}
		}
	}
}

fn back(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<GameState>>) {
	next_menu.set(GameState::MainMenu);
}
