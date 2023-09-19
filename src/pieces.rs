use bevy::prelude::*;
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween};
use std::time::Duration;

pub struct PiecesPlugin;
impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_pieces)
            .add_system(move_pieces.in_base_set(CoreSet::Last));
    }
}

mod piece_settings {
    pub static GLB_PIECES_PATH: &str = "resources/pieces.glb";
}

#[derive(Clone, Copy, PartialEq)]
pub enum PieceColor {
    White,
    Black,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PieceType {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn,
}

#[derive(Clone, Copy, Component)]
pub struct Piece {
    pub color: PieceColor,
    pub piece_type: PieceType,
    pub x: u8,
    pub y: u8,
}

impl Piece {
    // Returns the possible_positions that are available
    pub fn is_move_valid(&self, new_position: (u8, u8), pieces: Vec<Piece>) -> bool {
        // If there's a piece of the same color in the same square, it can't move
        if color_of_piece(new_position, &pieces) == Some(self.color) {
            return false;
        }

        match self.piece_type {
            PieceType::King => {
                // Horizontal
                ((self.x as i8 - new_position.0 as i8).abs() == 1
                    && (self.y == new_position.1))
                    // Vertical
                    || ((self.y as i8 - new_position.1 as i8).abs() == 1
                    && (self.x == new_position.0))
                    // Diagonal
                    || ((self.x as i8 - new_position.0 as i8).abs() == 1
                    && (self.y as i8 - new_position.1 as i8).abs() == 1)
            }
            PieceType::Queen => {
                is_path_empty((self.x, self.y), new_position, &pieces)
                    && ((self.x as i8 - new_position.0 as i8).abs()
                        == (self.y as i8 - new_position.1 as i8).abs()
                        || ((self.x == new_position.0 && self.y != new_position.1)
                            || (self.y == new_position.1 && self.x != new_position.0)))
            }
            PieceType::Bishop => {
                is_path_empty((self.x, self.y), new_position, &pieces)
                    && (self.x as i8 - new_position.0 as i8).abs()
                        == (self.y as i8 - new_position.1 as i8).abs()
            }
            PieceType::Knight => {
                ((self.x as i8 - new_position.0 as i8).abs() == 2
                    && (self.y as i8 - new_position.1 as i8).abs() == 1)
                    || ((self.x as i8 - new_position.0 as i8).abs() == 1
                        && (self.y as i8 - new_position.1 as i8).abs() == 2)
            }
            PieceType::Rook => {
                is_path_empty((self.x, self.y), new_position, &pieces)
                    && ((self.x == new_position.0 && self.y != new_position.1)
                        || (self.y == new_position.1 && self.x != new_position.0))
            }
            PieceType::Pawn => {
                if self.color == PieceColor::White {
                    // Normal move
                    if new_position.0 as i8 - self.x as i8 == 1
                        && (self.y == new_position.1)
                        && color_of_piece(new_position, &pieces).is_none()
                    {
                        return true;
                    }

                    // Move 2 squares
                    if self.x == 1
                        && new_position.0 as i8 - self.x as i8 == 2
                        && (self.y == new_position.1)
                        && is_path_empty((self.x, self.y), new_position, &pieces)
                        && color_of_piece(new_position, &pieces).is_none()
                    {
                        return true;
                    }

                    // Take piece
                    if new_position.0 as i8 - self.x as i8 == 1
                        && (self.y as i8 - new_position.1 as i8).abs() == 1
                        && color_of_piece(new_position, &pieces) == Some(PieceColor::Black)
                    {
                        return true;
                    }
                } else {
                    // Normal move
                    if new_position.0 as i8 - self.x as i8 == -1
                        && (self.y == new_position.1)
                        && color_of_piece(new_position, &pieces).is_none()
                    {
                        return true;
                    }

                    // Move 2 squares
                    if self.x == 6
                        && new_position.0 as i8 - self.x as i8 == -2
                        && (self.y == new_position.1)
                        && is_path_empty((self.x, self.y), new_position, &pieces)
                        && color_of_piece(new_position, &pieces).is_none()
                    {
                        return true;
                    }

                    // Take piece
                    if new_position.0 as i8 - self.x as i8 == -1
                        && (self.y as i8 - new_position.1 as i8).abs() == 1
                        && color_of_piece(new_position, &pieces) == Some(PieceColor::White)
                    {
                        return true;
                    }
                }

                false
            }
        }
    }
}

fn move_pieces(mut commands: Commands, mut query: Query<(Entity, &mut Transform, &Piece)>) {
    for (entity, transform, piece) in query.iter_mut() {
        let target_position = Vec3::new(piece.x as f32, 0., piece.y as f32);

        let tween = Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_secs_f32(0.2),
            TransformPositionLens {
                start: transform.translation,
                end: target_position,
            },
        );

        commands
            .entity(entity)
            .insert(Animator::<Transform>::new(tween));
    }
}

// fn move_pieces(time: Res<Time>, mut query: Query<(&mut Transform, &Piece)>) {
//     for (mut transform, piece) in query.iter_mut() {
//         // Get the direction to move in
//         let direction = Vec3::new(piece.x as f32, 0., piece.y as f32) - transform.translation;

//         // Only move if the piece isn't already there (distance is big)
//         if direction.length() > 0.1 {
//             info!("Moving piece: {:?}", piece.piece_type);
//             info!("Piece location: {:?}", transform.translation);
//             transform.translation += direction.normalize() * time.delta_seconds();
//         }
//     }
// }

/// Returns None if square is empty, returns a Some with the color if not
fn color_of_piece(pos: (u8, u8), pieces: &[Piece]) -> Option<PieceColor> {
    pieces
        .iter()
        .find(|p| p.x == pos.0 && p.y == pos.1)
        .map(|p| p.color)
}

fn is_path_empty(begin: (u8, u8), end: (u8, u8), pieces: &[Piece]) -> bool {
    if begin.0 == end.0 {
        is_column_empty(begin, end, pieces)
    } else if begin.1 == end.1 {
        is_row_empty(begin, end, pieces)
    } else {
        is_diagonal_empty(begin, end, pieces)
    }
}

fn is_column_empty(begin: (u8, u8), end: (u8, u8), pieces: &[Piece]) -> bool {
    !pieces.iter().any(|p| {
        p.x == begin.0 && ((p.y > begin.1 && p.y < end.1) || (p.y > end.1 && p.y < begin.1))
    })
}

fn is_row_empty(begin: (u8, u8), end: (u8, u8), pieces: &[Piece]) -> bool {
    !pieces.iter().any(|p| {
        p.y == begin.1 && ((p.x > begin.0 && p.x < end.0) || (p.x > end.0 && p.x < begin.0))
    })
}

fn is_diagonal_empty(begin: (u8, u8), end: (u8, u8), pieces: &[Piece]) -> bool {
    let (x_diff, y_diff) = (
        (begin.0 as i8 - end.0 as i8).abs(),
        (begin.1 as i8 - end.1 as i8).abs(),
    );

    if x_diff != y_diff {
        return false;
    }

    (1..x_diff)
        .map(|i| {
            if begin.0 < end.0 && begin.1 < end.1 {
                // left bottom - right top
                (begin.0 + i as u8, begin.1 + i as u8)
            } else if begin.0 < end.0 && begin.1 > end.1 {
                // left top - right bottom
                (begin.0 + i as u8, begin.1 - i as u8)
            } else if begin.0 > end.0 && begin.1 < end.1 {
                // right bottom - left top
                (begin.0 - i as u8, begin.1 + i as u8)
            } else {
                // begin.0 > end.0 && begin.1 > end.1
                // right top - left bottom
                (begin.0 - i as u8, begin.1 - i as u8)
            }
        })
        .all(|pos| color_of_piece(pos, pieces).is_none())
}

fn create_pieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Load all the meshes
    let king_handle: Handle<Mesh> =
        asset_server.load(format!("{}#Mesh0/Primitive0", piece_settings::GLB_PIECES_PATH).as_str());
    let king_cross_handle: Handle<Mesh> =
        asset_server.load(format!("{}#Mesh1/Primitive0", piece_settings::GLB_PIECES_PATH).as_str());
    let pawn_handle: Handle<Mesh> =
        asset_server.load(format!("{}#Mesh2/Primitive0", piece_settings::GLB_PIECES_PATH).as_str());
    let knight_1_handle: Handle<Mesh> =
        asset_server.load(format!("{}#Mesh3/Primitive0", piece_settings::GLB_PIECES_PATH).as_str());
    let knight_2_handle: Handle<Mesh> =
        asset_server.load(format!("{}#Mesh4/Primitive0", piece_settings::GLB_PIECES_PATH).as_str());
    let rook_handle: Handle<Mesh> =
        asset_server.load(format!("{}#Mesh5/Primitive0", piece_settings::GLB_PIECES_PATH).as_str());
    let bishop_handle: Handle<Mesh> =
        asset_server.load(format!("{}#Mesh6/Primitive0", piece_settings::GLB_PIECES_PATH).as_str());
    let queen_handle: Handle<Mesh> =
        asset_server.load(format!("{}#Mesh7/Primitive0", piece_settings::GLB_PIECES_PATH).as_str());

    let black_material = materials.add(Color::rgb(0., 0.2, 0.2).into());
    let white_material = materials.add(Color::rgb(1., 0.8, 0.8).into());

    spawn_rook(
        &mut commands,
        white_material.clone(),
        PieceColor::White,
        rook_handle.clone(),
        (0, 0),
    );
    spawn_knight(
        &mut commands,
        white_material.clone(),
        PieceColor::White,
        knight_1_handle.clone(),
        knight_2_handle.clone(),
        (0, 1),
    );
    spawn_bishop(
        &mut commands,
        white_material.clone(),
        PieceColor::White,
        bishop_handle.clone(),
        (0, 2),
    );
    spawn_queen(
        &mut commands,
        white_material.clone(),
        PieceColor::White,
        queen_handle.clone(),
        (0, 3),
    );
    spawn_king(
        &mut commands,
        white_material.clone(),
        PieceColor::White,
        king_handle.clone(),
        king_cross_handle.clone(),
        (0, 4),
    );
    spawn_bishop(
        &mut commands,
        white_material.clone(),
        PieceColor::White,
        bishop_handle.clone(),
        (0, 5),
    );
    spawn_knight(
        &mut commands,
        white_material.clone(),
        PieceColor::White,
        knight_1_handle.clone(),
        knight_2_handle.clone(),
        (0, 6),
    );
    spawn_rook(
        &mut commands,
        white_material.clone(),
        PieceColor::White,
        rook_handle.clone(),
        (0, 7),
    );

    for i in 0..8 {
        spawn_pawn(
            &mut commands,
            white_material.clone(),
            PieceColor::White,
            pawn_handle.clone(),
            (1, i as u8),
        );
    }

    spawn_rook(
        &mut commands,
        black_material.clone(),
        PieceColor::Black,
        rook_handle.clone(),
        (7, 0),
    );
    spawn_knight(
        &mut commands,
        black_material.clone(),
        PieceColor::Black,
        knight_1_handle.clone(),
        knight_2_handle.clone(),
        (7, 1),
    );
    spawn_bishop(
        &mut commands,
        black_material.clone(),
        PieceColor::Black,
        bishop_handle.clone(),
        (7, 2),
    );
    spawn_queen(
        &mut commands,
        black_material.clone(),
        PieceColor::Black,
        queen_handle,
        (7, 3),
    );
    spawn_king(
        &mut commands,
        black_material.clone(),
        PieceColor::Black,
        king_handle,
        king_cross_handle,
        (7, 4),
    );
    spawn_bishop(
        &mut commands,
        black_material.clone(),
        PieceColor::Black,
        bishop_handle,
        (7, 5),
    );
    spawn_knight(
        &mut commands,
        black_material.clone(),
        PieceColor::Black,
        knight_1_handle,
        knight_2_handle,
        (7, 6),
    );
    spawn_rook(
        &mut commands,
        black_material.clone(),
        PieceColor::Black,
        rook_handle,
        (7, 7),
    );

    for i in 0..8 {
        spawn_pawn(
            &mut commands,
            black_material.clone(),
            PieceColor::Black,
            pawn_handle.clone(),
            (6, i as u8),
        );
    }
}

pub fn spawn_king(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    piece_color: PieceColor,
    mesh: Handle<Mesh>,
    mesh_cross: Handle<Mesh>,
    position: (u8, u8),
) {
    commands
        // Spawn parent entity
        .spawn(PbrBundle {
            transform: Transform::from_translation(Vec3::new(
                position.0 as f32,
                0.,
                position.1 as f32,
            )),
            ..default()
        })
        .insert(Piece {
            color: piece_color,
            piece_type: PieceType::King,
            x: position.0,
            y: position.1,
        })
        // Add children to the parent
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh,
                material: material.clone(),
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., -1.9));
                    transform.scale *= Vec3::new(0.2, 0.2, 0.2);
                    transform
                },
                ..default()
            });
            parent.spawn(PbrBundle {
                mesh: mesh_cross,
                material,
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., -1.9));
                    transform.scale *= Vec3::new(0.2, 0.2, 0.2);
                    transform
                },
                ..default()
            });
        });
}

pub fn spawn_knight(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    piece_color: PieceColor,
    mesh_1: Handle<Mesh>,
    mesh_2: Handle<Mesh>,
    position: (u8, u8),
) {
    commands
        // Spawn parent entity
        .spawn(PbrBundle {
            transform: Transform::from_translation(Vec3::new(
                position.0 as f32,
                0.,
                position.1 as f32,
            )),
            ..default()
        })
        .insert(Piece {
            color: piece_color,
            piece_type: PieceType::Knight,
            x: position.0,
            y: position.1,
        })
        // Add children to the parent
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: mesh_1,
                material: material.clone(),
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., 0.9));
                    transform.scale *= Vec3::new(0.2, 0.2, 0.2);
                    transform
                },
                ..default()
            });
            parent.spawn(PbrBundle {
                mesh: mesh_2,
                material,
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., 0.9));
                    transform.scale *= Vec3::new(0.2, 0.2, 0.2);
                    transform
                },
                ..default()
            });
        });
}

pub fn spawn_queen(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    piece_color: PieceColor,
    mesh: Handle<Mesh>,
    position: (u8, u8),
) {
    commands
        .spawn(PbrBundle {
            transform: Transform::from_translation(Vec3::new(
                position.0 as f32,
                0.,
                position.1 as f32,
            )),
            ..default()
        })
        .insert(Piece {
            color: piece_color,
            piece_type: PieceType::Queen,
            x: position.0,
            y: position.1,
        })
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh,
                material,
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., -0.95));
                    transform.scale *= Vec3::new(0.2, 0.2, 0.2);
                    transform
                },
                ..default()
            });
        });
}

pub fn spawn_bishop(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    piece_color: PieceColor,
    mesh: Handle<Mesh>,
    position: (u8, u8),
) {
    commands
        .spawn(PbrBundle {
            transform: Transform::from_translation(Vec3::new(
                position.0 as f32,
                0.,
                position.1 as f32,
            )),
            ..default()
        })
        .insert(Piece {
            color: piece_color,
            piece_type: PieceType::Bishop,
            x: position.0,
            y: position.1,
        })
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh,
                material,
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.1, 0., 0.));
                    transform.scale *= Vec3::new(0.2, 0.2, 0.2);
                    transform
                },
                ..default()
            });
        });
}

pub fn spawn_rook(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    piece_color: PieceColor,
    mesh: Handle<Mesh>,
    position: (u8, u8),
) {
    commands
        .spawn(PbrBundle {
            transform: Transform::from_translation(Vec3::new(
                position.0 as f32,
                0.,
                position.1 as f32,
            )),
            ..default()
        })
        .insert(Piece {
            color: piece_color,
            piece_type: PieceType::Rook,
            x: position.0,
            y: position.1,
        })
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh,
                material,
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.1, 0., 1.8));
                    transform.scale *= Vec3::new(0.2, 0.2, 0.2);
                    transform
                },
                ..default()
            });
        });
}

pub fn spawn_pawn(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    piece_color: PieceColor,
    mesh: Handle<Mesh>,
    position: (u8, u8),
) {
    commands
        .spawn(PbrBundle {
            transform: Transform::from_translation(Vec3::new(
                position.0 as f32,
                0.,
                position.1 as f32,
            )),
            ..default()
        })
        .insert(Piece {
            color: piece_color,
            piece_type: PieceType::Pawn,
            x: position.0,
            y: position.1,
        })
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh,
                material,
                transform: {
                    let mut transform = Transform::from_translation(Vec3::new(-0.2, 0., 2.6));
                    transform.scale *= Vec3::new(0.2, 0.2, 0.2);
                    transform
                },
                ..default()
            });
        });
}
