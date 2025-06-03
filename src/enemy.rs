use bevy::prelude::*;
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_enemy_cluster);
    }
}

fn spawn_enemy_cluster(
    mut commands: Commands,
) {
    
}