//! On-screen HUD: crosshair, hotbar, depth meter, and the "too hard" warning.
//!
//! Built with Bevy UI so it costs almost nothing per frame: the hotbar is rebuilt only when the
//! selection or contents change, and the text nodes update in place.

use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::ui::widget::Text;

use deepcore_shared::content::Registry;
use deepcore_shared::voxel::SURFACE_Y;

use crate::Content;
use crate::player::{LocalPlayer, Targeting};
use crate::world::ClientWorld;

/// Marker for the depth meter text.
#[derive(Component)]
struct DepthText;

/// Marker for the selected-block text.
#[derive(Component)]
struct TargetText;

/// Marker for the warning text ("Too hard — needs Tier N").
#[derive(Component)]
struct WarningText;

/// Marker for one hotbar slot's label.
#[derive(Component)]
struct HotbarLabel(usize);

/// Marker for the hotbar container.
#[derive(Component)]
struct HotbarRoot;

/// Marker for a slot's border, so the selected slot can be highlighted.
#[derive(Component)]
struct HotbarSlotBorder(usize);

/// Registers the HUD.
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, (update_depth, update_target, update_hotbar));
    }
}

/// Builds the static HUD layout once.
fn spawn_hud(mut commands: Commands) {
    // Root container spanning the window.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            // Crosshair: four small bars forming a cross at the centre.
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    width: Val::Px(2.0),
                    height: Val::Px(2.0),
                    margin: UiRect::all(Val::Px(-1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                Pickable::IGNORE,
            ));

            // Depth meter, top left.
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(16.0),
                    top: Val::Px(12.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|column| {
                column.spawn((
                    Text::new("Depth: 0 m"),
                    TextFont { font_size: FontSize::Px(20.0), ..default() },
                    TextColor(Color::WHITE),
                    DepthText,
                ));
                column.spawn((
                    Text::new(""),
                    TextFont { font_size: FontSize::Px(15.0), ..default() },
                    TextColor(Color::srgba(0.85, 0.9, 1.0, 0.9)),
                    TargetText,
                ));
                column.spawn((
                    Text::new(""),
                    TextFont { font_size: FontSize::Px(15.0), ..default() },
                    TextColor(Color::srgb(1.0, 0.35, 0.3)),
                    WarningText,
                ));
            });

            // Hotbar, bottom centre.
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(18.0),
                    left: Val::Percent(50.0),
                    margin: UiRect::left(Val::Px(-234.0)),
                    width: Val::Px(468.0),
                    height: Val::Px(52.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(4.0),
                    ..default()
                },
                HotbarRoot,
                Pickable::IGNORE,
            ))
            .with_children(|bar| {
                for i in 0..9 {
                    bar.spawn((
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor::all(if i == 0 {
                            Color::srgb(0.95, 0.85, 0.45)
                        } else {
                            Color::srgba(0.0, 0.0, 0.0, 0.7)
                        }),
                        HotbarSlotBorder(i),
                        BackgroundColor(Color::srgba(0.08, 0.09, 0.11, 0.65)),
                        children![(
                            Text::new(format!("{}", i + 1)),
                            TextFont { font_size: FontSize::Px(13.0), ..default() },
                            TextColor(Color::srgba(0.8, 0.85, 0.9, 0.75)),
                            HotbarLabel(i),
                        )],
                    ));
                }
            });
        });
}

/// Updates the depth meter and the current layer name.
fn update_depth(
    content: Res<Content>,
    player: Single<&LocalPlayer>,
    mut text: Single<&mut Text, With<DepthText>>,
) {
    let y = player.body.position.y;
    let depth = (SURFACE_Y as f32 - y).max(0.0);
    let layer = content
        .registry
        .layers
        .iter()
        .find(|l| depth >= l.def.depth.0 as f32 && depth < l.def.depth.1 as f32)
        .map_or("Surface", |l| l.def.name.as_str());
    let layer_text = if depth < 1.0 { "Camp" } else { layer };
    **text = Text::new(format!("Depth: {depth:.0} m   ·   {layer_text}"));
}

/// Shows what the crosshair is on, and warns when the tool is too weak.
fn update_target(
    content: Res<Content>,
    targeting: Res<Targeting>,
    world: Res<ClientWorld>,
    player: Single<&LocalPlayer>,
    mut target_text: Single<&mut Text, (With<TargetText>, Without<WarningText>)>,
    mut warning: Single<&mut Text, (With<WarningText>, Without<TargetText>)>,
) {
    let registry: &Registry = &content.registry;
    **target_text = match targeting.hit {
        Some(hit) => {
            let block = world.block_at(hit);
            let def = registry.block(block);
            let name = def.map_or("Unknown", |b| b.def.id.as_str()).replace('_', " ");
            let hardness = def.map_or(0.0, |b| b.def.hardness);
            let tier = targeting.required_tier;
            if targeting.too_hard {
                Text::new(format!("{name}  ·  needs Tier {tier}"))
            } else {
                Text::new(format!(
                    "{name}  ·  hardness {hardness:.1}  ·  {:.1} m",
                    targeting.distance
                ))
            }
        }
        None => Text::new(""),
    };
    **warning = match &player.blocked_reason {
        Some(reason) => Text::new(reason.clone()),
        None => Text::new(""),
    };
}

/// Renders the hotbar contents; cheap because it only touches nine labels.
fn update_hotbar(
    content: Res<Content>,
    player: Single<&LocalPlayer>,
    mut labels: Query<(&HotbarLabel, &mut Text)>,
    mut borders: Query<(&HotbarSlotBorder, &mut BorderColor)>,
) {
    for (border, mut color) in &mut borders {
        *color = BorderColor::all(if border.0 == player.selected {
            Color::srgb(0.95, 0.85, 0.45)
        } else {
            Color::srgba(0.0, 0.0, 0.0, 0.7)
        });
    }
    for (label, mut text) in &mut labels {
        let slot = player.hotbar[label.0];
        **text = match slot {
            Some(slot) => {
                let name = content.registry.item_name(slot.item);
                if slot.count > 1 {
                    format!("{}\n×{}", short_name(name), slot.count)
                } else {
                    short_name(name)
                }
            }
            None => String::new(),
        };
    }
}

/// First word of an item name, so it fits in a 48 px slot.
fn short_name(name: &str) -> String {
    name.split_whitespace().next().unwrap_or(name).to_string()
}
