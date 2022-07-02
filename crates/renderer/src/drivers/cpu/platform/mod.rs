#[cfg(target_os = "windows")]
pub mod win32;
#[cfg(target_os = "windows")]
pub use win32::Win32PlatformImpl as ConcretePlatformImpl;

use crate::WindowDrawTarget;

pub trait BackBuffer {
    fn new(surface: &mut skia::Surface) -> anyhow::Result<Self>
    where
        Self: Sized;
}

pub trait PlatformApi {
    type BackBuffer: BackBuffer;
    fn present_backbuffer(
        window: &impl WindowDrawTarget,
        back_buffer: Self::BackBuffer,
    ) -> anyhow::Result<()>;
}
