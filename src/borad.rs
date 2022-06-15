use bevy::prelude::*;
use bevy_mod_picking::*;

use crate::pieces::*;

pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedSquare>()
            .init_resource::<SelectedPiece>()
            .add_startup_system(create_board)
            .add_system_to_stage(CoreStage::PostUpdate, select_square);
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

fn select_square(
    mut events: EventReader<PickingEvent>,
    mouse_button_inputs: Res<Input<MouseButton>>,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    squares_query: Query<&Square>,
    mut pieces_query: Query<(Entity, &mut Piece)>,
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

                        let pieces_vec = pieces_query.iter_mut().map(|(_, piece)| *piece).collect();
                        if let Ok((_piece_entity, mut piece)) = pieces_query.get_mut(selected_piece_entity)
                        {
                            if piece.is_move_valid((current_square.x, current_square.y), pieces_vec) {
                                info!("Moving piece from ({},{}) to ({},{})",
                                piece.x, piece.y, current_square.x, current_square.y );
                                piece.x = current_square.x;
                                piece.y = current_square.y;
                            }
                        }
                        selected_square.entity = None;
                        selected_piece.entity = None;
                    } else if selected_piece.entity.is_none() {
                        // no piece is selected
                        // we check if on this square stands a piece
                        for (piece_entity, piece) in pieces_query.iter_mut() {
                            if piece.x == current_square.x && piece.y == current_square.y {
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
