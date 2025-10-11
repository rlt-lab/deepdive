use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::components::{CurrentLevel, DepthIndicator};
use crate::states::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing), 
            spawn_depth_indicator
        )
        .add_systems(
            Update, 
            update_depth_indicator.run_if(in_state(GameState::Playing))
        );
    }
}

pub fn spawn_depth_indicator(
    mut commands: Commands,
    assets: Res<GameAssets>,
) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("Depth 0"),
                TextFont {
                    font: assets.akkurat_font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                DepthIndicator,
            ));
        });
}

pub fn update_depth_indicator(
    current_level: Res<CurrentLevel>,
    mut text_query: Query<&mut Text, With<DepthIndicator>>,
) {
    if current_level.is_changed() {
        for mut text in text_query.iter_mut() {
            text.0 = format!("Depth {}", current_level.level);
        }
    }
}
