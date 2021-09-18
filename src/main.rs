use bevy::{
    asset::AssetServer,
    core::FixedTimestep,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    sprite::collide_aabb::collide,
};

use rand::distributions::{Distribution, Standard};
use rand::{thread_rng, Rng};

const WINDOWHEIGHT: f32 = 900.0;
const WINDOWWIDTH: f32 = 1000.0;

// For BLOCK_SPAWN_TIMESTEP, it's once every two seconds
const BLOCK_SPAWN_TIMESTEP: f64 = 120.0 / 60.0;

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
        .add_startup_system(spawn_starting_block.system())
        .add_startup_system(render_score.system())
        .add_system_set(
            SystemSet::new()
                // This prints out "goodbye world" twice every second
                .with_run_criteria(FixedTimestep::step(BLOCK_SPAWN_TIMESTEP))
                .with_system(spawn_runtime_blocks.system()),
        )
        .add_system(move_player.system())
        .add_system(move_blocks.system())
        .add_system(player_collision_system.system())
        // Turn on to see framerate, also import line above
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}


fn add_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

struct Collidable;

// PLAYER CODE

// The float value is the player movement speed in 'pixels/second'.
struct Player {
    velocity: f32,
    teleport_distance: f32,
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
        .insert(Player {
            velocity: 300.0,
            teleport_distance: 70.0,
        })
        .insert(Collidable);
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<(&Player, &mut Transform, &Sprite)>,
    time: Res<Time>,
) {
    if let Ok((player, mut transform, sprite)) = player_query.single_mut() {
        // Get input from the keyboard (WASD)
        let up: bool = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
        let down: bool =
            keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);
        let left: bool =
            keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
        let right: bool =
            keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);

        // If left is pressed than it will be -1, right 1, both they cancel out.
        let x_axis: i8 = -(left as i8) + right as i8;
        let y_axis: i8 = -(down as i8) + up as i8;
        let move_delta: Vec2 = Vec2::new(x_axis as f32, y_axis as f32);

        // move the player
        let delta_time = time.delta_seconds();
        transform.translation.x += move_delta.x * player.velocity * delta_time;
        transform.translation.y += move_delta.y * player.velocity * delta_time;

        // Wrap the player if they go off screen
        if transform.translation.x > WINDOWWIDTH / 2.0 + sprite.size.x {
            transform.translation.x = -WINDOWWIDTH / 2.0;
        }

        if transform.translation.x < -WINDOWWIDTH / 2.0 - sprite.size.x {
            transform.translation.x = WINDOWWIDTH / 2.0;
        }

        if transform.translation.y > WINDOWHEIGHT / 2.0 + sprite.size.y {
            transform.translation.y = -WINDOWHEIGHT / 2.0;
        }

        if transform.translation.y < -WINDOWHEIGHT / 2.0 - sprite.size.y {
            transform.translation.y = WINDOWHEIGHT / 2.0;
        }

        // teleport the player if they press space
        if keyboard_input.just_pressed(KeyCode::Space) {
            if y_axis == -1 {
                transform.translation.y -= player.teleport_distance;
            }

            if y_axis == 1 {
                transform.translation.y += player.teleport_distance;
            }

            if x_axis == 1 {
                transform.translation.x += player.teleport_distance;
            }

            if x_axis == -1 {
                transform.translation.x -= player.teleport_distance;
            }
        }
    }
}

// simple, player collides with block system
fn player_collision_system(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Sprite, &Transform), With<Player>>,
    collider_query: Query<&Transform, (With<Collidable>, Without<Player>)>,
) {
    if let Ok((player_entity, sprite, player_transform)) = player_query.single_mut() {
        let player_size = sprite.size;

        for transform in collider_query.iter() {
            let collision = collide(
                player_transform.translation,
                player_size,
                transform.translation,
                sprite.size,
            );

            if let Some(_collision) = collision {
                // NOTE maybe add lives later
                // Remove the player if they collide with a block
                commands.entity(player_entity).despawn();
            }
        }
    }
}

// Block Code
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0..=3) {
            0 => Direction::Left,
            1 => Direction::Right,
            2 => Direction::Up,
            _ => Direction::Down,
        }
    }
}

struct Block {
    velocity: f32,
    direction: Direction,
}

fn spawn_starting_block(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let mut counter = 0;

    let block_number = 20;
    while counter < block_number {
        spawn_block(&mut commands, &mut materials, Color::rgb(1.0, 0.5, 1.0));
        counter += 1;
    }
}

// spawns blocks as a way to make the game harder during runtime
// this will only run every spawn block timestep
fn spawn_runtime_blocks(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    spawn_block(&mut commands, &mut materials, Color::rgb(0.2, 0.5, 1.0))
}

fn spawn_block(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    color: Color,
) {
    let mut rng = thread_rng();
    let rand_direction: Direction = rand::random();
    let x_starting_position = rng.gen_range(0.0..=WINDOWWIDTH);
    let y_starting_position = rng.gen_range(0.0..=WINDOWHEIGHT);
    let sprite_size_x = 80.0;
    let sprite_size_y = 80.0;

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(color.into()),
            transform: Transform::from_xyz(x_starting_position, y_starting_position, 1.0),
            sprite: Sprite::new(Vec2::new(sprite_size_x, sprite_size_y)),
            ..Default::default()
        })
        .insert(Block {
            velocity: 300.0,
            direction: rand_direction,
        })
        .insert(Collidable);
}

// move the block by its own velocity
fn move_blocks(mut block_query: Query<(&Block, &mut Transform, &Sprite)>, time: Res<Time>) {
    for (block, mut transform, sprite) in block_query.iter_mut() {
        let block_speed = block.velocity * time.delta_seconds();
        match &block.direction {
            Direction::Left => transform.translation.x -= block_speed,
            Direction::Right => transform.translation.x += block_speed,
            Direction::Up => transform.translation.y += block_speed,
            Direction::Down => transform.translation.y -= block_speed,
        };

        // Wrap the block if they go off screen
        if transform.translation.x > WINDOWWIDTH / 2.0 + sprite.size.x {
            transform.translation.x = -WINDOWWIDTH / 2.0;
        }

        if transform.translation.x < -WINDOWWIDTH / 2.0 - sprite.size.x {
            transform.translation.x = WINDOWWIDTH / 2.0;
        }

        if transform.translation.y > WINDOWHEIGHT / 2.0 + sprite.size.y {
            transform.translation.y = -WINDOWHEIGHT / 2.0;
        }

        if transform.translation.y < -WINDOWHEIGHT / 2.0 - sprite.size.y {
            transform.translation.y = WINDOWHEIGHT / 2.0;
        }
    }
}

// Score

fn render_score(mut commands: Commands, asset_server: Res<AssetServer>) {

    // UI camera
    // Need ui camera to see UI
    commands.spawn_bundle(UiCameraBundle::default());

    let text = Text::with_section(
        "hello world!".to_string(),
        TextStyle {
            // It does seem to find the asset
            font: asset_server.load("fonts/Roboto-Thin.ttf"),
            font_size: 60.0,
            color: Color::BLACK,
        },
        TextAlignment {
            vertical: VerticalAlign::Center,
            horizontal: HorizontalAlign::Center,
        },
    );

    // NOTE
    // I need to keep messing around with this until I get it in the top right corner
    let style = Style {
        // TODO what is this doing
        align_self: AlignSelf::FlexEnd,
        // TODO what is this doing
        position_type: PositionType::Absolute,
        position: Rect{
            top: Val::Px(60.0),
            right: Val::Px(80.0),

            // default is spawning in the lower left hand corner
            ..Default::default()
        },
        ..Default::default()
    };

    commands.spawn_bundle(TextBundle {
        style: style,
        text: text,
        ..Default::default()
    });
}

// TODO this function should be called every frame
// fn score_update_system() {

// }
