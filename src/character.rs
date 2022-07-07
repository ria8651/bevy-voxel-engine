use bevy::{input::mouse::MouseMotion, prelude::*};

const SPEED: f32 = 2.0;
const SENSITIVITY: f32 = 0.004;

#[derive(Component)]
struct CharacterEntity {
    velocity: Vec3,
    rotation: Vec2,
}

impl Default for CharacterEntity {
    fn default() -> Self {
        Self {
            velocity: Vec3::new(0.0, 0.0, 0.0),
            rotation: Vec2::new(0.0, 0.0),
        }
    }
}

pub struct Character;

impl Plugin for Character {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_character)
            .add_system(update_character);
    }
}

fn setup_character(mut commands: Commands, mut windows: ResMut<Windows>) {
    toggle_grab_cursor(windows.get_primary_mut().unwrap());

    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 1.0, -1.0).looking_at(Vec3::ZERO, Vec3::Y),
            perspective_projection: PerspectiveProjection {
                fov: 1.48353,
                near: 0.05,
                far: 10000.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle((CharacterEntity::default(), super::MainCamera));

    // commands.spawn_bundle((
    //     Transform::from_xyz(0.0, 1.0, -1.0).looking_at(Vec3::ZERO, Vec3::Y),
    //     CharacterEntity::default(),
    // ));
}

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    window.set_cursor_lock_mode(!window.cursor_locked());
    window.set_cursor_visibility(!window.cursor_visible());
}

fn update_character(
    mut character: Query<(&mut Transform, &mut CharacterEntity)>,
    keys: Res<Input<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    time: Res<Time>,
    mut windows: ResMut<Windows>,
) {
    let window = windows.get_primary_mut().unwrap();
    if keys.just_pressed(KeyCode::Escape) {
        toggle_grab_cursor(window);
    }

    if window.cursor_locked() {
        let (mut transform, mut character) = character.single_mut();

        // movement
        let mut input = Vec3::new(
            (keys.pressed(KeyCode::D) as i32 - keys.pressed(KeyCode::A) as i32) as f32,
            (keys.pressed(KeyCode::Space) as i32 - keys.pressed(KeyCode::LShift) as i32) as f32,
            (keys.pressed(KeyCode::S) as i32 - keys.pressed(KeyCode::W) as i32) as f32,
        );
        if input != Vec3::ZERO {
            input = input.normalize();
        }
        input *= SPEED;

        let target_velocity = input.z * transform.local_z()
            + input.x * transform.local_x()
            + input.y * transform.local_y();
        let delta_time = time.delta_seconds();
        character.velocity = character.velocity
            + (target_velocity - character.velocity) * (1.0 - 0.9f32.powf(delta_time * 120.0));
        transform.translation += character.velocity * delta_time;

        // rotation
        let mut mouse_delta = Vec2::new(0.0, 0.0);
        for event in mouse_motion_events.iter() {
            mouse_delta += event.delta;
        }
        if mouse_delta != Vec2::ZERO {
            let sensitivity = SENSITIVITY;
            character.rotation -= mouse_delta * sensitivity;
            character.rotation.y = character.rotation.y.clamp(-1.54, 1.54);

            // Order is important to prevent unintended roll
            transform.rotation = Quat::from_axis_angle(Vec3::Y, character.rotation.x)
                * Quat::from_axis_angle(Vec3::X, character.rotation.y);
        }
    }
}
