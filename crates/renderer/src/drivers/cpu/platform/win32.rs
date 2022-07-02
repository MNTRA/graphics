use std::mem;
use anyhow::Error;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, Win32Handle};
use windows::Win32::Foundation::{HINSTANCE, HWND};
use windows::Win32::Graphics::Gdi::{
    GetDC, ReleaseDC, StretchDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
};

use super::{BackBuffer, PlatformApi};
use crate::WindowDrawTarget;

pub struct Win32PlatformImpl;
impl PlatformApi for Win32PlatformImpl {
    type BackBuffer = Win32BackBuffer;
    fn present_backbuffer(
        window: &impl WindowDrawTarget,
        back_buffer: Self::BackBuffer,
    ) -> anyhow::Result<()> {
        let (hwnd, _) = get_hwnd_and_instance(&window);
        let (window_width, window_height) = window.get_draw_bounds();

        let mut bitmapinfo = BITMAPINFO::default();
        bitmapinfo.bmiHeader = BITMAPINFOHEADER {
            biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: back_buffer.width,
            biHeight: back_buffer.height,
            biPlanes: 1,
            biBitCount: back_buffer.bytes_per_pixel, //32,
            biCompression: BI_RGB as u32,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        };

        let hdc = unsafe { GetDC(hwnd) };
        let scan_lines = unsafe {
            StretchDIBits(
                hdc,
                0,
                0,
                window_width as i32,
                window_height as i32,
                0,
                0,
                back_buffer.width,
                back_buffer.height,
                back_buffer.data as *const _,
                &bitmapinfo as *const BITMAPINFO,
                DIB_RGB_COLORS,
                SRCCOPY,
            )
        };
        unsafe { ReleaseDC(hwnd, hdc) };
        if scan_lines == 0 {
            return Err(Error::msg("Unable to blit to window"));
        }
        Ok(())
    }
}

pub struct Win32BackBuffer {
    pub width: i32,
    pub height: i32,
    pub bytes_per_pixel: u16,
    pub data: *const u8,
}

impl BackBuffer for Win32BackBuffer {
    fn new(surface: &mut skia::Surface) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let width = surface.width();
        let height = surface.height();
        let tlp = surface.canvas().access_top_layer_pixels().unwrap();

        Ok(Win32BackBuffer {
            width,
            height,
            bytes_per_pixel: 32,
            data: tlp.pixels.as_ptr() as *const _,
        })
    }
}

fn get_hwnd_and_instance(handle: &impl HasRawWindowHandle) -> (HWND, HINSTANCE) {
    if let RawWindowHandle::Win32(Win32Handle {
        hinstance, hwnd, ..
    }) = handle.raw_window_handle()
    {
        (HWND(hwnd as isize), HINSTANCE(hinstance as isize))
    } else {
        panic!("Called for the wrong platform")
    }
}
