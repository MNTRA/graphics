pub mod cpu;
// mod vulkan;
// mod vulkanold;

use crate::WindowDrawTarget;
use skia::ISize;

use self::cpu::Cpu;

pub enum SkiaDriver {
    Cpu(cpu::Cpu),
}

impl SkiaDriver {
    #[inline(always)]
    pub fn new_cpu(target: &impl WindowDrawTarget) -> anyhow::Result<Self> {
        let mut cpu_driver = cpu::Cpu::default();
        let bounds = target.get_draw_bounds();
        let mut surface = cpu_driver.create_surface(target, (bounds.0 as i32, bounds.1 as i32))?;
        surface.canvas().clear(skia::Color::CYAN);
        Ok(SkiaDriver::Cpu(Cpu(surface)))
    }

    #[inline(always)]
    pub fn present_surface(&mut self, target: &impl WindowDrawTarget) -> anyhow::Result<()> {
        match self {
            SkiaDriver::Cpu(cpu) => cpu.present_surface(target, &mut cpu.0.clone()),
        }
    }
}

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
