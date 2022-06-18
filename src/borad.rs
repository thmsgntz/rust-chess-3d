use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_mod_picking::*;

use crate::pieces::*;

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
        app
            .init_resource::<SelectedSquare>()
            .init_resource::<SelectedPiece>()
            .init_resource::<PlayerTurn>()
            .add_startup_system(create_board)
            .add_system_to_stage(CoreStage::PostUpdate,select_piece)
            .add_system_to_stage(CoreStage::PostUpdate, select_square.before(select_piece))
            .add_system_to_stage(CoreStage::PostUpdate,move_piece.after(select_piece))
            .add_system_to_stage(CoreStage::PostUpdate,despawn_taken_pieces.after(move_piece));
    }
}

#[derive(Component, Debug)]
pub struct Square {
    pub x: u8,
    pub y: u8,
}

impl Square {
    fn is_white(&self) -> bool {
        (self.x + self.y + 1) % 2 == 0
    }
}

/* We're deriving Default so that when the plugin is initialized it starts with a None value,
but we could provide an initial value in case we wanted to, by implementing FromResources.
You can see how to do that with this Bevy example.
https://github.com/bevyengine/bevy/blob/841755aaf23acfd55b375c37390daeb302c5b30b/examples/ecs/state.rs#L160
*/
#[derive(Default)]
pub struct SelectedSquare {
    entity: Option<Entity>,
}

#[derive(Default)]
pub struct SelectedPiece {
    entity: Option<Entity>,
}

pub fn create_board (
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
)
{
    // Add meshes and materials
    let mesh = meshes.add(Mesh::from(shape::Plane{size: 1.}));
    let white_material = materials.add(Color::rgb(1., 0.9, 0.9).into());
    let black_material = materials.add(Color::rgb(0., 0.1, 0.1).into());
    let hovered_material = materials.add(Color::rgb(1., 0.0, 0.0).into());
    let selected_material = materials.add(Color::rgb(9.,0.,9.).into());

    for i in 0..8  {
        for j in 0..8 {
            let initial_mat = if (i + j + 1) % 2 == 0 {
                white_material.clone()
            } else {
                black_material.clone()
            };

            commands
                .spawn_bundle(PbrBundle{
                    mesh: mesh.clone(),
                    material: initial_mat.clone(),
                    transform: Transform::from_translation(Vec3::new(i as f32, 0., j as f32)),
                    ..Default::default()})
                .insert(Square {
                    x: i,
                    y: j,
                })
                .insert_bundle(PickableBundle {
                    pickable_mesh: Default::default(),
                    interaction: Default::default(),
                    focus_policy: Default::default(),
                    pickable_button: PickableButton {
                        initial: Some(initial_mat.clone()),
                        hovered: Some(hovered_material.clone()),
                        pressed: None,
                        selected: Some(selected_material.clone())},
                    selection: Default::default(),
                    hover: Default::default()
                });
        }
    }
}

fn despawn_taken_pieces(
    mut commands: Commands,
    mut app_exit_events: EventWriter<AppExit>,
    query: Query<(Entity, &Piece, With<Taken>)>,
) {

    for (entity, piece, _taken) in query.iter() {
        info!("despawn_taken_pieces");
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
        return
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
        }
    }
}


fn select_piece (
    selected_square: Res<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    turn: Res<PlayerTurn>,
    squares_query: Query<&Square>,
    pieces_query: Query<(Entity, &Piece)>,
) {
    if !selected_square.is_changed() {
        return
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
            info!("Selecting piece: {}", piece_entity.id());
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
        match event {
            PickingEvent::Selection(e) => {
                info!("Selection");
                if let SelectionEvent::JustSelected(selected_entity) = e {
                    if let Ok(_current_square) = squares_query.get(*selected_entity) {
                        info!("Selecting square: {}", selected_entity.id());
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
            },
            _ => {},
        }
    }
}

#[allow(dead_code)]
fn select_square_old(
    mut commands: Commands,
    mut events: EventReader<PickingEvent>,
    mut app_exit_events: EventWriter<AppExit>,
    mouse_button_inputs: Res<Input<MouseButton>>,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    mut turn: ResMut<PlayerTurn>,
    squares_query: Query<&Square>,
    mut pieces_query: Query<(Entity, &mut Piece, &Children)>,
)
{
    // Only run if the left button is pressed
    if !mouse_button_inputs.just_pressed(MouseButton::Left) {
        return;
    }


    for event in events.iter() {
        match event {
            PickingEvent::Selection(e) => {
                info!("Selection");
                if let SelectionEvent::JustSelected(selected_entity) = e {
                    let current_square = if let Ok(current_square) = squares_query.get(*selected_entity) {
                        current_square
                    } else {
                        // Nothing selected
                        // not working
                        info!("Deselecting.");
                        selected_square.entity = None;
                        selected_piece.entity = None;
                        break;
                    };

                    info!("Selecting entity: {}", selected_entity.id());
                    selected_square.entity = Some(*selected_entity);

                    if let Some(selected_piece_entity) = selected_piece.entity {
                        // a piece is already selected, move it to the selected square
                        info!("Piece selected and square selected");

                        let pieces_vec = pieces_query
                            .iter_mut()
                            .map(|(_, piece, _)| *piece)
                            .collect();

                        let pieces_entity_vec: Vec<(Entity, Piece, Vec<Entity>)> = pieces_query
                            .iter_mut()
                            .map(|(entity, piece, children)| {
                                (
                                    entity,
                                    *piece,
                                    children.iter().map(|entity| *entity).collect(),
                                )
                            })
                            .collect();

                        if let Ok((_piece_entity, mut piece, _)) = pieces_query.get_mut(selected_piece_entity)
                        {
                            if piece.is_move_valid((current_square.x, current_square.y), pieces_vec) {
                                info!("Moving piece from ({},{}) to ({},{})",
                                piece.x, piece.y, current_square.x, current_square.y );

                                for (other_entity, other_piece, other_children) in pieces_entity_vec {
                                    if other_piece.x == current_square.x
                                        && other_piece.y == current_square.y
                                        && other_piece.color != piece.color
                                    {
                                        // If the king is taken, we should exit
                                        if other_piece.piece_type == PieceType::King {
                                            println!(
                                                "{} won! Thanks for playing!",
                                                match turn.0 {
                                                    PieceColor::White => "White",
                                                    PieceColor::Black => "Black",
                                                }
                                            );
                                            app_exit_events.send(AppExit);
                                        }

                                        // Despawn piece
                                        commands.entity(other_entity).despawn();
                                        // Despawn all of it's children
                                        for child in other_children {
                                            commands.entity(child).despawn();
                                        }
                                    }
                                }

                                piece.x = current_square.x;
                                piece.y = current_square.y;

                                turn.0 = match turn.0 {
                                    PieceColor::White => PieceColor::Black,
                                    PieceColor::Black => PieceColor::White,
                                }
                            }
                        }
                        selected_square.entity = None;
                        selected_piece.entity = None;
                    } else if selected_piece.entity.is_none() {
                        // no piece is selected
                        // we check if on this square stands a piece
                        for (piece_entity, piece, _) in pieces_query.iter_mut() {
                            if piece.x == current_square.x
                                && piece.y == current_square.y
                                && piece.color == turn.0
                            {
                                // piece_entity is now the entity in the same square
                                selected_piece.entity = Some(piece_entity);
                                info!("Selecting piece: {}", piece_entity.id());
                            }
                        }
                    }
                } else {
                    //info!("Hello from deselect");
                }
            }
            PickingEvent::Hover(_) => {
                //info!("Hello from hover");
            }
            PickingEvent::Clicked(_) => {
                //info!("Hello from clicked");
            }
        }
    }
}

#[allow(dead_code)]
fn color_squares(
    mut events: EventReader<PickingEvent>,
    selected_square: Res<SelectedSquare>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &Square, &Handle<StandardMaterial>)>,
)
{
    // Helpful:
    //  Get entity from Event: https://stackoverflow.com/a/72260406
    //  bevy_mod_picking event example: https://github.com/aevyrie/bevy_mod_picking/blob/main/examples/events.rs
    //  bevy_picking doc: https://docs.rs/bevy_mod_picking/latest/bevy_mod_picking/index.html
    for event in events.iter() {
        match event {
            PickingEvent::Hover(e) => {
                /*  Example of USE:

                     if let HoverEvent::JustEntered(ent) = e {
                         info!("ID of entity: {}", ent.id());
                         for (entity, square, handled) in query.iter() {
                             if entity == *ent {
                                 info!("\t=>Found !")
                             }
                         }
                     }
                */
                info!("color_squares:: Event: {:?}", e);

                let (HoverEvent::JustEntered(hovered_entity) |
                HoverEvent::JustLeft(hovered_entity)) = e;

                let (entity, square, material_handle) = query.get(*hovered_entity).unwrap();

                if selected_square.entity == Some(entity) {
                    // I added it first to always see the selected square, in blue
                    let material = materials.get_mut(material_handle).unwrap();
                    material.base_color = Color::rgb(0.9, 0.1, 0.9);


                } else if let HoverEvent::JustEntered(hovered_entity) = e {
                    if *hovered_entity == entity {
                        let material = materials.get_mut(material_handle).unwrap();
                        info!("Just Entered Entity: {}", entity.id());
                        material.base_color = Color::rgb(0.8, 0.3, 0.3);
                    }

                } else if let HoverEvent::JustLeft(hovered_entity) = e {
                    if *hovered_entity == entity {
                        let material = materials.get_mut(material_handle).unwrap();
                        material.base_color = if square.is_white() {
                            info!("Just Left entity: {} which was white", entity.id());
                            Color::rgb(1., 0.9, 0.9)
                        } else {
                            info!("Just Left entity: {} which was black", entity.id());
                            Color::rgb(0., 0.1, 0.1)
                        };
                    }
                }

                info!("\n");
            }
            _ => {}
        }
    }
}
