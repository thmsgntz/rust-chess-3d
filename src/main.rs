use bevy::prelude::*;

mod settings {
    use bevy::window::WindowMode;

    pub static NAME: &str = "Chess Game by Bueur Games";
    pub const WINDOW_WIDTH: f32 = 1200.;
    pub const WINDOW_HEIGHT: f32 = 600.;
    pub const WINDOW_POSITION_X: f32 = 50.;
    pub const WINDOW_POSITION_Y: f32 = 25.;
    pub const WINDOW_MODE: WindowMode = WindowMode::Windowed;
}

fn create_pieces (
    mut commands:Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Load all the meshes
    let king_handle: Handle<Mesh> =
        asset_server.load("ressources/pieces.glb#Mesh0/Primitive0");


    let black_material = materials.add(Color::rgb(0., 0.2, 0.2).into());

    commands
        // Spawn parent entity
        .spawn_bundle(PbrBundle {
            mesh: king_handle.clone(),
            material: black_material.clone(),
            transform: {
                let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., -1.9));
                transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
                transform
            },
            ..Default::default()
        });
}

fn create_board (
    mut commands:Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
)
{
    // Add meshes and materials
    let mesh = meshes.add(Mesh::from(shape::Plane{size: 1.}));
    let white_material = materials.add(Color::rgb(1., 0.9, 0.9).into());
    let black_material = materials.add(Color::rgb(0., 0.1, 0.1).into());

    for i in 0..8  {
        for j in 0..8 {
            commands.spawn_bundle(PbrBundle{
                mesh: mesh.clone(),
                material: if (i + j + 1) % 2 == 0 {
                    white_material.clone()
                } else {
                    black_material.clone()
                },
                transform: Transform::from_translation(Vec3::new(i as f32, 0., j as f32)),
                ..Default::default()
            });
        }
    }
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

    commands.spawn_bundle(camera);

    // Light
    commands.spawn_bundle(PointLightBundle  {
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
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(create_board)
        .add_system(create_pieces)
        .run();
}
