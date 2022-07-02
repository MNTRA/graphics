mod platform;
use crate::WindowDrawTarget;

use super::Driver;
use anyhow::Error;
use skia::{ISize, Surface};

use platform::{BackBuffer, ConcretePlatformImpl, PlatformApi};

pub struct Cpu(pub(crate) skia::Surface);

impl Driver for Cpu {
    fn create_surface(
        &mut self,
        _: &impl WindowDrawTarget,
        size: impl Into<ISize>,
    ) -> anyhow::Result<skia::Surface> {
        Surface::new_raster_n32_premul(size.into())
            .map_or_else(|| Err(Error::msg("Unable to create Surface")), |e| Ok(e))
    }
    fn present_surface(
        &mut self,
        window: &impl WindowDrawTarget,
        surface: &mut skia::Surface,
    ) -> anyhow::Result<()> {
        let back_buffer = <ConcretePlatformImpl as PlatformApi>::BackBuffer::new(surface)?;
        <ConcretePlatformImpl as PlatformApi>::present_backbuffer(window, back_buffer)?;
        Ok(())
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self(skia::Surface::new_null((1, 1)).unwrap())
    }
}
