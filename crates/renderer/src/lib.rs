mod drivers;

use bevy_ecs::{prelude::*, system::assert_is_system};
use derive_deref::{Deref, DerefMut};
use drivers::Driver;
use raw_window_handle::HasRawWindowHandle;
use utilities::EcsPlugin;

pub struct RenderObject {}

pub trait WindowDrawTarget: HasRawWindowHandle + Send + Sync + 'static {
    fn get_draw_bounds(&self) -> (u32, u32);
}

pub struct RenderingPlugin;
impl EcsPlugin for RenderingPlugin {
    fn build(world: &mut bevy_ecs::prelude::World, schedule: &mut bevy_ecs::schedule::Schedule) {
        todo!()
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Surface(skia::Surface);

// TODO: Is this really safe to do?
unsafe impl Send for Surface {}
unsafe impl Sync for Surface {}

impl Surface {
    pub fn new_cpu(target: &impl WindowDrawTarget) -> anyhow::Result<Self> {
        let mut cpu_driver = drivers::CpuSurface::default();
        let bounds = target.get_draw_bounds();
        let mut surface = cpu_driver.create_surface(target, (bounds.0 as i32, bounds.1 as i32))?;
        surface.canvas().clear(skia::Color::BLACK);
        Ok(Surface(surface))
    }

    pub fn present_surface(&mut self, target: &impl WindowDrawTarget) -> anyhow::Result<()> {
        let mut cpu_driver = drivers::CpuSurface::default();
        cpu_driver.present_surface(target, &mut self.0)
    }

    // ========================================================

    pub fn clone(&self) {
        panic!("Clone is not implimented for thread safety")
    }
}

fn render_system(render_targets: Query<()>) {
    assert_is_system(render_system);
}
