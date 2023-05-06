use bevy::app::AppExit;

use bevy::prelude::*;
use bevy_mod_picking::*;

use crate::pieces::*;

#[derive(Resource)]
pub struct PlayerTurn(PieceColor);

#[derive(Component)]
struct Taken;

impl PlayerTurn {
    pub fn change_turn(&mut self) {
        self.0 = match self.0 {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }

    pub fn get_current_turn(&self) -> String {
        match self.0 {
            PieceColor::White => "White".to_string(),
            PieceColor::Black => "Black".to_string(),
        }
    }
}

impl Default for PlayerTurn {
    fn default() -> Self {
        Self(PieceColor::White)
    }
}

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SquareMaterials>()
            .init_resource::<SelectedSquare>()
            .init_resource::<SelectedPiece>()
            .init_resource::<PlayerTurn>()
            .add_startup_system(create_board)
            .add_system(
                select_piece
                    .in_base_set(CoreSet::PostUpdate)
                    .before(move_piece),
            )
            .add_system(
                select_square
                    .in_base_set(CoreSet::PostUpdate)
                    .before(select_piece),
            )
            .add_system(move_piece.in_base_set(CoreSet::PostUpdate))
            .add_system(
                despawn_taken_pieces
                    .in_base_set(CoreSet::PostUpdate)
                    .after(move_piece),
            );
    }
}

#[derive(Component, Debug)]
pub struct Square {
    pub x: u8,
    pub y: u8,
}

#[derive(Resource)]
pub struct SquareMaterials {
    black_color: Handle<StandardMaterial>,
    white_color: Handle<StandardMaterial>,
}

impl FromWorld for SquareMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        SquareMaterials {
            black_color: materials.add(Color::rgb(0., 0.1, 0.1).into()),
            white_color: materials.add(Color::rgb(1., 0.9, 0.9).into()),
        }
    }
}

#[derive(Default, Resource)]
pub struct SelectedSquare {
    entity: Option<Entity>,
}

#[derive(Default, Resource)]
pub struct SelectedPiece {
    entity: Option<Entity>,
}

pub fn create_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<SquareMaterials>,
) {
    // Add meshes and materials
    let mesh = meshes.add(Mesh::from(shape::Plane::from_size(1.)));

    for i in 0..8 {
        for j in 0..8 {
            let initial_mat = if (i + j + 1) % 2 == 0 {
                materials.white_color.clone()
            } else {
                materials.black_color.clone()
            };

            commands
                .spawn((
                    PbrBundle {
                        mesh: mesh.clone(),
                        material: initial_mat.clone(),
                        transform: Transform::from_translation(Vec3::new(i as f32, 0., j as f32)),
                        ..default()
                    },
                    PickableBundle::default(),
                ))
                .insert(Square { x: i, y: j });
        }
    }
}

fn despawn_taken_pieces(
    mut commands: Commands,
    mut app_exit_events: EventWriter<AppExit>,
    query: Query<(Entity, &Piece, With<Taken>)>,
) {
    for (entity, piece, _taken) in query.iter() {
        info!("despawning piece: {:?} {:?}", entity, piece.piece_type);
        // If the king is taken, we should exit
        if piece.piece_type == PieceType::King {
            println!(
                "{} won! Thanks for playing!",
                match piece.color {
                    PieceColor::White => "Black",
                    PieceColor::Black => "White",
                }
            );
            app_exit_events.send(AppExit);
        }

        // Despawn piece and children
        commands.entity(entity).despawn_recursive();
    }
}

fn move_piece(
    mut commands: Commands,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    mut turn: ResMut<PlayerTurn>,
    squares_query: Query<&Square>,
    mut pieces_query: Query<(Entity, &mut Piece)>,
) {
    if !selected_square.is_changed() {
        return;
    }

    let square_entity = if let Some(entity) = selected_square.entity {
        entity
    } else {
        return;
    };

    let square = if let Ok(square) = squares_query.get(square_entity) {
        square
    } else {
        return;
    };

    if let Some(selected_piece_entity) = selected_piece.entity {
        let pieces_vec = pieces_query.iter_mut().map(|(_, piece)| *piece).collect();
        let pieces_entity_vec = pieces_query
            .iter_mut()
            .map(|(entity, piece)| (entity, *piece))
            .collect::<Vec<(Entity, Piece)>>();
        // Move the selected piece to the selected square
        let mut piece =
            if let Ok((_piece_entity, piece)) = pieces_query.get_mut(selected_piece_entity) {
                piece
            } else {
                return;
            };

        if piece.is_move_valid((square.x, square.y), pieces_vec) {
            // Check if a piece of the opposite color exists in this square and despawn it
            info!("move_piece and valid_move");
            for (other_entity, other_piece) in pieces_entity_vec {
                if other_piece.x == square.x
                    && other_piece.y == square.y
                    && other_piece.color != piece.color
                {
                    // Mark the piece as taken
                    info!("piece taken");
                    commands.entity(other_entity).insert(Taken);
                }
            }

            // Move piece
            piece.x = square.x;
            piece.y = square.y;

            // Change turn
            turn.change_turn();

            selected_square.entity = None;
            selected_piece.entity = None;
        } else {
            info!("invalid move or piece selected");
        }
    }
}

fn select_piece(
    selected_square: Res<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    turn: Res<PlayerTurn>,
    squares_query: Query<&Square>,
    pieces_query: Query<(Entity, &Piece)>,
) {
    if !selected_square.is_changed() {
        return;
    }

    info!("Select_piece");

    let square_entity = if let Some(entity) = selected_square.entity {
        entity
    } else {
        return;
    };

    let square = if let Ok(square) = squares_query.get(square_entity) {
        square
    } else {
        return;
    };

    // Select the piece in the currently selected square
    for (piece_entity, piece) in pieces_query.iter() {
        if piece.x == square.x && piece.y == square.y && piece.color == turn.0 {
            // piece_entity is now the entity in the same square
            info!("Selecting piece: {}", piece_entity.index());
            selected_piece.entity = Some(piece_entity);
            break;
        }
    }
}

fn select_square(
    mut events: EventReader<PickingEvent>,
    mouse_button_inputs: Res<Input<MouseButton>>,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    squares_query: Query<&Square>,
) {
    // Only run if the left button is pressed
    if !mouse_button_inputs.just_pressed(MouseButton::Left) {
        return;
    }

    for event in events.iter() {
        if let PickingEvent::Selection(e) = event {
            info!("Selection");
            if let SelectionEvent::JustSelected(selected_entity) = e {
                if let Ok(_current_square) = squares_query.get(*selected_entity) {
                    info!("Selecting square: {}", selected_entity.index());
                    selected_square.entity = Some(*selected_entity);
                } else {
                    // Nothing selected
                    // not working
                    info!("Deselecting.");
                    selected_square.entity = None;
                    selected_piece.entity = None;
                    break;
                };
            }
        }
    }
}

