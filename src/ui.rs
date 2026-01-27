use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

use crate::raycast::HighlightState;

const UI_FONT_PATH: &str = "fonts/SourceHanSansSC-Regular.otf";
const MENU_BG: Color = Color::srgba(0.08, 0.09, 0.12, 0.92);
const MENU_OVERLAY: Color = Color::srgba(0.0, 0.0, 0.0, 0.45);
const INFO_BG: Color = Color::srgba(0.06, 0.08, 0.12, 0.78);
const BUTTON_NORMAL: Color = Color::srgb(0.20, 0.22, 0.28);
const BUTTON_HOVER: Color = Color::srgb(0.28, 0.30, 0.38);
const BUTTON_PRESSED: Color = Color::srgb(0.36, 0.12, 0.12);

#[derive(Resource, Default)]
pub struct MenuState {
    pub open: bool,
}

#[derive(Component)]
pub struct VoxelInfoText;

#[derive(Component)]
struct ExitMenuRoot;

#[derive(Component)]
struct ExitButton;

#[derive(Component)]
struct Crosshair;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .add_systems(Startup, setup_ui)
            .add_systems(
                Update,
                (update_voxel_info, toggle_exit_menu, exit_button_system),
            );
    }
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(UI_FONT_PATH);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(14.0),
                top: px(14.0),
                width: px(260.0),
                padding: UiRect::all(px(12.0)),
                ..default()
            },
            BackgroundColor(INFO_BG),
        ))
        .with_child((
            Text::new("注视方块：无"),
            TextFont {
                font: font.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
            VoxelInfoText,
        ));

    // 十字准星
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100.0),
                height: percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Crosshair,
        ))
        .with_children(|parent| {
            // 水平线
            parent.spawn((
                Node {
                    width: px(16.0),
                    height: px(2.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
            ));
            // 垂直线
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: px(2.0),
                    height: px(16.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
            ));
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100.0),
                height: percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(MENU_OVERLAY),
            Visibility::Hidden,
            ExitMenuRoot,
        ))
        .with_child((
            Node {
                width: px(320.0),
                padding: UiRect::all(px(18.0)),
                row_gap: px(12.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(MENU_BG),
            BorderColor::all(Color::srgb(0.5, 0.55, 0.62)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("暂停菜单"),
                TextFont {
                    font: font.clone(),
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new("按 Esc 返回游戏"),
                TextFont {
                    font: font.clone(),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.82, 0.9)),
            ));

            parent
                .spawn((
                    Button,
                    ExitButton,
                    Node {
                        width: percent(100.0),
                        height: px(44.0),
                        border: UiRect::all(px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(BUTTON_NORMAL),
                    BorderColor::all(Color::srgb(0.55, 0.6, 0.7)),
                ))
                .with_child((
                    Text::new("退出游戏"),
                    TextFont {
                        font: font.clone(),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
        });
}

fn update_voxel_info(
    highlight: Res<HighlightState>,
    mut text_q: Query<&mut Text, With<VoxelInfoText>>,
) {
    if !highlight.is_changed() {
        return;
    }
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };
    let value = match highlight.current {
        Some(hit) => {
            let def = hit.kind.def();
            format!(
                "注视方块：{}\n位置: ({}, {}, {})\n温度: {:.1}°C\n湿度: {:.2}\n硬度: {:.2}\n延展度: {:.2}",
                def.name,
                hit.pos.x,
                hit.pos.y,
                hit.pos.z,
                def.props.temperature,
                def.props.humidity,
                def.props.hardness,
                def.props.ductility
            )
        }
        None => "注视方块：无".to_string(),
    };
    text.0 = value;
}

fn toggle_exit_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<MenuState>,
    mut menu_q: Query<&mut Visibility, With<ExitMenuRoot>>,
    mut crosshair_q: Query<&mut Visibility, (With<Crosshair>, Without<ExitMenuRoot>)>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    menu_state.open = !menu_state.open;
    let Ok(mut visibility) = menu_q.single_mut() else {
        return;
    };
    let Ok(mut crosshair_visibility) = crosshair_q.single_mut() else {
        return;
    };

    if menu_state.open {
        *visibility = Visibility::Visible;
        *crosshair_visibility = Visibility::Hidden;
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    } else {
        *visibility = Visibility::Hidden;
        *crosshair_visibility = Visibility::Visible;
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }
}

fn exit_button_system(
    mut interaction_q: Query<
        (&Interaction, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<ExitButton>),
    >,
    mut text_q: Query<&mut Text>,
    mut app_exit_writer: MessageWriter<AppExit>,
) {
    for (interaction, mut color, children) in &mut interaction_q {
        let Ok(mut text) = text_q.get_mut(children[0]) else {
            continue;
        };
        match *interaction {
            Interaction::Pressed => {
                *color = BUTTON_PRESSED.into();
                text.0 = "正在退出...".to_string();
                app_exit_writer.write(AppExit::Success);
            }
            Interaction::Hovered => {
                *color = BUTTON_HOVER.into();
                text.0 = "退出游戏".to_string();
            }
            Interaction::None => {
                *color = BUTTON_NORMAL.into();
                text.0 = "退出游戏".to_string();
            }
        }
    }
}
