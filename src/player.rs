use crate::{LastEntityChained, Player};
use bevy::prelude::*;
use bevy_defer::{AsyncAccess, AsyncCommandsExtension, AsyncWorld};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, (setup_player_spritesheet, setup_player, player_animation_player).chain());
	}
}

fn player_animation_player(
    mut commands: Commands,
	player: Single<Entity, With<Player>>,
	player_sprite_sheet: Res<PlayerSpriteSheet>
) {
    let player = player.clone();
	let sprite_sheet = player_sprite_sheet.clone();
    commands.spawn_task(move || async move {
        let player = AsyncWorld.entity(player);
		let sprite_sheet = sprite_sheet;
        loop {
			'game_state: {
				match player.component::<PlayerState>().get(|t| t.clone())? {
					PlayerState::Idle => {
						player.component::<Sprite>().get_mut(|sprite| {
							*sprite = sprite_sheet.idle.clone();
						})?;
						for i in 0..4 {
							if player.component::<PlayerState>().get(|t| *t != PlayerState::Idle)? {
								break 'game_state;
							}
							player.component::<Sprite>().get_mut(|sprite| {
								sprite.texture_atlas.as_mut().unwrap().index = i
							})?;
							AsyncWorld.sleep_frames(16).await;
						}
					}
					PlayerState::Walking(direction) => {
						match direction {
							Direction::Right => {
								player.component::<Sprite>().get_mut(|sprite| {
									*sprite = sprite_sheet.run.clone();
									sprite.flip_x = true;
								})?;
							}
							_ => {
								player.component::<Sprite>().get_mut(|sprite| {
									*sprite = sprite_sheet.run.clone();
								})?;
							}
						}
					}
					PlayerState::Attack(_) => {

					}
				}
			}
        }
    });
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Default)]
pub enum PlayerState {
	#[default]
	Idle,
	Walking(Direction),
	Attack(Direction),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
	Left,
	Right,
	Up,
	Down,
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
			PlayerState::Idle,
			Transform::from_translation(Vec3::new(-100.0, 30.0, 0.0)),
			Player,
			player_sprite_sheet.idle.clone(),
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
            custom_size: Some(Vec2::new(32.0, 32.0)),
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
			..default()
		},
	});
}
