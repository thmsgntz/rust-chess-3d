mod pieces;
mod borad;

use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_mod_picking::*;

mod settings {
    use bevy::window::WindowMode;

    pub static NAME: &str = "Chess Game by Buer Games";
    pub const WINDOW_WIDTH: f32 = 1200.;
    pub const WINDOW_HEIGHT: f32 = 600.;
    pub const WINDOW_POSITION_X: f32 = 50.;
    pub const WINDOW_POSITION_Y: f32 = 25.;
    pub const WINDOW_MODE: WindowMode = WindowMode::Windowed;
}

fn setup (
    mut commands: Commands,
) {

    // Camera
    // https://github.com/bevyengine/bevy/blob/main/examples/3d/orthographic.rs
    // https://docs.rs/bevy/0.7.0/bevy/prelude/struct.OrthographicCameraBundle.html#
    let mut camera = OrthographicCameraBundle::new_3d();
    camera.orthographic_projection.scale = 5.0;
    /*camera.transform = Transform::from_xyz(1.0, 20.0, 0.0)
                        .looking_at(Vec3::new(0.0,0.0,0.0),
                                    Vec3::new(0.0,1.0,0.0));*/
    camera.transform = Transform::from_matrix(Mat4::from_rotation_translation(
        Quat::from_xyzw(-0.3, -0.5, -0.3, 0.5).normalize(),
        Vec3::new(-7.0, 20.0, 4.0)));

    commands
        .spawn_bundle(camera)
        .insert_bundle(PickingCameraBundle::default());

    // Light
    commands
        .spawn_bundle(PointLightBundle  {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        });
}



fn main() {
    App::new()
        // Set antialiasing to use 4 samples
        .insert_resource(Msaa { samples: 4 })
        // Set WindowDescriptor Resource to change title and size
        .insert_resource(WindowDescriptor {
            title: settings::NAME.parse().unwrap(),
            width: settings::WINDOW_WIDTH,
            height: settings::WINDOW_HEIGHT,
            position: Vec2::new(settings::WINDOW_POSITION_X, settings::WINDOW_POSITION_Y).into(),
            mode: settings::WINDOW_MODE,
            present_mode: PresentMode::Mailbox,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)// <- Adds Picking, Interaction, and Highlighting plugins.
        //.add_plugin(DebugCursorPickingPlugin) // <- Adds the green debug cursor.
        //.add_plugin(DebugEventsPickingPlugin) // <- Adds debug event logging.
        .add_plugin(borad::BoardPlugin)
        .add_startup_system(setup)
        .add_startup_system(pieces::create_pieces)
        .run();
}
