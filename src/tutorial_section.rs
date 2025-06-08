use crate::menus::GameState;
use crate::theme::widget;
use crate::{Enemy, EnemyColor, EnemyPolarity};
use bevy::prelude::*;
use bevy_defer::{AsyncAccess, AsyncCommandsExtension, AsyncWorld};
use std::f32::consts::PI;

pub struct TutorialPlugin;
impl Plugin for TutorialPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::Tutorial), do_tutorial);
	}
}

fn do_tutorial(mut commands: Commands) {
	commands.spawn((Camera2d, StateScoped(GameState::Tutorial)));
	commands.spawn_task(move || async {
		let e = AsyncWorld.spawn_bundle((
			Text2d::new("Welcome To Chain Wiper"),
			TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
			Transform::from_translation(Vec3::new(0.0, 45.0, 0.0)),
			StateScoped(GameState::Tutorial)
		));
		let color = e.component::<TextColor>();
		for i in 0..128 {
			color.get_mut(|c| c.0.set_alpha(i as f32 / 128.0))?;
			AsyncWorld.yield_now().await;
		}
		let e2 = AsyncWorld.spawn_bundle((
			Text2d::new("In this game there are slimes of opposite polarities"),
			TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
			StateScoped(GameState::Tutorial)
		));
		let color = e2.component::<TextColor>();
		for i in 0..128 {
			e.component::<Transform>()
				.translate_by(Vec3::new(0.0, 1.0, 0.0))?;
			color.get_mut(|c| c.0.set_alpha(i as f32 / 128.0))?;
			AsyncWorld.yield_now().await;
		}

		for _ in 0..128 {
			AsyncWorld.yield_now().await;
			e2.component::<Transform>()
				.translate_by(Vec3::new(0.0, 1.0, 0.0))?;
		}

		let (e1, e2, e3, e4) =
			AsyncWorld.resource_scope(|asset_server: Mut<AssetServer>| {
				let e1 = AsyncWorld.spawn_bundle((
					Sprite {
						image: asset_server.load("images/slime.png"),
						rect: Some(Rect::new(0.0, 16.0, 16.0 * 2.0, 16.0 * 2.0)),
						color: Color::srgba(1.0, 1.0, 1.0, 0.0),
						..default()
					},
					Transform::from_translation(Vec3::new(-70.0, 0.0, 0.0))
						.with_scale(Vec3::splat(4.0)),
					Enemy {
						enemy_color: EnemyColor::Red,
						enemy_polarity: EnemyPolarity::Positive,
					},
					StateScoped(GameState::Tutorial),
				));

				let e2 = AsyncWorld.spawn_bundle((
					Sprite {
						image: asset_server.load("images/slime.png"),
						rect: Some(Rect::new(0.0, 16.0, 16.0 * 2.0, 16.0 * 2.0)),
						color: Color::srgba(1.0, 1.0, 1.0, 0.0),
						..default()
					},
					Transform::from_translation(Vec3::new(70.0, 0.0, 0.0))
						.with_scale(Vec3::splat(4.0)),
					Enemy {
						enemy_color: EnemyColor::Red,
						enemy_polarity: EnemyPolarity::Negative,
					},
					StateScoped(GameState::Tutorial),
				));

				let e3 = AsyncWorld.spawn_bundle((
					Sprite {
						image: asset_server.load("images/chain.png"),
						color: Color::srgba(1.0, 1.0, 1.0, 0.0),
						..default()
					},
					Transform::from_translation(Vec3::new(25.0, 0.0, -1.0))
						.with_scale(Vec3::splat(4.0))
						.with_rotation(Quat::from_euler(
							EulerRot::XYZ,
							0.0,
							0.0,
							PI / 2.0,
						)),
					StateScoped(GameState::Tutorial),
				));

				let e4 = AsyncWorld.spawn_bundle((
					Sprite {
						image: asset_server.load("images/chain.png"),
						color: Color::srgba(1.0, 1.0, 1.0, 0.0),
						..default()
					},
					Transform::from_translation(Vec3::new(-25.0, 0.0, -1.0))
						.with_scale(Vec3::splat(4.0))
						.with_rotation(Quat::from_euler(
							EulerRot::XYZ,
							0.0,
							0.0,
							PI / 2.0,
						)),
					StateScoped(GameState::Tutorial),
				));

				(e1, e2, e3, e4)
			});

		for i in 0..128 {
			let val = i as f32 / 128.0;
			e2.component::<Sprite>()
				.get_mut(|s| s.color.set_alpha(val))
				.unwrap();
			e1.component::<Sprite>()
				.get_mut(|s| s.color.set_alpha(val))
				.unwrap();
			AsyncWorld.yield_now().await;
		}

		let e2 = AsyncWorld.spawn_bundle((
			Text2d::new(
				"Your chain must be balanced\n\
            For every dark red slime, you must have a light red slime\n\
            For every dark blue slime you must have a light blue slime",
			),
			TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
			Transform::from_translation(Vec3::new(0.0, -200.0, 0.0)),
			StateScoped(GameState::Tutorial),
		));
		let color = e2.component::<TextColor>();
		for i in 0..128 {
			e2.component::<Transform>()
				.translate_by(Vec3::new(0.0, 0.5, 0.0))?;
			color.get_mut(|c| c.0.set_alpha(i as f32 / 128.0))?;
			AsyncWorld.yield_now().await;
		}


		for i in 0..64 {
			let val = i as f32 / 64.0;
			e3.component::<Sprite>()
				.get_mut(|s| s.color.set_alpha(val))
				.unwrap();
			e4.component::<Sprite>()
				.get_mut(|s| s.color.set_alpha(val))
				.unwrap();
			AsyncWorld.yield_now().await;
		}

		let e2 = AsyncWorld.spawn_bundle((
			Text2d::new(
				"Press space to start the chain reaction",
			),
			TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
			Transform::from_translation(Vec3::new(0.0, -265.0, 0.0)),
			StateScoped(GameState::Tutorial),
		));
		let color = e2.component::<TextColor>();
		for i in 0..64 {
			e2.component::<Transform>()
				.translate_by(Vec3::new(0.0, 1.0, 0.0))?;
			color.get_mut(|c| c.0.set_alpha(i as f32 / 64.0))?;
			AsyncWorld.yield_now().await;
		}

		let e2 = AsyncWorld.spawn_bundle((
			Text2d::new("Good luck"),
			TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
			Transform::from_translation(Vec3::new(0.0, -275.0, 0.0)),
			StateScoped(GameState::Tutorial),
		));
		let color = e2.component::<TextColor>();
		for i in 0..64 {
			color.get_mut(|c| c.0.set_alpha(i as f32 / 128.0))?;
			AsyncWorld.yield_now().await;
		}

		AsyncWorld.sleep_frames(100).await;

		AsyncWorld.spawn_bundle((
			widget::ui_root("Awa Menu"),
			GlobalZIndex(2),
			StateScoped(GameState::Tutorial),
			children![widget::button("Start", settings_menu),],
			Transform::from_translation(Vec3::new(0.0, -450.0, 0.0)),
		));

		Ok(())
	});
}

fn settings_menu(
	_: Trigger<Pointer<Click>>,
	mut next_menu: ResMut<NextState<GameState>>,
) {
	next_menu.set(GameState::Settings);
}
