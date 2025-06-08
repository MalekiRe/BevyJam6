use crate::menus::GameState;
use bevy::audio::Volume;
use bevy::prelude::*;

pub struct MusicPlugin;
impl Plugin for MusicPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup_game_music);
		app.add_systems(Startup, setup_other_music);
		app.add_systems(Update, music_controller);
	}
}

#[derive(Component)]
pub struct NonGameMusic;

#[derive(Component)]
pub struct GameMusic;

fn music_controller(
	game_state: Res<State<GameState>>,
	mut non_game_music: Single<
		&mut AudioSink,
		(With<NonGameMusic>, Without<GameMusic>),
	>,
	mut game_music: Single<&mut AudioSink, With<GameMusic>>,
) {
	if game_state.get() == &GameState::Game {
		non_game_music.set_volume(Volume::Linear(0.001));
		game_music.set_volume(Volume::Linear(0.15));
	} else {
		game_music.set_volume(Volume::Linear(0.001));
		non_game_music.set_volume(Volume::Linear(0.15));
	}
}

fn setup_other_music(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.spawn((
		AudioPlayer::new(asset_server.load("audio/music/Overworld.ogg")),
		PlaybackSettings::LOOP.with_volume(Volume::Linear(0.15)),
		NonGameMusic,
	));
}

fn setup_game_music(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.spawn((
		AudioPlayer::new(asset_server.load("audio/music/Pookatori and Friends.ogg")),
		PlaybackSettings::LOOP.with_volume(Volume::Linear(0.15)),
		GameMusic,
	));
}
