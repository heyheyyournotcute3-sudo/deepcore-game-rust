//! Movement: pure, deterministic AABB-vs-voxel collision and the player controller.
//!
//! The same code runs on the client (prediction) and the server (authority), so it must be
//! deterministic: fixed timestep, no randomness, no floating point accumulation that depends
//! on frame rate. Gravity and speeds come from [`MovementConfig`].

use bevy_math::Vec3;

use crate::voxel::{BlockId, BlockPos};

/// Tunables for the player controller.
#[derive(Debug, Clone, Copy)]
pub struct MovementConfig {
    /// Horizontal walk speed in blocks per second.
    pub walk_speed: f32,
    /// Sprint multiplier.
    pub sprint_mult: f32,
    /// Crouch multiplier.
    pub crouch_mult: f32,
    /// Upward velocity applied on jump, in blocks per second.
    pub jump_speed: f32,
    /// Downward acceleration in blocks per second squared.
    pub gravity: f32,
    /// Terminal fall speed.
    pub terminal_speed: f32,
    /// Player collision box half extents.
    pub half_extents: Vec3,
    /// Step height: blocks this low are climbed without jumping.
    pub step_height: f32,
    /// Climb speed while on a ladder.
    pub climb_speed: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            walk_speed: 4.4,
            sprint_mult: 1.6,
            crouch_mult: 0.45,
            jump_speed: 8.2,
            gravity: 26.0,
            terminal_speed: 60.0,
            half_extents: Vec3::new(0.3, 0.9, 0.3),
            step_height: 0.55,
            climb_speed: 3.0,
        }
    }
}

/// The physical state of a player that movement acts on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyState {
    /// Feet-centre position. The AABB is centred on `x`/`z` and spans `y ..= y + 2*half.y`.
    pub position: Vec3,
    /// Current velocity in blocks per second.
    pub velocity: Vec3,
    /// True when the body is standing on something.
    pub on_ground: bool,
    /// True when the body is against a ladder.
    pub on_ladder: bool,
}

impl Default for BodyState {
    fn default() -> Self {
        Self { position: Vec3::ZERO, velocity: Vec3::ZERO, on_ground: false, on_ladder: false }
    }
}

/// One tick of player input, already reduced to intent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveIntent {
    /// Desired horizontal movement in world space (`x`, `z`), each in `-1..=1`.
    ///
    /// Clients derive this from WASD and the camera yaw, which keeps the physics independent of
    /// where the player is looking — ladders and walls have no facing.
    pub dir: (f32, f32),
    /// Jump requested this tick.
    pub jump: bool,
    /// Sprint requested.
    pub sprint: bool,
    /// Crouch requested.
    pub crouch: bool,
    /// Look yaw in radians. Kept for replication; physics does not use it.
    pub yaw: f32,
}

impl Default for MoveIntent {
    fn default() -> Self {
        Self { dir: (0.0, 0.0), jump: false, sprint: false, crouch: false, yaw: 0.0 }
    }
}

/// A collision query: is this block solid for the player?
pub trait SolidQuery {
    /// True when the block at `pos` blocks movement.
    fn is_solid(&self, pos: BlockPos) -> bool;
    /// True when the block at `pos` is climbable (a ladder).
    fn is_ladder(&self, pos: BlockPos) -> bool {
        let _ = pos;
        false
    }
}

/// Closure-based solid query, convenient for tests and for the client's mesh cache.
pub struct FnQuery<F, L = fn(BlockPos) -> bool> {
    /// Solid test.
    pub solid: F,
    /// Ladder test.
    pub ladder: L,
}

impl<F: Fn(BlockPos) -> bool, L: Fn(BlockPos) -> bool> SolidQuery for FnQuery<F, L> {
    fn is_solid(&self, pos: BlockPos) -> bool {
        (self.solid)(pos)
    }
    fn is_ladder(&self, pos: BlockPos) -> bool {
        (self.ladder)(pos)
    }
}

/// Builds a query from a block lookup closure.
pub fn query_from_blocks<F>(block_at: F, ladder: BlockId) -> impl SolidQuery
where
    F: Fn(BlockPos) -> BlockId + Copy,
{
    FnQuery {
        solid: move |p: BlockPos| !block_at(p).is_air(),
        ladder: move |p: BlockPos| block_at(p) == ladder,
    }
}

/// Axis-aligned box in world space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    /// Minimum corner.
    pub min: Vec3,
    /// Maximum corner.
    pub max: Vec3,
}

impl Aabb {
    /// Builds the player box from a feet position.
    #[must_use]
    pub fn player(position: Vec3, half: Vec3) -> Self {
        Self {
            min: Vec3::new(position.x - half.x, position.y, position.z - half.z),
            max: Vec3::new(position.x + half.x, position.y + half.y * 2.0, position.z + half.z),
        }
    }

    /// True when this box overlaps any solid block.
    #[must_use]
    pub fn collides<Q: SolidQuery>(&self, query: &Q) -> bool {
        let min = BlockPos::from_world(self.min);
        let max = BlockPos::from_world(self.max);
        for x in min.x..=max.x {
            for y in min.y..=max.y {
                for z in min.z..=max.z {
                    if query.is_solid(BlockPos::new(x, y, z)) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

/// Result of one movement step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepResult {
    /// New body state.
    pub body: BodyState,
    /// True when the body was blocked horizontally this step.
    pub blocked_horizontal: bool,
    /// True when the body landed this step (was falling, now on ground).
    pub landed: bool,
}

/// Advances a body by one fixed tick.
///
/// Movement is resolved per axis so sliding along walls works and a fast fall cannot tunnel
/// through a floor at our speed range. The body is swept in small sub-steps when the per-tick
/// displacement exceeds half a block.
pub fn step<Q: SolidQuery>(
    body: BodyState,
    intent: &MoveIntent,
    config: &MovementConfig,
    dt: f32,
    query: &Q,
) -> StepResult {
    let was_on_ground = body.on_ground;
    let mut velocity = body.velocity;

    // --- desired horizontal velocity from the world-space intent ---
    let (wish_x, wish_z) = intent.dir;
    let input_len = (wish_x * wish_x + wish_z * wish_z).sqrt();
    let (wish_x, wish_z) =
        if input_len > 1.0 { (wish_x / input_len, wish_z / input_len) } else { (wish_x, wish_z) };
    let mut speed = config.walk_speed;
    if intent.sprint && !intent.crouch {
        speed *= config.sprint_mult;
    }
    if intent.crouch {
        speed *= config.crouch_mult;
    }
    velocity.x = wish_x * speed;
    velocity.z = wish_z * speed;

    // --- ladder check, against the column the body is about to occupy ---
    //
    // Testing the current column only would fail as soon as the body drifts off the ladder,
    // so we probe the destination column (where the horizontal move ends up) as well.
    let dest = body.position + Vec3::new(velocity.x, 0.0, velocity.z) * dt;
    let on_ladder = [0.2_f32, 0.9, 1.6].iter().any(|h| {
        query.is_ladder(BlockPos::from_world(body.position + Vec3::Y * *h))
            || query.is_ladder(BlockPos::from_world(dest + Vec3::Y * *h))
    });
    if on_ladder {
        // Climb up while pushing horizontally into the ladder, down while crouching.
        let pushing = input_len > 0.1;
        velocity.y = if pushing {
            config.climb_speed
        } else if intent.crouch {
            -config.climb_speed
        } else {
            0.0
        };
        // Hold position in the ladder column instead of walking past it.
        velocity.x = 0.0;
        velocity.z = 0.0;
    } else {
        velocity.y = (velocity.y - config.gravity * dt).max(-config.terminal_speed);
        if intent.jump && body.on_ground {
            velocity.y = config.jump_speed;
        }
    }

    // --- integrate with sub-steps so we never tunnel ---
    let displacement = velocity * dt;
    let steps = ((displacement.length() / 0.45).ceil() as usize).max(1);
    let sub_dt = dt / steps as f32;
    let mut position = body.position;
    let mut blocked_horizontal = false;
    let mut on_ground = false;

    for _ in 0..steps {
        // X axis
        let dx = velocity.x * sub_dt;
        if dx != 0.0 {
            let candidate = position + Vec3::X * dx;
            if Aabb::player(candidate, config.half_extents).collides(query) {
                if !try_step_up(position, dx, 0.0, config, query, &mut position) {
                    blocked_horizontal = true;
                }
            } else {
                position = candidate;
            }
        }
        // Z axis
        let dz = velocity.z * sub_dt;
        if dz != 0.0 {
            let candidate = position + Vec3::Z * dz;
            if Aabb::player(candidate, config.half_extents).collides(query) {
                if !try_step_up(position, 0.0, dz, config, query, &mut position) {
                    blocked_horizontal = true;
                }
            } else {
                position = candidate;
            }
        }
        // Y axis
        let dy = velocity.y * sub_dt;
        if dy != 0.0 {
            let candidate = position + Vec3::Y * dy;
            if Aabb::player(candidate, config.half_extents).collides(query) {
                if dy < 0.0 {
                    on_ground = true;
                    // Snap flush to the floor top.
                    let floor_y = position.y.floor();
                    position.y = floor_y.max(candidate.y);
                }
                velocity.y = 0.0;
            } else {
                position = candidate;
            }
        }
    }

    // A body resting exactly on a block counts as grounded even without downward motion.
    if !on_ground && velocity.y <= 0.0 {
        let probe = position - Vec3::Y * 0.02;
        if Aabb::player(probe, config.half_extents).collides(query) {
            on_ground = true;
        }
    }

    StepResult {
        body: BodyState { position, velocity, on_ground, on_ladder },
        blocked_horizontal,
        landed: on_ground && !was_on_ground,
    }
}

/// Attempts to climb a single block step in the given horizontal direction.
fn try_step_up<Q: SolidQuery>(
    position: Vec3,
    dx: f32,
    dz: f32,
    config: &MovementConfig,
    query: &Q,
    out: &mut Vec3,
) -> bool {
    for lift in [0.55_f32, config.step_height] {
        let raised = position + Vec3::Y * lift;
        let candidate = raised + Vec3::new(dx, 0.0, dz);
        let blocked = Aabb::player(candidate, config.half_extents).collides(query)
            || Aabb::player(raised, config.half_extents).collides(query);
        if !blocked {
            // Settle back down onto the step.
            let settled = candidate - Vec3::Y * lift;
            let settled_blocked = Aabb::player(settled, config.half_extents).collides(query);
            *out = if settled_blocked { candidate } else { settled };
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A world with a floor at y = 0 and nothing else.
    fn floor_query(p: BlockPos) -> bool {
        p.y <= 0
    }

    fn no_ladder(_: BlockPos) -> bool {
        false
    }

    fn flat() -> FnQuery<fn(BlockPos) -> bool, fn(BlockPos) -> bool> {
        FnQuery {
            solid: floor_query as fn(BlockPos) -> bool,
            ladder: no_ladder as fn(BlockPos) -> bool,
        }
    }

    #[test]
    fn gravity_pulls_body_down_to_the_floor() {
        let cfg = MovementConfig::default();
        let mut body = BodyState { position: Vec3::new(0.5, 5.0, 0.5), ..Default::default() };
        for _ in 0..120 {
            body = step(body, &MoveIntent::default(), &cfg, 1.0 / 30.0, &flat()).body;
        }
        assert!(body.on_ground, "body should land");
        assert!((body.position.y - 1.0).abs() < 0.1, "y = {}", body.position.y);
    }

    #[test]
    fn walking_forward_moves_in_the_look_direction() {
        let cfg = MovementConfig::default();
        let mut body = BodyState { position: Vec3::new(0.5, 1.0, 0.5), ..Default::default() };
        let intent = MoveIntent { dir: (0.0, -1.0), yaw: 0.0, ..Default::default() };
        for _ in 0..30 {
            body = step(body, &intent, &cfg, 1.0 / 30.0, &flat()).body;
        }
        assert!(body.position.z < 0.0, "intent -Z should move toward -Z");
        assert!((body.position.x - 0.5).abs() < 1e-3);
    }

    #[test]
    fn jump_leaves_the_ground_then_lands() {
        let cfg = MovementConfig::default();
        let mut body =
            BodyState { position: Vec3::new(0.5, 1.0, 0.5), on_ground: true, ..Default::default() };
        let jump = MoveIntent { jump: true, ..Default::default() };
        let after = step(body, &jump, &cfg, 1.0 / 30.0, &flat());
        assert!(after.body.position.y > 1.0, "should rise");
        body = after.body;
        for _ in 0..120 {
            body = step(body, &MoveIntent::default(), &cfg, 1.0 / 30.0, &flat()).body;
        }
        assert!(body.on_ground);
    }

    #[test]
    fn wall_blocks_horizontal_movement() {
        // Floor plus a wall at x >= 2.
        fn wall(p: BlockPos) -> bool {
            p.y <= 0 || (p.x >= 2 && p.y <= 3)
        }
        let q = FnQuery {
            solid: wall as fn(BlockPos) -> bool,
            ladder: no_ladder as fn(BlockPos) -> bool,
        };
        let cfg = MovementConfig::default();
        let mut body = BodyState { position: Vec3::new(0.5, 1.0, 0.5), ..Default::default() };
        let intent = MoveIntent { dir: (1.0, 0.0), yaw: 0.0, ..Default::default() };
        for _ in 0..120 {
            body = step(body, &intent, &cfg, 1.0 / 30.0, &q).body;
        }
        assert!(body.position.x < 2.0, "must not pass the wall, x = {}", body.position.x);
    }

    #[test]
    fn sprint_is_faster_than_walk() {
        let cfg = MovementConfig::default();
        let start = BodyState {
            position: Vec3::new(0.5, 1.0, 100.0),
            on_ground: true,
            ..Default::default()
        };
        let walk = MoveIntent { dir: (0.0, -1.0), ..Default::default() };
        let sprint = MoveIntent { dir: (0.0, -1.0), sprint: true, ..Default::default() };
        let mut a = start;
        let mut b = start;
        for _ in 0..30 {
            a = step(a, &walk, &cfg, 1.0 / 30.0, &flat()).body;
            b = step(b, &sprint, &cfg, 1.0 / 30.0, &flat()).body;
        }
        let da = (a.position.z - start.position.z).abs();
        let db = (b.position.z - start.position.z).abs();
        assert!(db > da * 1.4, "sprint {db} should clearly beat walk {da}");
    }

    #[test]
    fn movement_is_deterministic() {
        let cfg = MovementConfig::default();
        let run = || {
            let mut body = BodyState { position: Vec3::new(3.5, 4.0, 3.5), ..Default::default() };
            let intent =
                MoveIntent { dir: (0.6, -0.8), sprint: true, yaw: 1.2, ..Default::default() };
            for _ in 0..200 {
                body = step(body, &intent, &cfg, 1.0 / 30.0, &flat()).body;
            }
            body.position
        };
        assert_eq!(run(), run(), "same inputs must give identical results");
    }

    #[test]
    fn ladder_allows_climbing() {
        fn ladder(p: BlockPos) -> bool {
            p.x == 0 && p.z == 0 && p.y > 0 && p.y < 5
        }
        fn no_solid(_: BlockPos) -> bool {
            false
        }
        let q = FnQuery {
            solid: no_solid as fn(BlockPos) -> bool,
            ladder: ladder as fn(BlockPos) -> bool,
        };
        let cfg = MovementConfig::default();
        let mut body = BodyState { position: Vec3::new(0.5, 1.0, 0.5), ..Default::default() };
        let intent = MoveIntent { dir: (0.0, 1.0), ..Default::default() };
        let start_y = body.position.y;
        for _ in 0..30 {
            body = step(body, &intent, &cfg, 1.0 / 30.0, &q).body;
        }
        assert!(body.position.y > start_y + 1.0, "ladder should lift the body");
    }

    #[test]
    fn fast_fall_does_not_tunnel() {
        let cfg = MovementConfig::default();
        let mut body = BodyState {
            position: Vec3::new(0.5, 40.0, 0.5),
            velocity: Vec3::new(0.0, -60.0, 0.0),
            ..Default::default()
        };
        for _ in 0..200 {
            body = step(body, &MoveIntent::default(), &cfg, 1.0 / 30.0, &flat()).body;
        }
        assert!(body.position.y > 0.5, "must not fall through the floor, y = {}", body.position.y);
    }
}
