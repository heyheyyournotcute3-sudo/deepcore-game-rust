//! The fixed-timestep simulation: tick accounting and (from M2) mining validation, machines,
//! power and hazards. Everything here runs on the server only.

use bevy::prelude::*;

use crate::{Content, RunLimit};

/// Simulation statistics, logged periodically.
#[derive(Resource, Default, Debug)]
pub struct SimStats {
    /// Ticks simulated since startup.
    pub ticks: u64,
    /// Ticks accumulated in the current one-second report window.
    pub window_ticks: u32,
    /// Seconds elapsed in the current report window.
    pub window_seconds: f32,
    /// Longest single tick observed, in milliseconds.
    pub worst_tick_ms: f32,
}

/// Registers the server simulation.
pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimStats>()
            .add_systems(FixedUpdate, (count_tick, enforce_run_limit).chain());
    }
}

/// Counts ticks and reports the effective tick rate once per second.
fn count_tick(
    mut stats: ResMut<SimStats>,
    mut limit: ResMut<RunLimit>,
    time: Res<Time<Fixed>>,
    content: Res<Content>,
) {
    stats.ticks += 1;
    stats.window_ticks += 1;
    let dt = time.delta_secs();
    stats.window_seconds += dt;
    stats.worst_tick_ms = stats.worst_tick_ms.max(dt * 1000.0);
    limit.elapsed += dt;

    if stats.window_seconds >= 1.0 {
        let rate = f64::from(stats.window_ticks as f32 / stats.window_seconds);
        debug!(
            "tick {rate:.1}/s, {} blocks / {} recipes loaded",
            content.registry.blocks.len(),
            content.registry.recipes.len()
        );
        stats.window_ticks = 0;
        stats.window_seconds = 0.0;
    }
}

/// Stops the process after `--exit-after` seconds, so smoke tests terminate on their own.
fn enforce_run_limit(limit: Res<RunLimit>, mut exit: MessageWriter<AppExit>, stats: Res<SimStats>) {
    if limit.seconds > 0 && limit.elapsed >= limit.seconds as f32 {
        info!("run limit reached after {} ticks; shutting down cleanly", stats.ticks);
        exit.write(AppExit::Success);
    }
}
