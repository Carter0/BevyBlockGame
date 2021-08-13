use bevy::prelude::*;

const WINDOWHEIGHT: f32 = 900.0;
const WINDOWWIDTH: f32 = 500.0;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Asteroids Clone".to_string(),
            width: WINDOWWIDTH,
            height: WINDOWHEIGHT,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(add_camera.system())
        .add_startup_system(spawn_player.system())
        .add_system(move_player.system())
        .run();
}

fn add_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

// PLAYER CODE

// The float value is the player movement speed in 'pixels/second'.
struct Player {
    velocity: f32,
}

fn spawn_player(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let sprite_size_x = 40.0;
    let sprite_size_y = 40.0;

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(0.5, 0.5, 1.0).into()),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            sprite: Sprite::new(Vec2::new(sprite_size_x, sprite_size_y)),
            ..Default::default()
        })
        .insert(Player { velocity: 10.0 });
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<(&Player, &mut Transform)>,
) {
    let (player, mut transform) = player_query
        .single_mut()
        .expect("There should always be exactly one player in the game!");

    // Get input from the keyboard (WASD)
    let up: bool = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
    let down: bool = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);
    let left: bool = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
    let right: bool = keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);

    // If left is pressed than it will be -1, right 1, both they cancel out.
    let x_axis: i8 = -(left as i8) + right as i8;
    let y_axis: i8 = -(down as i8) + up as i8;
    let move_delta: Vec2 = Vec2::new(x_axis as f32, y_axis as f32);

    transform.translation.x += move_delta.x * player.velocity;
    transform.translation.y += move_delta.y * player.velocity;
}
