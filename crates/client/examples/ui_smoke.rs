//! Minimal UI smoke test: proves Bevy UI and text render with this crate's feature set.
//!
//! Run with `cargo run -p deepcore_client --example ui_smoke` and screenshot the window. It is
//! kept in the tree because UI breakage after a Bevy upgrade is otherwise hard to attribute.

use bevy::prelude::*;
use bevy::text::FontSize;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, report_ui)
        .run();
}

/// Logs how many text nodes exist and where the panel landed, once at frame 60.
fn report_ui(
    mut frames: Local<u32>,
    texts: Query<&Text>,
    nodes: Query<(&ComputedNode, &GlobalTransform), With<BackgroundColor>>,
    all: Query<&ComputedNode>,
    raw: Query<Entity, With<Node>>,
) {
    *frames += 1;
    if *frames != 60 {
        return;
    }
    info!(
        "ui_smoke: {} text nodes, {} computed nodes, {} Node entities",
        texts.iter().count(),
        all.iter().count(),
        raw.iter().count()
    );
    for (node, transform) in &nodes {
        info!("ui_smoke: node {:?} at {:?}", node.size(), transform.translation());
    }
}

fn setup(mut commands: Commands) {
    // Reproduce the client exactly: a Camera3d spawned in a *different* Startup system from the
    // UI, plus an extra bare Camera that has no render graph.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn(Camera::default());

    // A large opaque panel in the middle of the screen.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(200.0),
            top: Val::Px(200.0),
            width: Val::Px(400.0),
            height: Val::Px(200.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.9, 0.2, 0.2)),
        children![(
            Text::new("UI works"),
            TextFont { font_size: FontSize::Px(48.0), ..default() },
            TextColor(Color::WHITE),
        )],
    ));
}
