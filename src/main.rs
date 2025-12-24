

fn main() {
    asteroidslib::run();
}


// fn main() {
//     App::new()
//         .add_plugins(
//             DefaultPlugins.set(WindowPlugin {
//                 primary_window: Some(Window {
//                     resolution: WindowResolution::new(
//                         VIEWPORT_WIDTH as u32,
//                         VIEWPORT_HEIGHT as u32,
//                     ),
//                     title: "Asteroids".into(),
//                     ..default()
//                 }),
//                 ..default()
//             }),
//         )
//         .add_message::<ResetGame>()
//         .add_systems(Startup, setup)
//         .add_systems(
//             Update,
//             (
//                 keyboard_events,
//                 decelerate_starship,
//                 remove_bullet,
//                 update_position,
//                 // keep this ordering requirement:
//                 sync_translate_transform.after(update_position),
//                 sync_asteroid_scale_transform,
//                 sync_starship_rotation_transform,
//                 detect_starship_asteroid_collision,
//                 detect_bullet_asteroid_collision,
//                 reset_game,
//             ),
//         )
//         .run();
// }
