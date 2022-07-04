pub mod colors;
mod drivers;

use bevy_ecs::prelude::{Component, Entity};
use colors::Colour;
use derive_deref::{Deref, DerefMut};
use drivers::SkiaDriver;
use raw_window_handle::HasRawWindowHandle;
use utilities::EcsPlugin;

use id_tree::Tree;

pub trait WindowDrawTarget: HasRawWindowHandle + Send + Sync + 'static {
    fn get_draw_bounds(&self) -> (u32, u32);
}

#[derive(Debug)]
pub struct RenderingPlugin;
impl EcsPlugin for RenderingPlugin {
    fn build(_: &mut bevy_ecs::prelude::World, _: &mut bevy_ecs::schedule::Schedule) {}
}

pub trait Painter {
    fn clear(&mut self, colour: impl Into<Colour>);
}

#[derive(Component, Deref, DerefMut)]
pub struct Surface(SkiaDriver);

// TODO: Is this really safe to do?
unsafe impl Send for Surface {}
unsafe impl Sync for Surface {}

impl Surface {
    #[inline(always)]
    pub fn new_cpu(target: &impl WindowDrawTarget) -> anyhow::Result<Self> {
        Ok(Surface(SkiaDriver::new_cpu(target)?))
    }

    #[inline(always)]
    pub fn present_surface(&mut self, target: &impl WindowDrawTarget) -> anyhow::Result<()> {
        self.0.present_surface(target)
    }

    // ========================================================

    pub fn clone(&self) {
        panic!("Clone is not implimented for thread safety")
    }
}

#[derive(Component)]
pub struct RenderElementTree(Tree<Entity>);
