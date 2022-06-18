use crate::{borad::*, pieces::*};
use bevy::prelude::*;

// Component to mark the Text entity
#[derive(Component)]
struct NextMoveText;

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_next_move_text)
            .add_system(next_move_text_update);
    }
}

/// Initialize UiCamera and text
fn init_next_move_text(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
) {
    let font = asset_server.load("ressources/FiraSans-Bold.ttf");

    commands.spawn_bundle(UiCameraBundle::default());

    // See: https://github.com/bevyengine/bevy/blob/main/examples/ui/text.rs#L34
    commands.spawn_bundle(
            TextBundle {
                style: Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: Rect {
                        bottom: Val::Px(5.0),
                        right: Val::Px(15.0),
                        ..default()
                    },
                    ..Default::default()
                },
                text: Text::with_section(
                    String::from("Next move: White"),
                     TextStyle {
                                font: font.clone(),
                                font_size: 42.0,
                                color: Color::BLACK
                            },
                    TextAlignment {
                     horizontal: HorizontalAlign::Center,
                        ..default()
                    }),
                ..Default::default()
            })
        .insert(NextMoveText);
}

fn next_move_text_update(
    turn: Res<PlayerTurn>,
    mut query: Query<&mut Text, With<NextMoveText>>,
) {
    if !turn.is_changed() {
        return
    }

    for mut text in query.iter_mut() {
        text.sections[0].value = format!(
            "Next move: {}",
            match turn.0 {
                PieceColor::White => "White",
                PieceColor::Black => "Black",
            }
        );
    }
}