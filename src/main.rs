use std::f32::consts::PI;
use std::ops::{AddAssign, DerefMut, Sub};
use std::time::Duration;

use bevy::asset::Handle;
use bevy::color::Color;
use bevy::color::palettes::css;
use bevy::image::Image;
use bevy::math::{EulerRot, Quat, Vec2, Vec3};
use bevy::prelude::{
	Alpha, ButtonInput, Circle, ColorMaterial, Entity, GlobalTransform,
	IntoScheduleConfigs, KeyCode, Local, Luminance, Mesh, Mesh2d, MeshMaterial2d,
	OnAdd, OnRemove, Rectangle, Resource, Single, Transform, Trigger, With, default,
};
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
use random_number::random;

fn main() {
	App::new()
		.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
		.add_systems(Startup, setup)
		.add_systems(Update, (move_player, move_enemy, move_enemy_2))
		.add_systems(
			Update,
			(despawn, draw_chains, enemy_chainable_graphic).chain(),
		)
		//.add_systems(Update, animate_sprite)
		.run();
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
			Transform::default(),
		))
		.id();

	commands.entity(e).add_child(child);
	for _ in 0..10 {
		e = commands
			.spawn((
				Mesh2d(meshes.add(Rectangle::new(30.0, 30.0))),
				MeshMaterial2d(materials.add(color)),
				Transform::from_translation(Vec3::new(100.0, 30.0, 0.0)),
				Chained { prev: e },
				Enemy,
				Velocity(Vec3::new(0.0, 0.0, 0.0)),
			))
			.observe(on_clickable_added)
			.observe(on_clickable_removed)
			.id();
	}

	commands.insert_resource(ChainAsset(asset_server.load("images/chain.png")));
}

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Velocity(pub Vec3);

fn move_enemy(mut enemy: Query<&mut Velocity, With<Enemy>>) {
	for mut e in enemy.iter_mut() {
		if random!(0.0..1.0) < 0.001 {
			e.0 = Vec3::new(random!(-1.0..1.0), random!(-1.0..1.0), 0.0);
		}
	}
}

fn move_enemy_2(mut enemy: Query<(&mut Transform, &Velocity)>) {
	for (mut t, v) in enemy.iter_mut() {
		t.translation.add_assign(v.0);
	}
}

#[derive(Component)]
pub struct EnemyClickable;
const DISTANCE_FOR_INTERACTION: f32 = 80.0;
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
	query: Query<&MeshMaterial2d<ColorMaterial>>,
	mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
	let color = query.get(trigger.target()).unwrap();
	color_materials.get_mut(color).unwrap().color = css::CORNFLOWER_BLUE.into();
	color_materials.deref_mut();
}

fn on_clickable_added(
	trigger: Trigger<OnAdd, EnemyClickable>,
	query: Query<&MeshMaterial2d<ColorMaterial>>,
	mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
	let color = query.get(trigger.target()).unwrap();
	let c: Color = css::CORNFLOWER_BLUE.into();
	color_materials.get_mut(color).unwrap().color = c.lighter(0.1);
	color_materials.deref_mut();
}

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
	const CHAIN_SIZE: f32 = 12.0 * 3.0;
	for (entity, chained) in chained.iter() {
		let position_1 = positions.get(entity).unwrap();
		let position_2 = positions.get(chained.prev).unwrap();
		let delta = position_1.translation() - position_2.translation();
		// angle in radians around Z‐axis (so sprite “points” from A→B)
		let angle = delta.y.atan2(delta.x);
		let distance = position_1.translation().distance(position_2.translation());
		let temp = (distance / CHAIN_SIZE);
		let mut distance = (distance / CHAIN_SIZE) as u32;
		let remainder = temp - distance as f32;

		if distance <= 1 {
			distance = 2;
		}
		for chain in 1..distance {
			let mut chain = position_1
				.translation()
				.lerp(position_2.translation(), (chain as f32 / distance as f32));
			chain.z = -1.0;
			let mut size = Vec3::splat(3.0) * ((remainder / distance as f32) + 1.0);
			if distance <= 2 {
				size = Vec3::splat(3.0);
			}
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

/*fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
	commands.spawn(Camera2d);

	let width = 6;
	let layout = TextureAtlasLayout::from_grid(
		// UVec2 { x: 23, y: 27 },
		UVec2 { x: 33, y: 33 },
		width,
		2,
		None,
		None,
	);
	let texture_atlas_layout = texture_atlas_layouts.add(layout);
	let run_animation_idx = AnimationIndices::new(width, 1, 0..width);
	let animation_timer =
		AnimationTimer(Timer::new(Duration::from_millis(100), TimerMode::Repeating));
	commands.spawn((
		Sprite::from_atlas_image(
			asset_server.load("images/ducky.png"),
			TextureAtlas {
				layout: texture_atlas_layout,
				index: run_animation_idx.first,
			},
		),
		run_animation_idx,
		animation_timer,
	));
}

#[derive(Component)]
struct AnimationIndices {
	first: usize,
	last: usize,
}

impl AnimationIndices {
	fn new(width: u32, row: u32, indices: impl Into<std::ops::Range<u32>>) -> Self {
		let indices: std::ops::Range<u32> = indices.into();
		let first = row * width + indices.start;
		let last = first + indices.end - 1;

		Self {
			first: first as usize,
			last: last as usize,
		}
	}
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
	time: Res<Time>,
	mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
	for (indices, mut timer, mut sprite) in &mut query {
		timer.tick(time.delta());

		if timer.just_finished() {
			if let Some(atlas) = &mut sprite.texture_atlas {
				atlas.index = if atlas.index == indices.last {
					indices.first
				} else {
					atlas.index + 1
				};
			}
		}
	}
}*/
