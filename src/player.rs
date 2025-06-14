use crate::menus::GameState;
use crate::{Enemy, LastEntityChained, Player};
use bevy::color::palettes::css;
use bevy::math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume};
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy_defer::{AsyncAccess, AsyncCommandsExtension, AsyncWorld};
use std::ops::Mul;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			OnEnter(GameState::Game),
			(
				setup_player_spritesheet,
				setup_player,
				player_animation_player,
			)
				.chain(),
		);
		app.add_systems(Update, handle_hit);
	}
}

fn handle_hit(
	player: Single<(&Aabb, &Transform), With<Player>>,
	slimes: Query<(&Aabb, &Transform), With<Enemy>>,
	mut gizmos: Gizmos,
	mut game_state: ResMut<NextState<GameState>>,
	asset_server: Res<AssetServer>,
	mut commands: Commands,
) {
	let mut player_aabb =
		Aabb2d::new(player.0.center.xy(), player.0.half_extents.xy() / 3.5);
	player_aabb.translate_by(player.1.translation.xy());
	/*gizmos.rect_2d(
		Isometry2d::new(player_aabb.center(), Rot2::default()),
		player_aabb.half_size().mul(Vec2::splat(2.0)),
		css::FOREST_GREEN,
	);*/
	for (slime_aabb, slime_transform) in slimes.iter() {
		let mut slime_aabb = Aabb2d::new(
			slime_aabb.center.xy(),
			slime_aabb.half_extents.xy() * Vec2::new(0.45, 0.8),
		);
		slime_aabb.translate_by(slime_transform.translation.xy());
		/*gizmos.rect_2d(
			Isometry2d::new(slime_aabb.center(), Rot2::default()),
			slime_aabb.half_size().mul(Vec2::splat(2.0)),
			css::CORNFLOWER_BLUE,
		);*/
		if slime_aabb.intersects(&player_aabb) {
			game_state.set(GameState::Shop);
			commands.spawn(AudioPlayer::new(asset_server.load("audio/die.ogg")));
		}
	}
}

fn player_animation_player(
	mut commands: Commands,
	player: Single<Entity, With<Player>>,
	player_sprite_sheet: Res<PlayerSpriteSheet>,
) {
	let player = player.clone();
	let sprite_sheet = player_sprite_sheet.clone();
	commands.spawn_task(move || async move {
		let player = AsyncWorld.entity(player);
		let sprite_sheet = sprite_sheet;
		loop {
			'game_state: {
				let player_state =
					player.component::<PlayerState>().get(|t| t.clone())?;
				match player_state.animation_state {
					AnimationState::Idle => {
						player.component::<Sprite>().get_mut(|sprite| {
							*sprite = sprite_sheet.idle.clone();
							sprite.flip_x = player_state.direction == Direction::Left;
						})?;
						for i in 0..4 {
							if player
								.component::<PlayerState>()
								.get(|t| *t != player_state)?
							{
								break 'game_state;
							}
							player.component::<Sprite>().get_mut(|sprite| {
								sprite.texture_atlas.as_mut().unwrap().index = i
							})?;
							AsyncWorld.sleep_frames(16).await;
						}
					}
					AnimationState::Walking => {
						player.component::<Sprite>().get_mut(|sprite| {
							*sprite = sprite_sheet.run.clone();
							sprite.flip_x = player_state.direction == Direction::Left;
						})?;
						for i in 0..8 {
							if player
								.component::<PlayerState>()
								.get(|t| *t != player_state)?
							{
								break 'game_state;
							}
							player.component::<Sprite>().get_mut(|sprite| {
								sprite.texture_atlas.as_mut().unwrap().index = i
							})?;
							AsyncWorld.sleep_frames(8).await;
						}
					}
					AnimationState::Attack => {
						player.component::<Sprite>().get_mut(|sprite| {
							*sprite = sprite_sheet.attack.clone();
							sprite.flip_x = player_state.direction == Direction::Left;
						})?;
						for i in 0..4 {
							if player
								.component::<PlayerState>()
								.get(|t| *t != player_state)?
							{
								break 'game_state;
							}
							player.component::<Sprite>().get_mut(|sprite| {
								sprite.texture_atlas.as_mut().unwrap().index = i
							})?;
							AsyncWorld.sleep_frames(4).await;
						}
						player.component::<PlayerState>().get_mut(|t| {
							t.animation_state = AnimationState::Idle;
						})?;
					}
				}
			}
		}
	});
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct PlayerState {
	pub animation_state: AnimationState,
	pub direction: Direction,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AnimationState {
	Idle,
	Walking,
	Attack,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
	Left,
	Right,
}

#[derive(Resource, Clone)]
pub struct PlayerSpriteSheet {
	pub attack: Sprite,
	pub idle: Sprite,
	pub run: Sprite,
}

fn setup_player(mut commands: Commands, player_sprite_sheet: Res<PlayerSpriteSheet>) {
	let e = commands
		.spawn((
			PlayerState {
				animation_state: AnimationState::Idle,
				direction: Direction::Right,
			},
			Transform::from_translation(Vec3::new(-100.0, 30.0, 0.0)),
			Player,
			player_sprite_sheet.idle.clone(),
			StateScoped(GameState::Game),
		))
		.id();
	/*let child = commands
	.spawn((
		MeshMaterial2d(
			materials.add(Color::from(css::CORNFLOWER_BLUE).with_alpha(0.05)),
		),
		Mesh2d(meshes.add(Circle::new(DISTANCE_FOR_INTERACTION))),
		Transform::from_translation(Vec3::new(0.0, 0.0, -5.0)),
	))
	.id();*/
	commands.insert_resource(LastEntityChained(e));
	//commands.entity(e).add_child(child);
}

fn setup_player_spritesheet(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
	commands.insert_resource(PlayerSpriteSheet {
		attack: Sprite {
			image: asset_server.load("images/player/attack.png"),
			texture_atlas: Some(TextureAtlas {
				layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
					UVec2::new(40, 39),
					1,
					8,
					None,
					Some(UVec2::new(48, 0)),
				)),
				index: 0,
			}),
			custom_size: Some(Vec2::new(64.0, 64.0)),
			..default()
		},
		idle: Sprite {
			image: asset_server.load("images/player/idle.png"),
			texture_atlas: Some(TextureAtlas {
				layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
					UVec2::new(40, 39),
					1,
					5,
					None,
					Some(UVec2::new(48, 0)),
				)),
				index: 0,
			}),
			custom_size: Some(Vec2::new(64.0, 64.0)),
			..default()
		},
		run: Sprite {
			image: asset_server.load("images/player/run.png"),
			texture_atlas: Some(TextureAtlas {
				layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
					UVec2::new(40, 39),
					1,
					8,
					None,
					Some(UVec2::new(48, 0)),
				)),
				index: 0,
			}),
			custom_size: Some(Vec2::new(64.0, 64.0)),
			..default()
		},
	});
}
