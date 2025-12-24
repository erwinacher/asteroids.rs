use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Mesh, Mesh2d, Indices, PrimitiveTopology};
use bevy::math::prelude::Circle;
use bevy::sprite_render::MeshMaterial2d;
use bevy::window::WindowResolution;
use bevy::window::PrimaryWindow;
use bevy::window::{Window, WindowResized};
use std::sync::{Mutex, OnceLock};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

const VIEWPORT_WIDTH: usize = 1280;
const VIEWPORT_HEIGHT: usize = 720;
const VIEWPORT_MAX_X: f32 = VIEWPORT_WIDTH as f32 / 2.0;
const VIEWPORT_MIN_X: f32 = -VIEWPORT_MAX_X;
const VIEWPORT_MAX_Y: f32 = VIEWPORT_HEIGHT as f32 / 2.0;
const VIEWPORT_MIN_Y: f32 = -VIEWPORT_MAX_Y;
const ASTEROID_VELOCITY: f32 = 2.0;
const BULLET_VELOCITY: f32 = 6.0;
const BULLET_DISTANCE: f32 = VIEWPORT_HEIGHT as f32 * 0.8;
const STARSHIP_ROTATION_SPEED: f32 = 5.0 * 2.0 * PI / 360.0;
const STARSHIP_ACCELERATION: f32 = 0.2;
const STARSHIP_DECELERATION: f32 = 0.01;
const STARSHIP_MAX_VELOCITY: f32 = 10.0;

#[derive(Default, Clone, Copy)]
struct VirtualInput {
    left: bool,
    right: bool,
    up: bool,
    fire: bool, // one-shot
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn mobile_left_down() {
    if let Ok(mut v) = virtual_input().lock() {
        v.left = true;
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn mobile_left_up() {
    if let Ok(mut v) = virtual_input().lock() {
        v.left = false;
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn mobile_right_down() {
    if let Ok(mut v) = virtual_input().lock() {
        v.right = true;
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn mobile_right_up() {
    if let Ok(mut v) = virtual_input().lock() {
        v.right = false;
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn mobile_up_down() {
    if let Ok(mut v) = virtual_input().lock() {
        v.up = true;
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn mobile_up_up() {
    if let Ok(mut v) = virtual_input().lock() {
        v.up = false;
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn mobile_fire() {
    if let Ok(mut v) = virtual_input().lock() {
        v.fire = true; // handled as "just pressed" next frame
    }
}

#[derive(Resource, Clone, Copy, Debug)]
struct ScreenBounds {
    half_width: f32,
    half_height: f32,
}

impl Default for ScreenBounds {
    fn default() -> Self {
        Self {
            half_width: 640.0,  // fallback before first update
            half_height: 360.0,
        }
    }
}


fn update_screen_bounds(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut bounds: ResMut<ScreenBounds>,
) {
    if let Ok(window) = windows.single() {
        bounds.half_width = window.width() / 2.0;
        bounds.half_height = window.height() / 2.0;
    }
}

#[derive(Resource, Default, Clone, Copy)]
struct MobileInputState {
    left: bool,
    right: bool,
    up: bool,
    fire_just_pressed: bool,
}

//#[cfg(target_arch = "wasm32")]
fn sync_mobile_input(mut state: ResMut<MobileInputState>) {
    if let Ok(mut v) = virtual_input().lock() {
        state.left = v.left;
        state.right = v.right;
        state.up = v.up;
        state.fire_just_pressed = v.fire;
        // fire is one-shot
        v.fire = false;
    }
}


static VIRTUAL_INPUT: OnceLock<Mutex<VirtualInput>> = OnceLock::new();

fn virtual_input() -> &'static Mutex<VirtualInput> {
    VIRTUAL_INPUT.get_or_init(|| Mutex::new(VirtualInput::default()))
}


pub fn run() {
    let mut app = App::new();

    let mut window = Window {
                resolution: WindowResolution::new(1280, 720),
                title: "Asteroids".into(),
                ..default()
            };
            window.resolution.set_scale_factor(1.0);


    app.add_plugins(
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // Let Bevy resize the <canvas> to fill its parent on web
                fit_canvas_to_parent: true,
                // any starting value, it will be overridden
                resolution: WindowResolution::new(1280, 720),
                title: "Asteroids".into(),
                ..default()
            }),
            ..default()
        }),
    )
    .init_resource::<ScreenBounds>()
    .add_message::<ResetGame>()
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (
            update_screen_bounds,                  // <— keep this early
            keyboard_events,
            decelerate_starship,
            remove_bullet,
            update_position,
            sync_translate_transform.after(update_position),
            sync_asteroid_scale_transform,
            sync_starship_rotation_transform,
            detect_starship_asteroid_collision,
            detect_bullet_asteroid_collision,
            reset_game,
        ),
    );

    #[cfg(target_arch = "wasm32")]
    {
        app.init_resource::<MobileInputState>()
            .add_systems(Update, sync_mobile_input);
    }

    app.run();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_start() {
    run();
}

// pub fn run() {
//     // This should be exactly the same as your current `main` body
//     App::new()
//         .add_plugins(
//             DefaultPlugins.set(WindowPlugin {
//                 primary_window: Some(Window {
//                     // whatever resolution you set
//                     resolution: WindowResolution::new(900, 600),
//                     title: "Asteroids".into(),
//                     ..default()
//                 }),
//                 ..default()
//             }),
//         )
//         .add_message::<ResetGame>()
//         //#[cfg(target_arch = "wasm32")]
//         .init_resource::<MobileInputState>()
//         .add_systems(Startup, setup)
//         .add_systems(
//             Update,
//             (
//                 keyboard_events,
//                 decelerate_starship,
//                 remove_bullet,
//                 update_position,
//                 sync_translate_transform.after(update_position),
//                 sync_asteroid_scale_transform,
//                 sync_starship_rotation_transform,
//                 detect_starship_asteroid_collision,
//                 detect_bullet_asteroid_collision,
//                 reset_game,
//             ),
//         )
//         .add_systems(Update, sync_mobile_input)
//         .run();


// }



#[derive(Debug, Clone, Copy)]
enum AsteroidSize {
  Big,
  Medium,
  Small,
}

impl AsteroidSize {
  fn scale(&self) -> f32 {
    match self {
      AsteroidSize::Big => 100.0,
      AsteroidSize::Medium => 65.0,
      AsteroidSize::Small => 30.0,
    }
  }
}

#[derive(Message)]
struct ResetGame;

#[derive(Component)]
struct Starship {
  rotation_angle: f32,
}

impl Starship {
  fn direction(&self) -> Vec2 {
    let (y, x) = (self.rotation_angle + PI / 2.0).sin_cos();

    Vec2::new(x, y)
  }
}

#[derive(Component)]
struct Bullet {
  start: Vec2,
}

#[derive(Component)]
struct Asteroid {
  size: AsteroidSize,
}

#[derive(Component)]
struct Position(Vec2);

#[derive(Component)]
struct Velocity(Vec2);

fn create_starship_mesh() -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [0.0, 0.5, 0.0],
            [-0.25, -0.5, 0.0],
            [0.25, -0.5, 0.0],
        ],
    );

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![[0.0, 0.0, 1.0]; 3],
    );

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![
            [0.5, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
        ],
    );

    mesh.insert_indices(Indices::U32(vec![0, 1, 2]));

    mesh
}

fn get_random_point(bounds: &ScreenBounds) -> Vec2 {
    Vec2::new(
        (rand::random::<f32>() * 2.0 - 1.0) * bounds.half_width,
        (rand::random::<f32>() * 2.0 - 1.0) * bounds.half_height,
    )
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    bounds: Res<ScreenBounds>,
) {
    // Camera (Bevy 0.17)
    commands.spawn(Camera2d);

    // Starship
    commands.spawn((
        Starship {
            rotation_angle: 0.0,
        },
        Position(Vec2::ZERO),
        Velocity(Vec2::ZERO),
        Mesh2d(meshes.add(create_starship_mesh())),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgba(
            1.0, 0.0, 0.0, 1.0,
        )))),
        Transform::from_scale(Vec3::splat(50.0))
            .with_translation(Vec3::new(0.0, 0.0, 1.0)),
    ));

    // Asteroids
    for _ in 0..6 {
        commands.spawn((
            Asteroid {
                size: AsteroidSize::Big,
            },
            Position(get_random_point(&bounds)),
            Velocity(get_random_point(&bounds).normalize() * ASTEROID_VELOCITY),
            Mesh2d(meshes.add(Mesh::from(Circle::default()))),
            MeshMaterial2d(
                materials.add(ColorMaterial::from(Color::srgba(
                    0.8, 0.8, 0.8, 1.0,
                ))),
            ),
            Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
        ));
    }
}


fn sync_translate_transform(mut query: Query<(&Position, &mut Transform)>) {
  for (position, mut transform) in &mut query {
    transform.translation =
      Vec3::new(position.0.x, position.0.y, transform.translation.z);
  }
}

fn sync_asteroid_scale_transform(
  mut query: Query<(&Asteroid, &mut Transform)>,
) {
  for (asteroid, mut transform) in &mut query {
    transform.scale = Vec3::splat(asteroid.size.scale())
  }
}

fn sync_starship_rotation_transform(
  mut query: Query<(&Starship, &mut Transform)>,
) {
  for (starship, mut transform) in &mut query {
    transform.rotation = Quat::from_rotation_z(starship.rotation_angle);
  }
}

fn update_position(
    bounds: Res<ScreenBounds>,
    mut query: Query<(&Velocity, &Transform, &mut Position)>,
) {
    let min_x = -bounds.half_width;
    let max_x = bounds.half_width;
    let min_y = -bounds.half_height;
    let max_y = bounds.half_height;

    for (velocity, transform, mut position) in &mut query {
        let mut new_position = position.0 + velocity.0;
        let half_scale = transform.scale.max_element() / 2.0;

        if new_position.x > max_x + half_scale {
            new_position.x = min_x - half_scale;
        } else if new_position.x < min_x - half_scale {
            new_position.x = max_x + half_scale;
        }

        if new_position.y > max_y + half_scale {
            new_position.y = min_y - half_scale;
        } else if new_position.y < min_y - half_scale {
            new_position.y = max_y + half_scale;
        }

        position.0 = new_position;
    }
}

fn keyboard_events(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Starship, &Position, &mut Velocity)>,
    mobile: Option<Res<MobileInputState>>, // works on native & wasm
) {
    let mobile = mobile.as_deref();

    let left_pressed = keys.pressed(KeyCode::ArrowLeft)
        || mobile.map_or(false, |m| m.left);

    let right_pressed = keys.pressed(KeyCode::ArrowRight)
        || mobile.map_or(false, |m| m.right);

    let up_pressed = keys.pressed(KeyCode::ArrowUp)
        || mobile.map_or(false, |m| m.up);

    let fire_just_pressed = keys.just_pressed(KeyCode::Space)
        || mobile.map_or(false, |m| m.fire_just_pressed);

    for (mut starship, starship_position, mut velocity) in &mut query {
        // rotation
        if left_pressed {
            starship.rotation_angle += STARSHIP_ROTATION_SPEED;
        } else if right_pressed {
            starship.rotation_angle -= STARSHIP_ROTATION_SPEED;
        }

        // thrust
        if up_pressed {
            velocity.0 += starship.direction() * STARSHIP_ACCELERATION;

            if velocity.0.length() > STARSHIP_MAX_VELOCITY {
                velocity.0 = velocity.0.normalize_or_zero() * STARSHIP_MAX_VELOCITY;
            }
        }

        // fire bullet
        if fire_just_pressed {
            commands.spawn((
                Bullet {
                    start: starship_position.0,
                },
                Position(starship_position.0),
                Velocity(starship.direction().normalize() * BULLET_VELOCITY),
                Mesh2d(meshes.add(Mesh::from(Circle::default()))),
                MeshMaterial2d(
                    materials.add(ColorMaterial::from(Color::srgba(
                        1.0, 1.0, 1.0, 1.0,
                    ))),
                ),
                Transform::default()
                    .with_scale(Vec3::splat(5.0))
                    .with_translation(Vec3::splat(0.0)),
            ));
        }
    }
}


// fn keyboard_events(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
//     keys: Res<ButtonInput<KeyCode>>,
//     mut query: Query<(&mut Starship, &Position, &mut Velocity)>,
//     #[cfg(target_arch = "wasm32")] mobile: Option<Res<MobileInputState>>,
// ) {
//     for (mut starship, starship_position, mut velocity) in &mut query {
//         // rotation
//         if keys.pressed(KeyCode::ArrowLeft) {
//             starship.rotation_angle += STARSHIP_ROTATION_SPEED;
//         } else if keys.pressed(KeyCode::ArrowRight) {
//             starship.rotation_angle -= STARSHIP_ROTATION_SPEED;
//         }

//         // thrust
//         if keys.pressed(KeyCode::ArrowUp) {
//             velocity.0 += starship.direction() * STARSHIP_ACCELERATION;

//             if velocity.0.length() > STARSHIP_MAX_VELOCITY {
//                 velocity.0 = velocity.0.normalize_or_zero() * STARSHIP_MAX_VELOCITY;
//             }
//         }

//         // fire bullet
//         if keys.just_pressed(KeyCode::Space) {
//             // inside keyboard_events, where you spawn bullets:
//             commands.spawn((
//                 Bullet {
//                     start: starship_position.0,
//                 },
//                 Position(starship_position.0),
//                 Velocity(starship.direction().normalize() * BULLET_VELOCITY),
//                 Mesh2d(meshes.add(Mesh::from(Circle::default()))),
//                 MeshMaterial2d(
//                     materials.add(ColorMaterial::from(Color::srgba(1.0, 1.0, 1.0, 1.0))),
//                 ),
//                 Transform::default()
//                     .with_scale(Vec3::splat(5.0))
//                     .with_translation(Vec3::splat(0.0)),
//             ));
//         }
//     }
// }


fn remove_bullet(
  mut commands: Commands,
  query: Query<(Entity, &Bullet, &Position)>,
) {
  for (entity, bullet, position) in &query {
    if (bullet.start - position.0).length() > BULLET_DISTANCE {
      commands.entity(entity).despawn();
    }
  }
}

fn decelerate_starship(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Starship>>,
) {
    if !keys.pressed(KeyCode::ArrowUp) {
        for mut velocity in &mut query {
            velocity.0 *= 1.0 - STARSHIP_DECELERATION;
        }
    }
}

fn detect_starship_asteroid_collision(
    _commands: Commands,
    starship_query: Query<(Entity, &Transform, &Position), With<Starship>>,
    asteroids_query: Query<(&Transform, &Position), With<Asteroid>>,
    mut reset_writer: MessageWriter<ResetGame>,
) {
    for (_starship_entity, starship_transform, starship_position) in &starship_query {
        for (asteroid_transform, asteroid_position) in &asteroids_query {
            let starship_size = starship_transform.scale.max_element();
            let asteroid_size = asteroid_transform.scale.max_element();
            let distance = (starship_position.0 - asteroid_position.0).length();

            if distance < starship_size / 4.0 + asteroid_size / 2.0 {
                // Ship hit → trigger full reset
                reset_writer.write(ResetGame);
                return; // only need one hit
            }
        }
    }
}

fn detect_bullet_asteroid_collision(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    bullets_query: Query<(Entity, &Transform, &Position), With<Bullet>>,
    asteroids_query: Query<(Entity, &Asteroid, &Transform, &Position)>,
        bounds: Res<ScreenBounds>,

) {
    for (bullet_entity, bullet_transform, bullet_position) in &bullets_query {
        for (asteroid_entity, asteroid, asteroid_transform, asteroid_position) in &asteroids_query {
            let bullet_size = bullet_transform.scale.max_element();
            let asteroid_size = asteroid_transform.scale.max_element();
            let distance = (bullet_position.0 - asteroid_position.0).length();

            if distance < bullet_size / 2.0 + asteroid_size / 2.0 {
                // remove bullet + asteroid
                commands.entity(bullet_entity).despawn();
                commands.entity(asteroid_entity).despawn();

                let asteroid_new_size = match asteroid.size {
                    AsteroidSize::Big => Some(AsteroidSize::Medium),
                    AsteroidSize::Medium => Some(AsteroidSize::Small),
                    AsteroidSize::Small => None,
                };

                if let Some(asteroid_new_size) = asteroid_new_size {
                    for _ in 0..2 {
                        commands.spawn((
                            Asteroid {
                                size: asteroid_new_size,
                            },
                            Position(asteroid_position.0),
                            Velocity(get_random_point(&bounds).normalize() * ASTEROID_VELOCITY),
                            Mesh2d(meshes.add(Mesh::from(Circle::default()))),
                            MeshMaterial2d(
                                materials.add(ColorMaterial::from(Color::srgba(
                                    0.8, 0.8, 0.8, 1.0,
                                ))),
                            ),
                            Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
                        ));
                    }
                }
            }
        }
    }
}

fn reset_game(
    mut reset_events: MessageReader<ResetGame>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    bounds: Res<ScreenBounds>,
    to_clear: Query<Entity, Or<(With<Starship>, With<Bullet>, With<Asteroid>)>>,
) {
    // Read messages; if none, do nothing this frame
    if reset_events.read().next().is_none() {
        return;
    }

    // 1) Despawn all gameplay entities
    for entity in &to_clear {
        commands.entity(entity).despawn();
    }

    // 2) Spawn starship (same as in setup)
    commands.spawn((
        Starship { rotation_angle: 0.0 },
        Position(Vec2::ZERO),
        Velocity(Vec2::ZERO),
        Mesh2d(meshes.add(create_starship_mesh())),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgba(
            1.0, 0.0, 0.0, 1.0,
        )))),
        Transform::from_scale(Vec3::splat(50.0))
            .with_translation(Vec3::new(0.0, 0.0, 1.0)),
    ));

    // 3) Spawn asteroids (using current screen bounds)
    for _ in 0..6 {
        let pos = get_random_point(&bounds);
        let vel_dir = get_random_point(&bounds).normalize();

        commands.spawn((
            Asteroid {
                size: AsteroidSize::Big,
            },
            Position(pos),
            Velocity(vel_dir * ASTEROID_VELOCITY),
            Mesh2d(meshes.add(Mesh::from(Circle::default()))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgba(
                0.8, 0.8, 0.8, 1.0,
            )))),
            Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
        ));
    }
}

