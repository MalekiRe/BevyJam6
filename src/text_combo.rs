use bevy::prelude::*;
use bevy_defer::{AsyncAccess, AsyncCommandsExtension, AsyncWorld};

pub struct TextComboPlugin;
impl Plugin for TextComboPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, on_text_combo_added);
	}
}

fn on_text_combo_added(mut commands: Commands, query: Query<Entity, Added<TextCombo>>) {
	for text_combo in query.iter() {
		commands.spawn_task(move || async move {
			for i in 1..(60 * 2) {
				AsyncWorld
					.entity(text_combo)
					.component::<Transform>()
					.translate_by(Vec3::new(0.0, ((120.0) / i as f32) / 10.0, 0.0))?;
				AsyncWorld.yield_now().await;
			}
			for _ in 0..30 {
				AsyncWorld
					.entity(text_combo)
					.component::<TextColor>()
					.get_mut(|text_color| {
						text_color.0.set_alpha(text_color.0.alpha() - 0.05);
					})?;
				AsyncWorld.yield_now().await;
			}
			AsyncWorld.entity(text_combo).despawn();
			Ok(())
		});
	}
}

#[derive(Component)]
pub struct TextCombo;
