mod explosion;

use std::cmp::max;
use std::collections::HashSet;
use std::f32::consts::PI;
use std::hint::unreachable_unchecked;
use std::ops::{Add, AddAssign, DerefMut, Sub};
use std::time::Duration;

use crate::explosion::FireParticleMaterial;
use bevy::asset::Handle;
use bevy::color::Color;
use bevy::color::palettes::css;
use bevy::ecs::children;
use bevy::ecs::component::HookContext;
use bevy::ecs::relationship::OrderedRelationshipSourceCollection;
use bevy::ecs::system::RunSystemOnce;
use bevy::ecs::world::DeferredWorld;
use bevy::image::Image;
use bevy::math::{EulerRot, Quat, Rect, Vec2, Vec3};
use bevy::prelude::{AlignItems, Alpha, AudioPlayer, ButtonInput, ChildOf, Circle, Click, ColorMaterial, ContainsEntity, Entity, Event, EventReader, EventWriter, FlexDirection, GlobalTransform, IVec2, IntoScheduleConfigs, JustifyContent, KeyCode, Local, Luminance, Mesh, Mesh2d, MeshMaterial2d, MeshPickingPlugin, Node, OnAdd, OnRemove, Pointer, PositionType, Pressed, Rectangle, Resource, Saturation, Single, Text, Transform, Trigger, Val, With, Without, World, default, Pickable};
use bevy::prelude::{BackgroundColor, SpawnRelated};
use bevy::sprite::SpriteImageMode;
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
use bevy::audio::{PlaybackSettings, Volume};
use bevy::prelude::EaseFunction::BounceOut;
use bevy_defer::{AsyncCommandsExtension, AsyncWorld};
use bevy_ecs_tilemap::map::TilemapId;
use bevy_ecs_tilemap::prelude::{
	ArrayTextureLoader, TileBundle, TilePos, TilemapAnchor, TilemapArrayTexture,
	TilemapSize, TilemapTexture, TilemapTileSize, TilemapType,
};
use bevy_ecs_tilemap::tiles::TileStorage;
use bevy_ecs_tilemap::{TilemapBundle, TilemapPlugin};
use bevy_enoki::prelude::{MultiCurve, OneShot, Particle2dMaterialPlugin, ParticleEffectInstance, ParticleSpawnerState, Rval};
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
			TilemapPlugin,
		))
		.add_plugins(EnokiPlugin)
		.add_plugins(Particle2dMaterialPlugin::<FireParticleMaterial>::default())
		.add_event::<StartChainReaction>()
		.add_systems(Startup, setup)
		.add_systems(
			Update,
			(
				move_player,
				move_enemy,
				prevent_enemies_from_collision,
				chain_slow_down,
				move_enemy_2,
				randomly_change_max_internal_velocity,
				camera_sync.after(move_player)
			),
		)
		.add_systems(
			Update,
			(
				despawn,
				draw_chains,
				enemy_chainable_graphic,
				draw_chain_balance,
			)
				.chain(),
		)
		.add_systems(Update, start_chain_reaction)
		.add_systems(Startup, setup_tilemap)
		.run();
}

fn setup_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {

	let texture_size = UVec2::new(16, 16);
	let size = IVec2::new(MAP_RADI.x as i32 / 16, MAP_RADI.y as i32 / 16);

	let plain = Vec2::new(40.0, 40.0);
	let grassy = Vec2::new(16.0 * 6.0, 16.0 * 2.0);
	let grassy2 = Vec2::new(16.0 * 7.0, 16.0 * 2.0);

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
					x as f32 * texture_size.x as f32,
					y as f32 * texture_size.y as f32,
					-10.0,
				)),
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

	let color: Color = css::CORNFLOWER_BLUE.into();
	let red: Color = css::INDIAN_RED.into();
	commands.spawn(Camera2d);
	let mut e = commands
		.spawn((
			Mesh2d(meshes.add(Rectangle::new(30.0, 30.0))),
			MeshMaterial2d(materials.add(red)),
			Transform::from_translation(Vec3::new(-100.0, 30.0, 0.0)),
			Player,
		))
		.id();
	let child = commands
		.spawn((
			MeshMaterial2d(
				materials.add(Color::from(css::CORNFLOWER_BLUE).with_alpha(0.05)),
			),
			Mesh2d(meshes.add(Circle::new(DISTANCE_FOR_INTERACTION))),
			Transform::from_translation(Vec3::new(0.0, 0.0, -5.0)),
		))
		.id();
	commands.insert_resource(LastEntityChained(e));
	commands.entity(e).add_child(child);
	for x in 0..20 {
		e = commands
			.spawn((
				Sprite {
					image: asset_server.load("images/slime.png"),
					rect: Some(Rect::new(0.0, 16.0, 16.0 * 2.0, 16.0 * 2.0)),
					//custom_size: Some(Vec2::new(30.0, 30.0)),
					..default()
				},
				Transform::from_translation(Vec3::new(x as f32 * 30.0, 30.0, 0.0)),
				Enemy::random(),
				MaxInternalVelocity::random(),
				Velocity(Vec3::new(0.0, 0.0, 0.0)),
				Pickable::default(),
			))
			.observe(on_clickable_added)
			.observe(on_clickable_removed)
			.observe(on_click_enemy)
			.id();
	}

	commands.insert_resource(ChainAsset(asset_server.load("images/chain.png")));
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
	println!("PLAYER: {}", *player);
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
		//let awa = awa;
		for (i, entity) in entities_to_destroy.into_iter().enumerate() {
			let sleep_duration =
				Duration::from_secs_f32((0.5 / 1.2_f32.powf(i as f32)).max(0.05));

			AsyncWorld.sleep(sleep_duration).await;
			AsyncWorld.run(|world: &mut World| {
				world
					.run_system_once(
						move |mut commands: Commands, asset_server: Res<AssetServer>| {
							commands.spawn((AudioPlayer::new(
								asset_server.load("audio/slime-squish.ogg"),
							), PlaybackSettings::ONCE
								.with_speed(0.9 / (3.0 / 1.1_f32.powf(i as f32)).max(0.3))
								.with_volume(Volume::default().add(Volume::Linear(2.5)))));
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
							let color = Color::from(*enemies.get(entity).unwrap());
							commands.spawn((
								ParticleEffectHandle(asset_server.add(Particle2dEffect {
									spawn_rate: 0.0,
									spawn_amount: 50,
									emission_shape: Default::default(), // Equivalent to Point
									lifetime: Rval::new(0.3, 0.5),
									linear_speed: Some(Rval::new(25.0, 25.0)),
									linear_acceleration: Some(Rval::new(-1.0, -1.5)),
									direction: Some(Rval::new(Vec2::new(0.1, 0.1), 0.314)),
									angular_speed: Some(Rval::new(200.0, 300.0)),
									angular_acceleration: Some(Rval::new(-300.0, -200.0)),
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
								})),
								OneShot::Despawn,
								ParticleSpawnerState::default(),
								ParticleSpawner(material_handle),
								transforms.get(entity).unwrap().compute_transform(),
							));
						},
					)
					.unwrap();
			});
			AsyncWorld.entity(entity).despawn();
		}
		Ok(())
	});
}

fn chain_slow_down(mut query: Query<&mut Velocity, With<Chained>>) {
	for mut v in query.iter_mut() {
		v.0 *= 1.06;
	}
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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

#[derive(Component, Clone, Copy)]
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
		let color = match value.enemy_color {
			EnemyColor::Red => Color::from(css::INDIAN_RED),
			EnemyColor::Green => Color::from(css::FOREST_GREEN),
			EnemyColor::Blue => Color::from(css::CORNFLOWER_BLUE),
		};
		match value.enemy_polarity {
			EnemyPolarity::Positive => color,
			EnemyPolarity::Negative => color.darker(0.13).with_saturation(0.6),
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

fn draw_chain_balance(
	mut commands: Commands,
	chained: Query<&Enemy, With<Chained>>,
	keyboard: Res<ButtonInput<KeyCode>>,
	asset_server: Res<AssetServer>,
	mut start_chain_reaction: EventWriter<StartChainReaction>,
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
				flex_direction: FlexDirection::Row,
				row_gap: Val::Px(20.0),
				column_gap: Val::Px(20.0),
				..default()
			},
			Despawn,
			Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
		))
		.id();
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
		if /*reds == 0 && greens == 0 && blues == 0*/ true {
			start_chain_reaction.write(StartChainReaction);
		} else {
			commands.spawn(AudioPlayer::new(asset_server.load("audio/error.ogg")));
		}
	}
}

fn on_click_enemy(
	mut trigger: Trigger<Pointer<Pressed>>,
	mut chained_enemies: Query<&mut Chained>,
	player: Single<Entity, With<Player>>,
	enemies: Query<Entity, (With<EnemyClickable>, Without<Chained>)>,
	mut commands: Commands,
	mut last_entity_chained: ResMut<LastEntityChained>,
	asset_server: Res<AssetServer>,
) {
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
const DISTANCE_FOR_INTERACTION: f32 = 100.0;
fn enemy_chainable_graphic(
	mut commands: Commands,
	enemies: Query<(Entity, &GlobalTransform), With<Enemy>>,
	player: Single<&GlobalTransform, With<Player>>,
) {
	for (enemy_entity, enemy_transform) in enemies.iter() {
		if player.translation().distance(enemy_transform.translation())
			<= DISTANCE_FOR_INTERACTION
		{
			commands.entity(enemy_entity).insert(EnemyClickable);
		} else {
			commands.entity(enemy_entity).remove::<EnemyClickable>();
		}
	}
}

fn on_clickable_removed(
	trigger: Trigger<OnRemove, EnemyClickable>,
	mut query: Query<(Entity, &mut Sprite, &Enemy)>,
	chained: Query<&Chained>,
) {
	let (entity, mut sprite, enemy) = query.get_mut(trigger.target()).unwrap();
	sprite.color = Color::from(*enemy);
	if chained.contains(entity) {
		sprite.color = sprite.color.with_saturation(0.2);
	}
}

fn on_clickable_added(
	trigger: Trigger<OnAdd, EnemyClickable>,
	mut query: Query<(&mut Sprite, &Enemy)>,
	chained: Query<&Chained>,
) {
	if chained.contains(trigger.target()) {
		return;
	}
	let (mut sprite, enemy) = query.get_mut(trigger.target()).unwrap();
	sprite.color = Color::from(*enemy).lighter(0.1);
}

const MAP_RADI: Vec2 = Vec2::new(4096.0, 4096.0);

fn move_player(
	mut player: Single<&mut Transform, With<Player>>,
	keyboard: ResMut<ButtonInput<KeyCode>>,
	mut velocity: Local<Vec3>,
) {
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
	if player.translation.y > MAP_RADI.y - BUFFER{
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
	const CHAIN_SIZE: f32 = 12.0 * 1.5;
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
				Vec3::splat(1.1)
			} else {
				Vec3::splat(1.5)
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
