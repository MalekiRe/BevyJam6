use bevy::audio::Volume;
use bevy::prelude::*;

pub struct MusicPlugin;
impl Plugin for MusicPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup_music);
	}
}

fn setup_music(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.spawn((
		AudioPlayer::new(asset_server.load("audio/music/Pookatori and Friends.ogg")),
		PlaybackSettings::LOOP.with_volume(Volume::Linear(0.15)),
	));
}
