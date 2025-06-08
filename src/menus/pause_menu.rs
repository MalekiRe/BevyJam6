use crate::menus::{GameState, PauseMenu};
use crate::theme::widget;
use bevy::prelude::*;

pub struct PauseMenuPlugin;
impl Plugin for PauseMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(PauseMenu::Paused), spawn_pause_menu);
		app.add_systems(
			Update,
			test_for_pause_menu.run_if(in_state(GameState::Game)),
		);
	}
}

fn test_for_pause_menu(
	keyboard: ResMut<ButtonInput<KeyCode>>,
	current_pause_menu: ResMut<State<PauseMenu>>,
	mut pause_menu: ResMut<NextState<PauseMenu>>,
) {
	if keyboard.just_pressed(KeyCode::Escape) {
		pause_menu.set(match current_pause_menu.get() {
			PauseMenu::Paused => PauseMenu::Unpaused,
			PauseMenu::Unpaused => PauseMenu::Paused,
		});
	}
}

fn spawn_pause_menu(mut commands: Commands) {
	commands.spawn((Camera2d, StateScoped(PauseMenu::Paused)));
	commands.spawn((
		widget::ui_root("Pause Menu"),
		GlobalZIndex(2),
		StateScoped(PauseMenu::Paused),
		children![
			widget::button("Resume", enter_gameplay),
			widget::button("Exit", exit_gameplay),
		],
	));
}

fn enter_gameplay(
	_: Trigger<Pointer<Click>>,
	mut next_menu: ResMut<NextState<GameState>>,
	mut pause_menu: ResMut<NextState<PauseMenu>>,
) {
	pause_menu.set(PauseMenu::Unpaused);
	next_menu.set(GameState::Game);
}

fn exit_gameplay(
	_: Trigger<Pointer<Click>>,
	mut next_menu: ResMut<NextState<GameState>>,
	mut pause_menu: ResMut<NextState<PauseMenu>>,
) {
	next_menu.set(GameState::MainMenu);
	pause_menu.set(PauseMenu::Unpaused);
}
