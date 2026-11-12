//! First-person input: mouse look, WASD, and converting them into the world-space
//! [`MoveIntent`](deepcore_shared::movement::MoveIntent) that the shared physics consumes.

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use deepcore_shared::movement::MoveIntent;

/// Look angles of the local player.
#[derive(Component, Debug, Clone, Copy)]
pub struct Look {
    /// Yaw in radians. 0 looks down -Z.
    pub yaw: f32,
    /// Pitch in radians, clamped to just under ±90°.
    pub pitch: f32,
}

impl Default for Look {
    fn default() -> Self {
        Self { yaw: 0.0, pitch: 0.0 }
    }
}

impl Look {
    /// Unit forward vector.
    #[must_use]
    pub fn forward(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        Vec3::new(-sy * cp, sp, -cy * cp)
    }

    /// Unit right vector (no pitch).
    #[must_use]
    pub fn right(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        Vec3::new(cy, 0.0, -sy)
    }
}

/// The local player's input state for this frame.
#[derive(Resource, Default)]
pub struct InputState {
    /// Keys currently held, reduced to an intent each frame.
    pub intent: MoveIntent,
    /// True while the left mouse button is held.
    pub mining: bool,
    /// True while the right mouse button was pressed this frame.
    pub use_pressed: bool,
    /// True when the inventory screen wants the cursor.
    pub ui_captures_mouse: bool,
    /// Mouse wheel delta this frame.
    pub hotbar_scroll: f32,
}

/// Captures the mouse on startup so the first mouse motion turns the camera.
pub fn grab_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;
}

/// Toggles the cursor lock with Escape so the window can be left.
pub fn toggle_cursor_lock(
    keys: Res<ButtonInput<KeyCode>>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        let locked = cursor.grab_mode == CursorGrabMode::Locked;
        cursor.grab_mode = if locked { CursorGrabMode::None } else { CursorGrabMode::Locked };
        cursor.visible = locked;
    }
}

/// Applies mouse motion to the player's look angles.
pub fn mouse_look(
    mut motion: MessageReader<MouseMotion>,
    input: Res<InputState>,
    mut look: Single<&mut Look>,
) {
    if input.ui_captures_mouse {
        motion.clear();
        return;
    }
    let mut delta = Vec2::ZERO;
    for event in motion.read() {
        delta += event.delta;
    }
    let sensitivity = 0.0022;
    look.yaw -= delta.x * sensitivity;
    look.pitch = (look.pitch - delta.y * sensitivity)
        .clamp(-std::f32::consts::FRAC_PI_2 + 0.01, std::f32::consts::FRAC_PI_2 - 0.01);
}

/// Builds the world-space movement intent from the keyboard and the current look.
pub fn gather_movement(
    keys: Res<ButtonInput<KeyCode>>,
    look: Single<&Look>,
    mut input: ResMut<InputState>,
) {
    let mut forward = 0.0;
    let mut right = 0.0;
    if keys.pressed(KeyCode::KeyW) {
        forward += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        forward -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        right += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        right -= 1.0;
    }

    let look_right = look.right();
    let look_forward = Vec3::new(-look.yaw.sin(), 0.0, -look.yaw.cos());
    let wish = look_right * right + look_forward * forward;
    let wish = if wish.length_squared() > 1.0 { wish.normalize() } else { wish };

    input.intent = MoveIntent {
        dir: (wish.x, wish.z),
        jump: keys.pressed(KeyCode::Space),
        sprint: keys.pressed(KeyCode::ShiftLeft),
        crouch: keys.pressed(KeyCode::ControlLeft),
        yaw: look.yaw,
    };
}

/// Tracks mouse buttons and the hotbar scroll wheel.
pub fn gather_actions(
    mouse: Res<ButtonInput<MouseButton>>,
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    mut input: ResMut<InputState>,
) {
    if input.ui_captures_mouse {
        input.mining = false;
        input.use_pressed = false;
        wheel.clear();
        return;
    }
    input.mining = mouse.pressed(MouseButton::Left);
    input.use_pressed = mouse.just_pressed(MouseButton::Right);
    let mut scroll = 0.0;
    for event in wheel.read() {
        scroll += event.y;
    }
    input.hotbar_scroll = scroll;
}

/// Applies the local player's body state to the camera transform.
pub fn apply_camera(
    player: Single<(&crate::player::LocalPlayer, &Look)>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
) {
    let (player, look) = *player;
    let body = player.body;
    let eye = body.position + Vec3::Y * 1.62;
    camera.translation = eye;
    camera.rotation = Quat::from_euler(EulerRot::YXZ, look.yaw, look.pitch, 0.0);
}

/// Registers first-person input systems.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>().add_systems(Startup, grab_cursor).add_systems(
            Update,
            (toggle_cursor_lock, mouse_look, gather_movement, gather_actions, apply_camera).chain(),
        );
    }
}
