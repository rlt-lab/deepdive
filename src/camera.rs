use bevy::prelude::*;

use crate::components::*;
use crate::constants::{TILE_SIZE, CAMERA_PADDING, ZOOM_SPEED, ZOOM_MIN, ZOOM_MAX};
use crate::map::GameMap;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        GameCamera,
        CameraFollow {
            target: Entity::PLACEHOLDER, // Will be set when player spawns
            lerp_speed: 2.0,
            zoom_level: 1.0,
            target_zoom: 1.0,
        },
    ));
}

pub fn setup_camera_follow(
    player_entity: Res<PlayerEntity>,
    mut camera_query: Query<&mut CameraFollow, With<GameCamera>>,
) {
    if let Ok(mut camera_follow) = camera_query.single_mut() {
        camera_follow.target = player_entity.0;
    }
}

pub fn camera_follow_system(
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &mut CameraFollow), (With<GameCamera>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<GameCamera>)>,
    map: Res<GameMap>,
) {
    if let (Ok((mut camera_transform, camera_follow)), Ok(player_transform)) = 
        (camera_query.single_mut(), player_query.single()) {
        
        if camera_follow.target != Entity::PLACEHOLDER {
            let target_pos = player_transform.translation;
            
            // Calculate map bounds in world coordinates
            let half_map_width = (map.width as f32 * TILE_SIZE) / 2.0;
            let half_map_height = (map.height as f32 * TILE_SIZE) / 2.0;
            let min_x = -half_map_width - CAMERA_PADDING;
            let max_x = half_map_width + CAMERA_PADDING;
            let min_y = -half_map_height - CAMERA_PADDING;
            let max_y = half_map_height + CAMERA_PADDING;
            
            // Apply constraints - only allow free camera movement for very small maps
            let constrained_x = if map.width <= 10 { 
                target_pos.x 
            } else { 
                target_pos.x.clamp(min_x, max_x) 
            };
            let constrained_y = if map.height <= 10 { 
                target_pos.y 
            } else { 
                target_pos.y.clamp(min_y, max_y) 
            };
            
            // Smooth camera interpolation
            camera_transform.translation = camera_transform.translation.lerp(
                Vec3::new(constrained_x, constrained_y, camera_transform.translation.z),
                camera_follow.lerp_speed * time.delta_secs()
            );
        }
    }
}

pub fn camera_zoom_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &mut CameraFollow), With<GameCamera>>,
) {
    if let Ok((mut camera_transform, mut camera_follow)) = camera_query.single_mut() {
        // Handle zoom input
        if keyboard_input.just_pressed(KeyCode::Equal) || keyboard_input.just_pressed(KeyCode::NumpadAdd) {
            camera_follow.target_zoom = (camera_follow.target_zoom * ZOOM_SPEED).min(ZOOM_MAX);
        }
        if keyboard_input.just_pressed(KeyCode::Minus) || keyboard_input.just_pressed(KeyCode::NumpadSubtract) {
            camera_follow.target_zoom = (camera_follow.target_zoom / ZOOM_SPEED).max(ZOOM_MIN);
        }
        
        // Reset zoom with F3 key (R conflicts with regenerate map)
        if keyboard_input.just_pressed(KeyCode::F3) {
            camera_follow.target_zoom = 1.0;
        }
        
        // Smooth zoom interpolation
        camera_follow.zoom_level = camera_follow.zoom_level + 
            (camera_follow.target_zoom - camera_follow.zoom_level) * 5.0 * time.delta_secs();
        
        // Apply zoom to camera scale
        camera_transform.scale = Vec3::splat(1.0 / camera_follow.zoom_level);
    }
}

pub fn camera_debug_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    camera_query: Query<(&Transform, &CameraFollow), With<GameCamera>>,
    player_query: Query<(&Transform, &Player), Without<GameCamera>>,
) {
    // Display camera debug info when F1 is pressed
    if keyboard_input.just_pressed(KeyCode::F1) {
        if let (Ok((camera_transform, camera_follow)), Ok((player_transform, player))) = 
            (camera_query.single(), player_query.single()) {
            
            println!("=== Camera Debug Info ===");
            println!("Player Grid Pos: ({}, {})", player.x, player.y);
            println!("Player World Pos: ({:.1}, {:.1}, {:.1})", 
                player_transform.translation.x, 
                player_transform.translation.y, 
                player_transform.translation.z);
            println!("Camera Pos: ({:.1}, {:.1}, {:.1})", 
                camera_transform.translation.x, 
                camera_transform.translation.y, 
                camera_transform.translation.z);
            println!("Camera Scale: {:.2}", camera_transform.scale.x);
            println!("Zoom Level: {:.2} -> {:.2}", 
                camera_follow.zoom_level, 
                camera_follow.target_zoom);
            println!("Camera Follow Speed: {:.1}", camera_follow.lerp_speed);
            println!("========================");
        }
    }
    
    // Display controls help when F2 is pressed
    if keyboard_input.just_pressed(KeyCode::F2) {
        println!("=== Camera Controls ===");
        println!("Movement: WASD or Arrow Keys");
        println!("Zoom In: + or NumPad +");
        println!("Zoom Out: - or NumPad -");
        println!("Reset Zoom: F3");
        println!("Debug Info: F1");
        println!("Controls Help: F2");
        println!("=======================");
    }
}
