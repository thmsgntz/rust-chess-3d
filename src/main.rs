#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod board;
mod pieces;
mod ui;

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::{PresentMode, WindowResolution};
use bevy_mod_picking::*;
use bevy_tweening::TweeningPlugin;

mod settings {
    use bevy::window::{MonitorSelection, WindowMode, WindowPosition};

    pub static NAME: &str = "Chess Game in RustxBevy!";
    pub const WINDOW_WIDTH: f32 = 1200.;
    pub const WINDOW_HEIGHT: f32 = 600.;
    pub const WINDOW_POSITION: WindowPosition = WindowPosition::Centered(MonitorSelection::Current);
    pub const WINDOW_MODE: WindowMode = WindowMode::Windowed;
}

fn setup(mut commands: Commands) {
    // Camera
    // https://github.com/bevyengine/bevy/blob/main/examples/3d/orthographic.rs
    // https://docs.rs/bevy/0.7.0/bevy/prelude/struct.OrthographicCameraBundle.html#
    let mut camera = Camera3dBundle {
        projection: OrthographicProjection {
            scale: 2.0,
            scaling_mode: ScalingMode::FixedVertical(5.0),
            ..default()
        }
        .into(),
        ..default()
    };

    /*camera.transform = Transform::from_xyz(1.0, 20.0, 0.0)
    .looking_at(Vec3::new(0.0,0.0,0.0),
                Vec3::new(0.0,1.0,0.0));*/
    camera.transform = Transform::from_matrix(Mat4::from_rotation_translation(
        Quat::from_xyzw(-0.3, -0.5, -0.3, 0.5).normalize(),
        Vec3::new(-7.0, 20.0, 4.0),
    ));

    commands
        .spawn(camera)
        .insert(PickingCameraBundle::default());

    // Light
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        ..Default::default()
    });
}

fn main() {
    App::new()
        // Set antialiasing to use 4 samples
        .insert_resource(ClearColor(Color::hex("A0A0A0").unwrap()))
        .insert_resource(Msaa::Sample4)
        // Set WindowDescriptor Resource to change title and size
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: settings::NAME.parse().unwrap(),
                resolution: WindowResolution::new(settings::WINDOW_WIDTH, settings::WINDOW_HEIGHT),
                position: settings::WINDOW_POSITION,
                mode: settings::WINDOW_MODE,
                present_mode: PresentMode::Fifo,
                ..Default::default()
            }),
            ..default()
        }))
        .add_plugins(DefaultPickingPlugins) // <- Adds Picking, Interaction, and Highlighting plugins.
        .add_plugin(TweeningPlugin)
        //.add_plugin(DebugCursorPickingPlugin) // <- Adds the green debug cursor.
        //.add_plugin(DebugEventsPickingPlugin) // <- Adds debug event logging.
        // .add_plugin(ui::UIPlugin)
        .add_plugin(board::BoardPlugin)
        .add_plugin(pieces::PiecesPlugin)
        .add_plugin(ui::UIPlugin)
        .add_startup_system(setup)
        .run();
}
