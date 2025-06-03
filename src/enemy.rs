use std::time::Duration;
use bevy::prelude::*;
use bevy_defer::{AsyncAccess, AsyncCommandsExtension, AsyncWorld};
use random_number::random;
use crate::{on_click_enemy, on_clickable_added, on_clickable_removed, Enemy, MaxInternalVelocity, Player, Velocity, MAP_RADI};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEnemy>();
        app.add_systems(Startup, spawn_enemy_clusters);
        app.add_systems(Update, handle_spawn_enemy);
    }
}

fn spawn_enemy_clusters(
    mut commands: Commands,
) {
    commands.spawn_task(move || async move {
       loop {
           AsyncWorld.sleep(Duration::new(random!(1..8), 0)).await;
           let mut enemy_types = vec![Enemy::random()];
           while random!(0.0..1.0) > 0.4 {
               enemy_types.push(Enemy::random());
           }
           let mut starting_position = Vec2::new(random!(0.01..1.0), random!(0.01..1.0));
           if starting_position.x < 0.5 {
               starting_position.x = -starting_position.x;
           }
           if starting_position.y < 0.5 {
               starting_position.y = -starting_position.y;
           }
           let starting_position = starting_position.try_normalize().unwrap(); // This should always succeed
           let t = AsyncWorld.query_filtered::<&mut Transform, With<Player>>().single();
           let t = t.get_mut(|a| a.clone())?;
           let starting_position = starting_position * Vec2::splat(1000.0) + t.translation.xy();
           for _ in 0..((random!(3..10) + random!(0..10)) / 3) {
               let enemy = enemy_types.get(random!(0..enemy_types.len()) as usize).unwrap().clone();
               AsyncWorld.send_event(SpawnEnemy {
                   position: starting_position + Vec2::new(random!(-50.0..50.0), random!(-50.0..50.0)),
                   enemy,
               })?;
           }
       } 
    });
}

#[derive(Event, Debug)]
pub struct SpawnEnemy {
    position: Vec2,
    enemy: Enemy,
}

fn handle_spawn_enemy(
    mut commands: Commands,
    mut spawn_enemy: EventReader<SpawnEnemy>,
    asset_server: Res<AssetServer>,
) {
    for spawn_enemy in spawn_enemy.read() {
        println!("spawning enemy at: {:#?}", spawn_enemy);
        commands
            .spawn((
                Sprite {
                    image: asset_server.load("images/slime.png"),
                    rect: Some(Rect::new(0.0, 16.0, 16.0 * 2.0, 16.0 * 2.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(spawn_enemy.position.x, spawn_enemy.position.y, 0.0)),
                spawn_enemy.enemy.clone(),
                MaxInternalVelocity::random(),
                Velocity(Vec3::new(0.0, 0.0, 0.0)),
                Pickable::default(),
            ))
            .observe(on_clickable_added)
            .observe(on_clickable_removed)
            .observe(on_click_enemy);
    }
}