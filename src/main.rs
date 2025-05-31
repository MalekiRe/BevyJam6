use std::time::Duration;

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

fn main() {
	App::new()
		.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
		.add_systems(Startup, setup)
		.add_systems(Update, animate_sprite)
		.run();
}

fn setup(
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
}
