use bevy::prelude::*;
use bevy_mod_picking::*;


pub struct BoardPlugin;
impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedSquare>()
            .add_startup_system(create_board)
            .add_system(color_squares)
            .add_system(select_square);
    }
}


#[derive(Component)]
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

pub fn create_board (
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
)
{
    // Add meshes and materials
    let mesh = meshes.add(Mesh::from(shape::Plane{size: 1.}));
    // let white_material = materials.add(Color::rgb(1., 0.9, 0.9).into());
    // let black_material = materials.add(Color::rgb(0., 0.1, 0.1).into());

    for i in 0..8  {
        for j in 0..8 {
            commands
                .spawn_bundle(PbrBundle{
                    mesh: mesh.clone(),
                    material: if (i + j + 1) % 2 == 0 {
                        // white_material.clone()
                        materials.add(Color::rgb(1., 0.9, 0.9).into())
                    } else {
                        // black_material.clone()
                        materials.add(Color::rgb(0., 0.1, 0.1).into())
                    },
                    transform: Transform::from_translation(Vec3::new(i as f32, 0., j as f32)),
                    ..Default::default()})
                .insert_bundle(PickableBundle::default())
                .insert(Square {
                    x: i,
                    y: j,
                });
        }
    }
}


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
                info!("\ncolor_squares:: Event: {:?}\n", e);
                for (entity, square, material_handle) in query.iter() {
                    let material = materials.get_mut(material_handle).unwrap();

                    if selected_square.entity == Some(entity) {
                        // I added it first to always see the selected square, in blue
                        material.base_color = Color::rgb(0.1, 0.1, 0.9);

                    } else if let HoverEvent::JustEntered(hovered_entity) = e {
                        if *hovered_entity == entity {
                            info!("Just Entered Entity: {}", entity.id());
                            material.base_color = Color::rgb(0.8, 0.3, 0.3);
                        }

                    } else if let HoverEvent::JustLeft(hovered_entity) = e {
                        if *hovered_entity == entity {
                            info!("Just Left entity: {} which was {}", entity.id(), square.is_white());
                            material.base_color = if square.is_white() {
                                Color::rgb(1., 0.9, 0.9)
                            } else {
                                Color::rgb(0., 0.1, 0.1)
                            }
                        }
                    }
                }

                // for (entity, square, material_handle) in query.iter() {
                //     // Get the actual material
                //     let material = materials.get_mut(material_handle).unwrap();
                //
                //     // Change the material color
                //     material.base_color = if let HoverEvent::JustEntered(_entity) = e  {
                //         info!("first condition true");
                //         Color::rgb(0.8, 0.3, 0.3)
                //     } else if Some(entity) == selected_square.entity {
                //         Color::rgb(0.9, 0.1, 0.1)
                //     } else if let HoverEvent::JustLeft(_entity) = e {
                //         if square.is_white() {
                //             Color::rgb(1., 0.9, 0.9)
                //         } else {
                //             Color::rgb(0., 0.1, 0.1)
                //         }
                //     } else {
                //         info!("Sad");
                //         Color::rgb(0., 0.1, 0.1)
                //     };
                // }
            }
            _ => {}
        }
    }
}

fn select_square(
    mut events: EventReader<PickingEvent>,
    mouse_button_inputs: Res<Input<MouseButton>>,
    mut selected_square: ResMut<SelectedSquare>,
)
{
    // Only run if the left button is pressed
    if !mouse_button_inputs.just_pressed(MouseButton::Left) {
        return;
    }

    for event in events.iter() {
        match event {
            PickingEvent::Selection(e) => {
                if let SelectionEvent::JustSelected(ent) = e {
                    info!("Selecting square: {}", ent.id());
                    selected_square.entity = Some(*ent);
                } else {
                    info!("Deselecting.");
                    selected_square.entity = None;
                }
            },
            _ => {}
        }
    }
}