//! Local player: physics stepping, block targeting, mining and placing.
//!
//! In M1 this runs against the locally generated preview world. The same systems become the
//! client's prediction layer in M2, with the server's authoritative state correcting them.

use bevy::prelude::*;

use deepcore_shared::TICK_SECONDS;
use deepcore_shared::content::{ItemCategory, Registry};
use deepcore_shared::crafting::{MineCheck, check_mine, ticks_for};
use deepcore_shared::movement::{BodyState, MovementConfig, query_from_blocks, step};
use deepcore_shared::voxel::raycast::raycast;
use deepcore_shared::voxel::{BlockId, BlockPos, Dir};

use crate::Content;
use crate::input::{InputState, Look};
use crate::world::ClientWorld;

/// The local player.
#[derive(Component, Debug, Clone)]
pub struct LocalPlayer {
    /// Physics state, shared with the server.
    pub body: BodyState,
    /// The hotbar: nine item stacks.
    pub hotbar: [Option<HotbarSlot>; 9],
    /// Selected hotbar index.
    pub selected: usize,
    /// Ticks spent mining the current target.
    pub mining_ticks: u32,
    /// The block currently being mined.
    pub mining_target: Option<BlockPos>,
    /// Ticks required to break the current target.
    pub mining_required: u32,
    /// Why the current target cannot be mined, if it cannot.
    pub blocked_reason: Option<String>,
}

/// One hotbar entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotbarSlot {
    /// Item in the slot.
    pub item: deepcore_shared::content::ItemId,
    /// Stack size.
    pub count: u32,
}

impl LocalPlayer {
    /// Creates a player standing at `spawn`.
    ///
    /// The starting loadout is exactly one steel pickaxe and nothing else, per the design:
    /// every lamp, ladder and support must be crafted from gathered materials.
    #[must_use]
    pub fn new(spawn: Vec3, registry: &Registry) -> Self {
        let mut hotbar = [None; 9];
        if let Some(pickaxe) = registry.item_id("steel_pickaxe") {
            hotbar[0] = Some(HotbarSlot { item: pickaxe, count: 1 });
        }
        Self {
            body: BodyState { position: spawn, ..Default::default() },
            hotbar,
            selected: 0,
            mining_ticks: 0,
            mining_target: None,
            mining_required: 0,
            blocked_reason: None,
        }
    }

    /// The selected slot, if any.
    #[must_use]
    pub fn selected_slot(&self) -> Option<HotbarSlot> {
        self.hotbar[self.selected]
    }
}

/// What the crosshair is pointing at.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct Targeting {
    /// The hit block, if any.
    pub hit: Option<BlockPos>,
    /// The face the ray entered.
    pub face: Option<Dir>,
    /// Distance in blocks.
    pub distance: f32,
    /// True when the current tool cannot break the hit block.
    pub too_hard: bool,
    /// Tier the block requires, for the "needs Tier N" message.
    pub required_tier: u8,
}

/// The mining crack overlay, drawn as a wireframe box on the targeted block.
#[derive(Resource, Default)]
pub struct SelectionOutline {
    /// The outline entity, or `None` when nothing is targeted.
    pub entity: Option<Entity>,
}

/// Player tuning.
#[derive(Resource)]
pub struct PlayerConfig {
    /// Shared movement configuration.
    pub movement: MovementConfig,
    /// Reach in blocks for mining and placing.
    pub reach: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self { movement: MovementConfig::default(), reach: 5.0 }
    }
}

/// Registers local player systems.
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Targeting>()
            .init_resource::<SelectionOutline>()
            .init_resource::<PlayerConfig>()
            .add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (select_hotbar, step_player, update_targeting, update_outline, mine_and_place)
                    .chain(),
            );
    }
}

/// Spawns the local player at the camp.
fn spawn_player(
    mut commands: Commands,
    content: Res<Content>,
    seed: Res<crate::Seed>,
    mut world: ResMut<ClientWorld>,
) {
    let generator = deepcore_shared::worldgen::WorldGen::new(seed.0, content.registry.clone());
    let spawn = generator.spawn_point();
    let centre = deepcore_shared::voxel::ChunkPos::from_block(BlockPos::from_world(spawn));
    // The chunk streaming system fills in the rest as the player moves.
    let generated = world.generate_area(&generator, centre, 1);
    info!("local preview: {generated} chunks generated around the camp at {spawn:?}");

    commands.spawn((
        LocalPlayer::new(spawn, &content.registry),
        Look::default(),
        Transform::from_translation(spawn),
    ));
    commands.spawn((Camera3d::default(), Transform::from_translation(spawn + Vec3::Y * 1.62)));
}

/// Number keys and the scroll wheel pick the hotbar slot.
fn select_hotbar(
    keys: Res<ButtonInput<KeyCode>>,
    mut input: ResMut<InputState>,
    mut player: Single<&mut LocalPlayer>,
) {
    let digits = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];
    for (i, key) in digits.iter().enumerate() {
        if keys.just_pressed(*key) {
            player.selected = i;
        }
    }
    if input.hotbar_scroll.abs() > 0.1 {
        let delta = if input.hotbar_scroll > 0.0 { -1 } else { 1 };
        let slot = (player.selected as i32 + delta).rem_euclid(9) as usize;
        player.selected = slot;
        input.hotbar_scroll = 0.0;
    }
}

/// Steps the local player's physics with the shared movement code.
///
/// Runs on a fixed accumulator so prediction matches the server tick for tick, no matter what
/// the frame rate is.
fn step_player(
    time: Res<Time>,
    input: Res<InputState>,
    config: Res<PlayerConfig>,
    content: Res<Content>,
    world: Res<ClientWorld>,
    mut player: Single<&mut LocalPlayer>,
    mut accumulator: Local<f32>,
) {
    *accumulator += time.delta_secs();
    let ladder = content.registry.block_id("ladder").unwrap_or(BlockId::AIR);
    let mut ticks = 0;
    while *accumulator >= TICK_SECONDS && ticks < 5 {
        let query = query_from_blocks(|p| world.block_at(p), ladder);
        let result = step(player.body, &input.intent, &config.movement, TICK_SECONDS, &query);
        player.body = result.body;
        *accumulator -= TICK_SECONDS;
        ticks += 1;
    }
    if *accumulator > TICK_SECONDS * 5.0 {
        *accumulator = 0.0;
    }
}

/// Raycasts from the eye to find the targeted block and whether it can be mined.
fn update_targeting(
    content: Res<Content>,
    config: Res<PlayerConfig>,
    world: Res<ClientWorld>,
    player: Single<(&LocalPlayer, &Look)>,
    mut targeting: ResMut<Targeting>,
) {
    let (player, look) = *player;
    let eye = player.body.position + Vec3::Y * 1.62;
    let hit = raycast(eye, look.forward(), config.reach, |p| !world.block_at(p).is_air());

    let Some(hit) = hit else {
        *targeting = Targeting::default();
        return;
    };

    let block_def = content.registry.block(world.block_at(hit.block));
    let tool = player.selected_slot().and_then(|slot| content.registry.tool(slot.item));
    let (too_hard, required_tier) = match block_def {
        Some(def) => {
            match check_mine(def.def.hardness, def.def.min_tier, def.def.unbreakable, tool, 1.0) {
                MineCheck::Allowed { .. } => (false, 0),
                MineCheck::TooHard { required_tier } => (true, required_tier),
                MineCheck::Unbreakable => (true, 99),
            }
        }
        None => (false, 0),
    };

    *targeting = Targeting {
        hit: Some(hit.block),
        face: Some(hit.face),
        distance: hit.distance,
        too_hard,
        required_tier,
    };
}

/// Keeps the selection box in sync with the targeted block.
///
/// The box is a single long-lived entity; only its transform and material colour change, so
/// nothing is allocated while the player looks around.
fn update_outline(
    mut commands: Commands,
    targeting: Res<Targeting>,
    mut outline: ResMut<SelectionOutline>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<
        (&mut Transform, &mut Visibility, &MeshMaterial3d<StandardMaterial>),
        With<SelectionBox>,
    >,
) {
    if outline.entity.is_none() {
        let mesh = meshes.add(Cuboid::new(1.004, 1.004, 1.004));
        let material = materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, 0.35),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        });
        let entity = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::default(),
                Visibility::Hidden,
                SelectionBox,
            ))
            .id();
        outline.entity = Some(entity);
        return;
    }
    let Some(entity) = outline.entity else {
        return;
    };
    let Ok((mut transform, mut visibility, material)) = query.get_mut(entity) else {
        return;
    };
    match targeting.hit {
        Some(hit) => {
            transform.translation = hit.center();
            *visibility = Visibility::Visible;
            if let Some(mut mat) = materials.get_mut(&material.0) {
                mat.base_color = if targeting.too_hard {
                    Color::srgba(1.0, 0.2, 0.15, 0.45)
                } else {
                    Color::srgba(1.0, 1.0, 1.0, 0.35)
                };
            }
        }
        None => *visibility = Visibility::Hidden,
    }
}

/// Marker for the selection outline entity.
#[derive(Component)]
pub struct SelectionBox;

/// Mines the targeted block over time, and places blocks on right click.
fn mine_and_place(
    mut commands: Commands,
    content: Res<Content>,
    input: Res<InputState>,
    targeting: Res<Targeting>,
    mut world: ResMut<ClientWorld>,
    mut player: Single<&mut LocalPlayer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut dirty: ResMut<crate::render::DirtyChunks>,
) {
    // --- mining ---
    if !input.mining {
        player.mining_ticks = 0;
        player.mining_target = None;
        player.blocked_reason = None;
    } else if let Some(hit) = targeting.hit {
        let tool = player.selected_slot().and_then(|slot| content.registry.tool(slot.item));
        let block_def = content.registry.block(world.block_at(hit));
        if let Some(def) = block_def {
            match check_mine(def.def.hardness, def.def.min_tier, def.def.unbreakable, tool, 1.0) {
                MineCheck::Allowed { seconds } => {
                    if player.mining_target != Some(hit) {
                        player.mining_target = Some(hit);
                        player.mining_ticks = 0;
                        player.mining_required = ticks_for(seconds, TICK_SECONDS).max(1);
                    }
                    player.mining_ticks += 1;
                    player.blocked_reason = None;
                    if player.mining_ticks >= player.mining_required {
                        // Break it: remove the block and (M3) grant the drops.
                        world.set_block(hit, BlockId::AIR);
                        dirty.0.push(deepcore_shared::voxel::ChunkPos::from_block(hit));
                        player.mining_ticks = 0;
                        player.mining_target = None;
                    }
                }
                MineCheck::TooHard { required_tier } => {
                    player.mining_ticks = 0;
                    player.blocked_reason = Some(format!("Too hard — needs Tier {required_tier}"));
                }
                MineCheck::Unbreakable => {
                    player.mining_ticks = 0;
                    player.blocked_reason = Some("Unbreakable".to_string());
                }
            }
        }
    }

    // --- placing ---
    if !input.use_pressed {
        return;
    }
    let (Some(hit), Some(face)) = (targeting.hit, targeting.face) else {
        return;
    };
    let Some(slot) = player.selected_slot() else {
        return;
    };
    let Some(item) = content.registry.item(slot.item) else {
        return;
    };
    let place_block = match &item.def.category {
        ItemCategory::Block => content.registry.block_id(&item.def.id),
        _ => None,
    };
    let Some(block) = place_block else {
        return;
    };

    let target = hit.offset_dir(face);
    if !world.block_at(target).is_air() {
        return;
    }
    // Do not place a block inside the player.
    let player_min = player.body.position;
    let player_max = player_min + Vec3::new(0.6, 1.8, 0.6);
    let block_min = Vec3::new(target.x as f32, target.y as f32, target.z as f32);
    let block_max = block_min + Vec3::ONE;
    let overlaps = block_min.x < player_max.x
        && block_max.x > player_min.x
        && block_min.y < player_max.y
        && block_max.y > player_min.y
        && block_min.z < player_max.z
        && block_max.z > player_min.z;
    if overlaps {
        return;
    }

    if world.set_block(target, block) {
        dirty.0.push(deepcore_shared::voxel::ChunkPos::from_block(target));
        let selected = player.selected;
        if let Some(slot) = &mut player.hotbar[selected] {
            slot.count = slot.count.saturating_sub(1);
            if slot.count == 0 {
                player.hotbar[selected] = None;
            }
        }
    }
    let _ = (&mut commands, &mut meshes, &mut materials);
}
