use crate::Score;
use crate::menus::GameState;
use crate::theme::widget;
use bevy::prelude::*;
use bevy_jornet::Leaderboard;
use bevy_simple_text_input::TextInput;

pub struct ShopMenuPlugin;
impl Plugin for ShopMenuPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<TotalPoints>();
		app.insert_resource(SlimeSlownessLevel(1));
		app.insert_resource(ChainRadiusLevel(1));
		app.add_systems(OnEnter(GameState::Shop), spawn_shop_menu);
		app.add_systems(OnExit(GameState::Game), add_to_total_points);
		app.add_systems(OnEnter(GameState::Game), reset_score);
	}
}

fn reset_score(mut score: ResMut<Score>) {
	score.0 = 0;
}

fn add_to_total_points(score: Res<Score>, mut total_points: ResMut<TotalPoints>) {
	total_points.0 += score.0 as u64;
}

fn despawn_these(mut commands: Commands, query: Query<Entity, With<DespawnThese>>) {
	for entity in query.iter() {
		commands.entity(entity).despawn();
	}
	commands.run_system_cached(spawn_shop_menu);
}

#[derive(Component)]
struct DespawnThese;

#[derive(Resource, Default)]
pub struct TotalPoints(pub u64);

#[derive(Component)]
struct PointsTracker;

fn spawn_shop_menu(
	mut commands: Commands,
	total_points: Res<TotalPoints>,
	slime_slowness_level: Res<SlimeSlownessLevel>,
	chain_radius_level: Res<ChainRadiusLevel>,
) {
	commands.spawn((Camera2d, StateScoped(GameState::Shop), DespawnThese));

	let menu = commands
		.spawn((
			widget::ui_root("Shop Menu"),
			GlobalZIndex(2),
			StateScoped(GameState::Shop),
			DespawnThese,
		))
		.id();
	let points_tracker = commands
		.spawn((
			widget::label(format!("Points: {}", total_points.0)),
			PointsTracker,
		))
		.id();
	commands.entity(menu).add_child(points_tracker);
	commands.entity(menu).insert((children![
		widget::button("New Round", enter_gameplay),
		widget::button("Main Menu", main_menu),
		widget::button(
			format!("Chain Radius {}", chain_radius_level.0),
			buy_chain_radius
		),
	],));
	let slime_slowness = commands
		.spawn(widget::button(
			format!("Slime Slow {}", slime_slowness_level.0),
			buy_slime_slowness,
		))
		.id();
	commands.entity(menu).add_child(slime_slowness);
}

#[derive(Resource, Default)]
pub struct SlimeSlownessLevel(u32);

#[derive(Resource, Default)]
pub struct ChainRadiusLevel(u32);

#[derive(Component)]
struct SlimeSlownessButton;

fn buy_chain_radius(
	_: Trigger<Pointer<Click>>,
	mut commands: Commands,
	mut chain_radius_level: ResMut<ChainRadiusLevel>,
	mut total_points: ResMut<TotalPoints>,
) {
	if total_points.0 < (chain_radius_level.0 * 3) as u64 {
		return;
	}
	total_points.0 -= (chain_radius_level.0 * 3) as u64;
	chain_radius_level.0 += 1;
	commands.run_system_cached(despawn_these);
}

fn buy_slime_slowness(
	_: Trigger<Pointer<Click>>,
	mut commands: Commands,
	mut slime_slowness_level: ResMut<SlimeSlownessLevel>,
	mut total_points: ResMut<TotalPoints>,
) {
	if total_points.0 < (slime_slowness_level.0 * 3) as u64 {
		return;
	}
	total_points.0 -= (slime_slowness_level.0 * 3) as u64;
	slime_slowness_level.0 += 1;
	commands.run_system_cached(despawn_these);
}

fn enter_gameplay(
	_: Trigger<Pointer<Click>>,
	mut next_menu: ResMut<NextState<GameState>>,
) {
	next_menu.set(GameState::Game);
}

fn main_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<GameState>>) {
	next_menu.set(GameState::MainMenu);
}
