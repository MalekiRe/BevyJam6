mod enemy;
mod explosion;
mod menus;
mod music;
mod player;
mod screen_shake;
mod text_combo;
mod theme;

use std::cmp::max;
use std::collections::HashSet;
use std::f32::consts::PI;
use std::hint::unreachable_unchecked;
use std::ops::{Add, AddAssign, DerefMut, Sub};
use std::time::Duration;

use crate::enemy::EnemyPlugin;
use crate::explosion::FireParticleMaterial;
use crate::menus::{GameState, PauseMenu};
use crate::music::MusicPlugin;
use crate::player::{AnimationState, Direction, PlayerPlugin, PlayerState};
use crate::screen_shake::{ScreenShakePlugin, SlimeDestroyed};
use crate::text_combo::{TextCombo, TextComboPlugin};
use bevy::asset::Handle;
use bevy::audio::{PlaybackSettings, Volume};
use bevy::color::Color;
use bevy::color::palettes::css;
use bevy::ecs::children;
use bevy::ecs::component::HookContext;
use bevy::ecs::relationship::OrderedRelationshipSourceCollection;
use bevy::ecs::system::RunSystemOnce;
use bevy::ecs::world::DeferredWorld;
use bevy::image::Image;
use bevy::math::{EulerRot, Quat, Rect, Vec2, Vec3};
use bevy::prelude::EaseFunction::BounceOut;
use bevy::prelude::{
	AlignItems, Alpha, AudioPlayer, ButtonInput, Camera, ChildOf, Circle, Click,
	ColorMaterial, ContainsEntity, Entity, Event, EventReader, EventWriter,
	FlexDirection, GlobalTransform, IVec2, IntoScheduleConfigs, JustifyContent,
	JustifyText, KeyCode, Local, Luminance, Mesh, Mesh2d, MeshMaterial2d,
	MeshPickingPlugin, Mut, Node, OnAdd, OnEnter, OnRemove, OrthographicProjection,
	Out, Over, Pickable, Plugin, Pointer, PositionType, Pressed, Rectangle, Resource,
	Saturation, Single, StateScoped, Text, Text2d, TextLayout, Transform, Trigger, Val,
	Window, With, Without, World, default, in_state,
};
use bevy::prelude::{BackgroundColor, SpawnRelated};
use bevy::render::camera::{CameraProjection, SubCameraView};
use bevy::render::primitives::Frustum;
use bevy::sprite::SpriteImageMode;
use bevy::window::PrimaryWindow;
use bevy::{
	DefaultPlugins,
	app::{App, Startup, Update},
	asset::{AssetServer, Assets},
	core_pipeline::core_2d::Camera2d,
	ecs::{
		component::Component,
		system::{Commands, Query, Res, ResMut},
	},
	image::{TextureAtlas, TextureAtlasLayout},
	math::UVec2,
	prelude::{Deref, DerefMut, PluginGroup},
	render::texture::ImagePlugin,
	sprite::Sprite,
	time::{Time, Timer, TimerMode},
};
use bevy_defer::{AsyncAccess, AsyncCommandsExtension, AsyncWorld};
use bevy_ecs_tilemap::map::TilemapId;
use bevy_ecs_tilemap::prelude::{
	ArrayTextureLoader, TileBundle, TilePos, TilemapAnchor, TilemapArrayTexture,
	TilemapSize, TilemapTexture, TilemapTileSize, TilemapType,
};
use bevy_ecs_tilemap::tiles::TileStorage;
use bevy_ecs_tilemap::{FrustumCulling, TilemapBundle, TilemapPlugin};
use bevy_enoki::prelude::{
	MultiCurve, OneShot, Particle2dMaterialPlugin, ParticleEffectInstance,
	ParticleSpawnerState, Rval,
};
use bevy_enoki::{
	EnokiPlugin, Particle2dEffect, ParticleEffectHandle, ParticleSpawner,
};
use rand::Rng;
use rand::distributions::Standard;
use rand::prelude::Distribution;
use random_number::random;

fn main() {
	App::new()
		.add_plugins((
			DefaultPlugins.set(ImagePlugin::default_nearest()),
			MeshPickingPlugin,
			bevy_defer::AsyncPlugin::default_settings(),
			menus::MenuPlugins,
		))
		.add_plugins(EnokiPlugin)
		.add_plugins(Particle2dMaterialPlugin::<FireParticleMaterial>::default())
		.add_plugins((
			PlayerPlugin,
			MusicPlugin,
			EnemyPlugin,
			ScreenShakePlugin,
			TextComboPlugin,
			MainGamePlugin,
		))
		.run();
}

fn setup_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
	let texture_size = UVec2::new(16, 16);
	let size = IVec2::new(MAP_RADI.x as i32 / 16, MAP_RADI.y as i32 / 16);

	let plain = Vec2::new(40.0, 40.0);
	for x in -size.x..size.x {
		for y in -size.y..size.y {
			let positions = if random!(0.0..1.0) <= 0.04 {
				if random!(0.0..1.0) <= 0.8 {
					match random!(0..8) {
						0 => Vec2::new(16.0 * 11.0, 16.0 * 2.0),
						1 => Vec2::new(16.0 * 11.0, 16.0 * 3.0),
						2 => Vec2::new(16.0 * 11.0, 16.0 * 4.0),
						3 => Vec2::new(16.0 * 10.0, 16.0 * 4.0),
						4 => Vec2::new(16.0 * 10.0, 16.0 * 3.0),
						_ => Vec2::new(16.0 * 10.0, 16.0 * 2.0),
					}
				} else {
					match random!(0..4) {
						0 => Vec2::new(16.0 * 9.0, 16.0 * 2.0),
						1 => Vec2::new(16.0 * 9.0, 16.0 * 1.0),
						2 => Vec2::new(16.0 * 8.0, 16.0 * 2.0),
						_ => Vec2::new(16.0 * 8.0, 16.0 * 1.0),
					}
				}
			} else {
				plain
			};
			commands.spawn((
				Sprite {
					image: asset_server.load("images/forest_.png"),
					rect: Some(Rect::new(
						positions.x,
						positions.y,
						positions.x + 16.0,
						positions.y + 16.0,
					)),
					..default()
				},
				Transform::from_translation(Vec3::new(
					(x * texture_size.x as i32) as f32,
					(y * texture_size.y as i32) as f32,
					-10.0,
				)),
				StateScoped(GameState::Game),
			));
		}
	}
}

fn camera_sync(
	mut camera: Single<&mut Transform, With<Camera2d>>,
	player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
) {
	camera.translation = player.translation;
}

pub struct MainGamePlugin;
impl Plugin for MainGamePlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<StartChainReaction>()
			.add_systems(OnEnter(GameState::Game), setup)
			.init_resource::<Score>()
			.add_systems(
				Update,
				(
					move_player,
					move_enemy,
					prevent_enemies_from_collision,
					chain_slow_down,
					move_enemy_2,
					randomly_change_max_internal_velocity,
					camera_sync.after(move_player),
				)
					.run_if(in_state(GameState::Game))
					.run_if(in_state(PauseMenu::Unpaused)),
			)
			.add_systems(
				Update,
				(
					despawn,
					(draw_chains, enemy_chainable_graphic, draw_chain_balance)
						.run_if(in_state(GameState::Game)),
				)
					.chain(),
			)
			.add_systems(
				Update,
				start_chain_reaction.run_if(in_state(GameState::Game)),
			)
			.add_systems(OnEnter(GameState::Game), setup_tilemap);
	}
}

fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<ColorMaterial>>,
) {
	commands.insert_resource(RadiusCircleAsset(
		MeshMaterial2d(
			materials.add(Color::from(css::CORNFLOWER_BLUE).with_alpha(0.1)),
		),
		Mesh2d(meshes.add(Circle::new(100.0))),
	));

	commands.spawn((
		Camera2d,
		Camera {
			sub_camera_view: Some(SubCameraView {
				full_size: UVec2::new(1920, 1080),
				offset: Vec2::new(640.0, 0.0),
				size: UVec2::new(1280, 720),
			}),
			order: 1,
			..default()
		},
		OrthographicProjection::default_2d().compute_frustum(&GlobalTransform::from(
			Transform::default().with_scale(Vec3::splat(1.5)),
		)),
		FrustumCulling(false),
		StateScoped(GameState::Game),
	));

	commands.insert_resource(ChainAsset(asset_server.load("images/pink_chain.png")));
}

fn start_chain_reaction(
	event_reader: EventReader<StartChainReaction>,
	query: Query<(Entity, &Chained)>,
	player: Single<Entity, With<Player>>,
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut res: ResMut<LastEntityChained>,
	mut materials: ResMut<Assets<FireParticleMaterial>>,
) {
	if event_reader.is_empty() {
		return;
	}
	let mut last_chained_entity = None;
	for (e, Chained { prev }) in query.iter() {
		if *prev == *player {
			last_chained_entity = Some(e);
		}
		println!("{}, chained: {}", e, prev);
	}

	let Some(mut last_chained_entity) = last_chained_entity else {
		commands.spawn(AudioPlayer::new(asset_server.load("audio/error.ogg")));
		return; // no chained entities at all lol
	};
	let mut entities_to_destroy = vec![];
	loop {
		entities_to_destroy.push(last_chained_entity);
		let mut changed = false;
		for (e, chained) in query.iter() {
			if chained.prev == last_chained_entity {
				last_chained_entity = e;
				changed = true;
			}
		}
		if !changed {
			break;
		}
	}
	println!("entities to destroy: {:?}", entities_to_destroy);

	// Bodge because my algorithm is bad somehow and is missing some entities
	for (e, _) in query.iter() {
		if !entities_to_destroy.contains(&e) {
			entities_to_destroy.push_back(e);
		}
	}
	let material_handle = materials.add(FireParticleMaterial {
		texture: asset_server.load("images/noise.png"),
	});

	commands.spawn(());
	//let awa = asset_server.load_untyped("shaders/ice.particle.ron");
	res.0 = *player;
	commands.spawn_task(move || async {
		let mut combo = 1;
		//let awa = awa;
		for (i, entity) in entities_to_destroy.into_iter().enumerate() {
			AsyncWorld.send_event(SlimeDestroyed).unwrap();
			let sleep_duration =
				Duration::from_secs_f32((0.5 / 1.2_f32.powf(i as f32)).max(0.05));
			println!("I AM HERE");
			AsyncWorld.sleep(sleep_duration).await;
			AsyncWorld.run(|world: &mut World| {
				world
					.run_system_once(
						move |mut commands: Commands,
						      asset_server: Res<AssetServer>| {
							commands.spawn((
								AudioPlayer::new(
									asset_server.load("audio/slime-squish.ogg"),
								),
								PlaybackSettings::ONCE
									.with_speed(
										0.9 / (3.0 / 1.1_f32.powf(i as f32)).max(0.3),
									)
									.with_volume(Volume::Linear(5.5)),
							));
						},
					)
					.unwrap();
			});
			AsyncWorld.run(move |world: &mut World| {
				world
					.run_system_once(
						move |mut commands: Commands,
						      particles: Res<Assets<Particle2dEffect>>,
						      asset_server: Res<AssetServer>,
						      mut materials: ResMut<Assets<FireParticleMaterial>>,
						      transforms: Query<&GlobalTransform>,
						      enemies: Query<&Enemy>| {
							let material_handle = materials.add(FireParticleMaterial {
								texture: asset_server.load("images/noise.png"),
							});
							let Ok(enemy) = enemies.get(entity) else {
								return;
							};
							let color = Color::from(*enemy);
							commands.spawn((
								ParticleEffectHandle(asset_server.add(
									Particle2dEffect {
										spawn_rate: 0.0,
										spawn_amount: 50,
										emission_shape: Default::default(), // Equivalent to Point
										lifetime: Rval::new(0.3, 0.5),
										linear_speed: Some(Rval::new(25.0, 25.0)),
										linear_acceleration: Some(Rval::new(
											-1.0, -1.5,
										)),
										direction: Some(Rval::new(
											Vec2::new(0.1, 0.1),
											0.314,
										)),
										angular_speed: Some(Rval::new(200.0, 300.0)),
										angular_acceleration: Some(Rval::new(
											-300.0, -200.0,
										)),
										gravity_direction: None,
										gravity_speed: None,
										scale: Some(Rval::new(0.0, 100.0)),
										linear_damp: Some(Rval::new(0.8, 20.0)),
										angular_damp: Some(Rval::new(0.0, 10.0)),
										scale_curve: Some(MultiCurve {
											points: vec![
												(10.0, 0.0, None),
												(30.0, 1.0, Some(BounceOut)),
											],
										}),
										color: Some(color.into()),
										color_curve: None,
									},
								)),
								OneShot::Despawn,
								ParticleSpawnerState::default(),
								ParticleSpawner(material_handle),
								transforms.get(entity).unwrap().compute_transform(),
							));
						},
					)
					.unwrap();
			});
			AsyncWorld.sleep_frames(5).await;
			AsyncWorld.spawn_bundle((
				AsyncWorld
					.entity(entity)
					.query::<&Transform>()
					.get(|a| a.clone())
					.unwrap(),
				Text2d::new(combo.to_string()),
				TextLayout::new_with_justify(JustifyText::Center),
				TextCombo,
			));
			AsyncWorld.entity(entity).despawn();
			combo += 2;
		}
		AsyncWorld.resource_scope(|mut score: Mut<Score>| {
			score.0 += combo;
		});
		Ok(())
	});
}

#[derive(Event)]
pub struct AddToScore(u32);

fn chain_slow_down(mut query: Query<&mut Velocity, With<Chained>>) {
	for mut v in query.iter_mut() {
		v.0 *= 1.06;
	}
}

#[derive(Clone, Copy, Debug)]
pub enum EnemyColor {
	Red,
	Green,
	Blue,
}
impl EnemyColor {
	pub fn random() -> Self {
		match random!(0..3) {
			0 => Self::Red,
			1 => Self::Green,
			2 => Self::Blue,
			_ => unreachable!(),
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub enum EnemyPolarity {
	Positive,
	Negative,
}
impl EnemyPolarity {
	pub fn random() -> Self {
		match random!(0..2) {
			0 => Self::Positive,
			1 => Self::Negative,
			_ => unreachable!(),
		}
	}
}

#[derive(Component, Clone, Copy, Debug)]
#[component(immutable)]
#[component(on_insert = on_insert_enemy)]
pub struct Enemy {
	enemy_color: EnemyColor,
	enemy_polarity: EnemyPolarity,
}
fn on_insert_enemy(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
	world.commands().queue(move |world: &mut World| {
		world
			.run_system_once(
				move |mut commands: Commands,
				      mut enemy: Query<(&mut Sprite, &Enemy)>| {
					let (mut sprite, enemy) = enemy.get_mut(entity).unwrap();
					sprite.color = (*enemy).into();
				},
			)
			.unwrap();
	});
}

impl Enemy {
	pub fn random() -> Enemy {
		Enemy {
			enemy_color: EnemyColor::random(),
			enemy_polarity: EnemyPolarity::random(),
		}
	}
}

impl From<Enemy> for Color {
	fn from(value: Enemy) -> Self {
		match (value.enemy_color, value.enemy_polarity) {
			(EnemyColor::Red, EnemyPolarity::Positive) => {
				Color::from(css::INDIAN_RED).lighter(0.03)
			}
			(EnemyColor::Red, EnemyPolarity::Negative) => {
				Color::from(css::INDIAN_RED).darker(0.16)
			}
			(EnemyColor::Blue, EnemyPolarity::Positive) => {
				Color::from(css::CORNFLOWER_BLUE)
					.with_saturation(0.98)
					.lighter(0.08)
			}
			(EnemyColor::Blue, EnemyPolarity::Negative) => {
				Color::from(css::CORNFLOWER_BLUE)
					.darker(0.20)
					.with_saturation(0.98)
			}
			(EnemyColor::Green, EnemyPolarity::Positive) => {
				Color::from(css::FOREST_GREEN)
					.lighter(0.05)
					.with_saturation(1.01)
			}
			(EnemyColor::Green, EnemyPolarity::Negative) => {
				Color::from(css::FOREST_GREEN).darker(0.05)
			}
		}
	}
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Velocity(pub Vec3);

#[derive(Component)]
pub struct MaxInternalVelocity(pub f32);
impl MaxInternalVelocity {
	pub fn random() -> Self {
		Self(random!(0.7..1.0))
	}
}

fn prevent_enemies_from_collision(
	enemy_positions: Query<(Entity, &GlobalTransform), With<Enemy>>,
	mut velocities: Query<&mut Velocity>,
) {
	const REPULSION_DISTANCE: f32 = 30.0;
	for (e1, p1) in enemy_positions.iter() {
		for (e2, p2) in enemy_positions.iter() {
			if e2 == e1 {
				continue;
			}
			if p1.translation().distance(p2.translation()) < REPULSION_DISTANCE {
				let awa = (p1.translation() - p2.translation()) / 350.0;
				let awa = awa.lerp(Vec3::default(), 0.6);
				velocities.get_mut(e1).unwrap().0 += awa;
				velocities.get_mut(e2).unwrap().0 -= awa;
			}
		}
	}
}

fn randomly_change_max_internal_velocity(mut query: Query<&mut MaxInternalVelocity>) {
	for mut v in query.iter_mut() {
		if random!(0.0..1.0) < 0.01 {
			*v = MaxInternalVelocity::random();
		}
	}
}

fn move_enemy(
	mut enemy: Query<
		(&mut Velocity, &GlobalTransform, &MaxInternalVelocity),
		With<Enemy>,
	>,
	player: Single<&GlobalTransform, With<Player>>,
) {
	for (mut e, p, v) in enemy.iter_mut() {
		let n = player.translation() - p.translation();
		e.0 = e.0.lerp(n.normalize_or_zero() * v.0, 0.1);
	}
}

fn move_enemy_2(mut enemy: Query<(&mut Transform, &Velocity)>) {
	for (mut t, v) in enemy.iter_mut() {
		t.translation.add_assign(v.0);
	}
}

#[derive(Resource)]
pub struct LastEntityChained(pub Entity);

#[derive(Event)]
pub struct StartChainReaction;

#[derive(Resource, Default)]
pub struct Score(u32);

fn draw_chain_balance(
	mut commands: Commands,
	chained: Query<&Enemy, With<Chained>>,
	keyboard: Res<ButtonInput<KeyCode>>,
	asset_server: Res<AssetServer>,
	mut start_chain_reaction: EventWriter<StartChainReaction>,
	score: Res<Score>,
) {
	let mut greens: i32 = 0;
	let mut reds: i32 = 0;
	let mut blues: i32 = 0;
	for enemy in chained.iter() {
		let mut val = match enemy.enemy_polarity {
			EnemyPolarity::Positive => 1,
			EnemyPolarity::Negative => -1,
		};
		match enemy.enemy_color {
			EnemyColor::Red => reds += val,
			EnemyColor::Green => greens += val,
			EnemyColor::Blue => blues += val,
		}
	}

	let e = commands
		.spawn((
			Node {
				// You can change the `Node` however you want at runtime
				position_type: PositionType::Absolute,
				width: Val::Percent(100.0),
				height: Val::Percent(15.0),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				flex_direction: FlexDirection::Column,
				row_gap: Val::Px(20.0),
				column_gap: Val::Px(20.0),
				..default()
			},
			children![
				Text(format!("Score: {}  ", score.0)),
				Text("Needed to balance: ".to_string()),
			],
			Despawn,
			Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
		))
		.id();
	let sub_node = commands
		.spawn((
			Node {
				// You can change the `Node` however you want at runtime
				position_type: PositionType::Relative,
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				flex_direction: FlexDirection::Row,
				row_gap: Val::Px(20.0),
				column_gap: Val::Px(20.0),
				..default()
			},
			Transform::from_translation(Vec3::new(0.0, -150.0, 0.0)),
			ChildOf(e),
		))
		.id();
	//commands.entity(e).add_child(sub_node);
	let e = sub_node;
	for _ in 0..reds.abs() {
		let enemy = Enemy {
			enemy_color: EnemyColor::Red,
			enemy_polarity: if reds.is_negative() {
				EnemyPolarity::Positive
			} else {
				EnemyPolarity::Negative
			},
		};
		commands.spawn((
			Node {
				width: Val::Px(20.0),
				height: Val::Px(20.0),
				..default()
			},
			BackgroundColor(enemy.into()),
			ChildOf(e),
		));
	}
	for _ in 0..blues.abs() {
		let enemy = Enemy {
			enemy_color: EnemyColor::Blue,
			enemy_polarity: if blues.is_negative() {
				EnemyPolarity::Positive
			} else {
				EnemyPolarity::Negative
			},
		};
		commands.spawn((
			Node {
				width: Val::Px(20.0),
				height: Val::Px(20.0),
				..default()
			},
			BackgroundColor(enemy.into()),
			ChildOf(e),
		));
	}
	for _ in 0..greens.abs() {
		let enemy = Enemy {
			enemy_color: EnemyColor::Green,
			enemy_polarity: if greens.is_negative() {
				EnemyPolarity::Positive
			} else {
				EnemyPolarity::Negative
			},
		};
		commands.spawn((
			Node {
				width: Val::Px(20.0),
				height: Val::Px(20.0),
				..default()
			},
			BackgroundColor(enemy.into()),
			ChildOf(e),
		));
	}
	if keyboard.just_pressed(KeyCode::Space) {
		if reds == 0 && greens == 0 && blues == 0 {
			start_chain_reaction.write(StartChainReaction);
		} else {
			commands.spawn(AudioPlayer::new(asset_server.load("audio/error.ogg")));
		}
	}
}

fn on_click_enemy(
	mut trigger: Trigger<Pointer<Pressed>>,
	mut chained_enemies: Query<&mut Chained>,
	mut player_state: Single<&mut PlayerState>,
	primary_window: Single<&Window, With<PrimaryWindow>>,
	player: Single<Entity, With<Player>>,
	enemies: Query<Entity, (With<EnemyClickable>, Without<Chained>)>,
	mut commands: Commands,
	mut last_entity_chained: ResMut<LastEntityChained>,
	asset_server: Res<AssetServer>,
) {
	player_state.animation_state = AnimationState::Attack;
	if let Some(cursor_position) = primary_window.cursor_position() {
		if cursor_position.x - (primary_window.size().x / 2.0) < 0.0 {
			player_state.direction = Direction::Left;
		}
		if cursor_position.x - (primary_window.size().x / 2.0) > 0.0 {
			player_state.direction = Direction::Right;
		}
	}
	println!("tried click");
	let Ok(enemy) = enemies.get(trigger.target) else {
		return;
	};
	trigger.propagate(false);
	if *player != last_entity_chained.0 {
		let mut chain = chained_enemies
			.get_mut(last_entity_chained.0.entity())
			.unwrap();
		chain.prev = enemy;
	}
	commands.entity(enemy).insert(Chained { prev: *player });
	commands.entity(enemy).remove::<EnemyClickable>();
	last_entity_chained.0 = enemy;
	commands.spawn(AudioPlayer::new(
		asset_server.load(format!("audio/enemy-attach-{}.ogg", random!(1..5))),
	));
	println!("added chain: {}", enemy);
}

#[derive(Component)]
pub struct EnemyClickable;
const DISTANCE_FOR_INTERACTION: f32 = 250.0;
fn enemy_chainable_graphic(
	mut commands: Commands,
	enemies: Query<(Entity, &GlobalTransform), With<Enemy>>,
	player: Single<&GlobalTransform, With<Player>>,
) {
	for (enemy_entity, enemy_transform) in enemies.iter() {
		if player.translation().distance(enemy_transform.translation())
			<= DISTANCE_FOR_INTERACTION
		{
			commands.entity(enemy_entity).try_insert(EnemyClickable);
		} else {
			commands.entity(enemy_entity).remove::<EnemyClickable>();
		}
	}
}

fn on_mouse_no_longer_over_enemy(
	trigger: Trigger<Pointer<Out>>,
	mut query: Query<(Entity, &mut Sprite, &Enemy)>,
	chained: Query<&Chained>,
) {
	let (entity, mut sprite, enemy) = query.get_mut(trigger.target()).unwrap();
	sprite.color = Color::from(*enemy);
	/*if chained.contains(entity) {
		sprite.color = sprite.color.with_saturation(0.2);
	}*/
}

fn on_mouse_over_enemy(
	trigger: Trigger<Pointer<Over>>,
	mut query: Query<(&mut Sprite, &Enemy), With<EnemyClickable>>,
	chained: Query<&Chained>,
) {
	if chained.contains(trigger.target()) {
		return;
	}
	let Ok((mut sprite, enemy)) = query.get_mut(trigger.target()) else {
		return;
	};
	sprite.color = Color::from(*enemy).lighter(0.1);
}

const MAP_RADI: Vec2 = Vec2::new(4096.0, 4096.0);

fn move_player(
	mut player: Single<&mut Transform, With<Player>>,
	mut player_state: Single<&mut PlayerState>,
	keyboard: ResMut<ButtonInput<KeyCode>>,
	mut velocity: Local<Vec3>,
) {
	let mut player_state: &mut PlayerState = &mut player_state;

	// Acceleration parameter (units per second^2)
	const ACCELERATION: f32 = 0.1;
	const SPEED: f32 = 4.0;

	let mut change = Vec2::default();
	if keyboard.pressed(KeyCode::KeyA) {
		change.x -= 1.0;
	}
	if keyboard.pressed(KeyCode::KeyD) {
		change.x += 1.0;
	}
	if keyboard.pressed(KeyCode::KeyW) {
		change.y += 1.0;
	}
	if keyboard.pressed(KeyCode::KeyS) {
		change.y -= 1.0;
	}

	if change.length() != 0.0 && player_state.animation_state != AnimationState::Attack
	{
		player_state.animation_state = AnimationState::Walking;
	}
	if change.length() == 0.0 && player_state.animation_state != AnimationState::Attack
	{
		player_state.animation_state = AnimationState::Idle;
	}
	if change.x < 0.0 {
		player_state.direction = Direction::Left
	}
	if change.x > 0.0 {
		player_state.direction = Direction::Right
	}

	let change = change.normalize_or_zero() * SPEED;
	let change = change.extend(0.0);

	for _ in 0..2 {
		*velocity = velocity.lerp(change, ACCELERATION);
	}
	if velocity.distance(change) <= 0.1 {
		*velocity = change;
	}

	player.translation += *velocity;

	const BUFFER: f32 = 1920.0;

	if player.translation.x > MAP_RADI.x - BUFFER {
		player.translation.x = MAP_RADI.x - BUFFER;
	}
	if player.translation.x < -MAP_RADI.x + BUFFER {
		player.translation.x = -MAP_RADI.x + BUFFER;
	}
	if player.translation.y > MAP_RADI.y - BUFFER {
		player.translation.y = MAP_RADI.y - BUFFER;
	}
	if player.translation.y < -MAP_RADI.y + BUFFER {
		player.translation.y = -MAP_RADI.y + BUFFER;
	}
}

#[derive(Resource, Deref, DerefMut)]
pub struct ChainAsset(pub Handle<Image>);

#[derive(Resource)]
pub struct RadiusCircleAsset(pub MeshMaterial2d<ColorMaterial>, pub Mesh2d);

#[derive(Component)]
pub struct Chained {
	prev: Entity,
}

#[derive(Component)]
pub struct Despawn;

fn draw_chains(
	mut commands: Commands,
	chained: Query<(Entity, &Chained)>,
	positions: Query<&GlobalTransform>,
	chain_asset: Res<ChainAsset>,
) {
	const CHAIN_SIZE: f32 = 12.0 * 1.0;
	for (entity, chained) in chained.iter() {
		let Ok(position_1) = positions.get(entity) else {
			continue;
		};
		let Ok(position_2) = positions.get(chained.prev) else {
			continue;
		};
		let delta = position_1.translation() - position_2.translation();
		// angle in radians around Z‐axis (so sprite “points” from A→B)
		let angle = delta.y.atan2(delta.x);
		let distance = position_1.translation().distance(position_2.translation());
		let temp = (distance / CHAIN_SIZE);
		let mut distance = (distance / CHAIN_SIZE) as u32;
		let remainder = temp - distance as f32;
		let mut flag = false;
		if distance <= 4 {
			flag = true;
			distance = 6;
		}
		for chain in 1..distance {
			let mut chain = position_1
				.translation()
				.lerp(position_2.translation(), (chain as f32 / distance as f32));
			chain.z = 1.0;
			let t = if flag {
				Vec3::splat(0.8)
			} else {
				Vec3::splat(1.0)
			};
			let size = t * ((remainder / distance as f32) + 1.0);
			commands.spawn((
				Sprite {
					image: chain_asset.0.clone(),
					..default()
				},
				Transform::from_translation(chain)
					.with_rotation(Quat::from_euler(
						EulerRot::XYZ,
						0.0,
						0.0,
						PI / 2.0 + angle,
					))
					.with_scale(size),
				Despawn,
			));
		}
	}
}

fn despawn(mut commands: Commands, query: Query<Entity, With<Despawn>>) {
	for entity in query.iter() {
		commands.entity(entity).despawn();
	}
}
