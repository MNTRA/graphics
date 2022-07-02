pub mod cpu;
// mod vulkan;
// mod vulkanold;

use skia::ISize;
use crate::WindowDrawTarget;

pub type CpuSurface = cpu::Cpu;

pub trait Driver {
    fn create_surface(
        &mut self,
        window: &impl WindowDrawTarget,
        dimensions: impl Into<ISize>,
    ) -> anyhow::Result<skia::Surface>;
    fn present_surface(
        &mut self,
        window: &impl WindowDrawTarget,
        surface: &mut skia::Surface,
    ) -> anyhow::Result<()>;
}
