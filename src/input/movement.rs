//! Player movement input handling.

use bevy::prelude::*;

use crate::components::{Player, MovementInput, MovementAnimation, Autoexplore, TileType};
use crate::constants::HOP_ANIM_TIMER;
use crate::map::GameMap;

use super::KeyBindings;

// ============================================================================
// MOVEMENT EVENTS
// ============================================================================

#[derive(Event)]
pub struct PlayerMoveIntent {
    pub direction: MoveDirection,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

// ============================================================================
// MOVEMENT INPUT SYSTEMS
// ============================================================================

/// Event-based input detection - only fires when key state changes
pub fn detect_movement_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    time: Res<Time>,
    mut player_query: Query<&mut MovementInput>,
    mut move_events: EventWriter<PlayerMoveIntent>,
) {
    if let Ok(mut movement_input) = player_query.single_mut() {
        // Check for any movement key being pressed (arrow keys only, no WASD)
        let up_pressed = key_bindings.is_pressed(&key_bindings.move_up, &keyboard_input);
        let down_pressed = key_bindings.is_pressed(&key_bindings.move_down, &keyboard_input);
        let left_pressed = key_bindings.is_pressed(&key_bindings.move_left, &keyboard_input);
        let right_pressed = key_bindings.is_pressed(&key_bindings.move_right, &keyboard_input);

        let any_movement_key = up_pressed || down_pressed || left_pressed || right_pressed;

        // Handle initial key press or continuous movement
        let should_move = if any_movement_key {
            if !movement_input.is_holding {
                // First press - fire event immediately
                movement_input.is_holding = true;
                movement_input.move_timer.reset();
                true
            } else {
                // Continuous movement - check timer
                movement_input.move_timer.tick(time.delta());
                if movement_input.move_timer.finished() {
                    movement_input.move_timer.reset();
                    true
                } else {
                    false
                }
            }
        } else {
            movement_input.is_holding = false;
            false
        };

        // Fire movement intent events based on key priority
        if should_move {
            // Vertical movement (prioritized)
            if up_pressed {
                move_events.write(PlayerMoveIntent { direction: MoveDirection::Up });
            } else if down_pressed {
                move_events.write(PlayerMoveIntent { direction: MoveDirection::Down });
            }

            // Horizontal movement
            if left_pressed {
                move_events.write(PlayerMoveIntent { direction: MoveDirection::Left });
            } else if right_pressed {
                move_events.write(PlayerMoveIntent { direction: MoveDirection::Right });
            }
        }
    }
}

/// Process movement intent events
pub fn handle_movement_input(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Player, &mut Sprite, Option<&Autoexplore>), Without<MovementAnimation>>,
    mut move_events: EventReader<PlayerMoveIntent>,
    map: Res<GameMap>,
) {
    if let Ok((entity, mut player, mut sprite, autoexplore_opt)) = player_query.single_mut() {
        // Cancel autoexplore if player manually moves
        if move_events.len() > 0 && autoexplore_opt.is_some() {
            commands.entity(entity).remove::<Autoexplore>();
            println!("Autoexplore cancelled by manual input");
        }

        // Process all movement events for this frame
        for event in move_events.read() {
            let mut movement_attempted = false;
            let mut new_x = player.x;
            let mut new_y = player.y;
            let mut flip_sprite_opt: Option<bool> = None;

            // Apply movement based on direction
            match event.direction {
                MoveDirection::Up => {
                    if new_y < map.height - 1 {
                        new_y += 1;
                        movement_attempted = true;
                    }
                }
                MoveDirection::Down => {
                    if new_y > 0 {
                        new_y -= 1;
                        movement_attempted = true;
                    }
                }
                MoveDirection::Left => {
                    if new_x > 0 {
                        new_x -= 1;
                        movement_attempted = true;
                        flip_sprite_opt = Some(false); // No flip for left (natural direction)
                    }
                }
                MoveDirection::Right => {
                    if new_x < map.width - 1 {
                        new_x += 1;
                        movement_attempted = true;
                        flip_sprite_opt = Some(true); // Flip for right (face right)
                    }
                }
            }

            // Check collision with walls and apply movement
            if movement_attempted && map.get(new_x, new_y) != TileType::Wall {
                // Calculate start and end positions for animation
                let start_world = map.grid_to_world(player.x, player.y);
                let end_world = map.grid_to_world(new_x, new_y);

                // Update player grid position
                player.x = new_x;
                player.y = new_y;

                // Handle sprite flipping
                if let Some(flip) = flip_sprite_opt {
                    sprite.flip_x = flip;
                }

                // Add movement animation component
                commands.entity(entity).insert(MovementAnimation {
                    start_pos: Vec3::new(start_world.x, start_world.y, 1.0),
                    end_pos: Vec3::new(end_world.x, end_world.y, 1.0),
                    timer: Timer::from_seconds(HOP_ANIM_TIMER, TimerMode::Once),
                });

                println!("Player moved to ({}, {})", new_x, new_y);
            } else if movement_attempted {
                println!("Cannot move to ({}, {}) - wall detected", new_x, new_y);
            }
        }
    }
}
