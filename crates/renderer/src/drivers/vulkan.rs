use anyhow::Error;
use ash::{
    extensions::khr::{self, Swapchain},
    vk::{
        self, ColorSpaceKHR, Extent2D, Format, Handle, ImageUsageFlags, PhysicalDevice,
        PresentModeKHR, SurfaceKHR, SwapchainCreateInfoKHR, SwapchainKHR,
    },
    Device, Entry, Instance,
};
use raw_window_handle::HasRawWindowHandle;
use skia::{
    gpu::{self, DirectContext},
    Budgeted, ImageInfo,
};
use std::{ffi::CString, os::raw::c_char, ptr};

use super::{Driver, DriverKind};

fn str_iter_collect(strs: impl Iterator<Item = &'static str>) -> Vec<*const c_char> {
    strs.map(|s: &'static str| s.as_ptr() as *const c_char)
        .collect::<Vec<_>>()
}

struct Vulkan {
    api: AshApi,
    context: DirectContext,
}

impl Vulkan {
    fn new() -> anyhow::Result<Self> {
        let api = AshApi::new()?;
        let context = Self::create_backend_context(&api);
        Ok(Self { api, context })
    }
    fn create_backend_context(api: &AshApi) -> DirectContext {
        let get_proc = |of| unsafe {
            match api.get_proc(of) {
                Some(f) => f as _,
                None => {
                    println!("resolve of {} failed", of.name().to_str().unwrap());
                    ptr::null()
                }
            }
        };
        let backend_context = unsafe {
            let exts: Vec<&str> = AshApi::get_required_windowing_extensions().collect();
            gpu::vk::BackendContext::new_with_extensions(
                api.instance.handle().as_raw() as _,
                api.physical_device.as_raw() as _,
                api.device.handle().as_raw() as _,
                (api.queue_and_index.0.as_raw() as _, api.queue_and_index.1),
                &get_proc,
                &exts,
                &[],
            )
        };
        DirectContext::new_vulkan(&backend_context, None).unwrap()
    }
}

impl Driver for Vulkan {
    const DRIVER: DriverKind = DriverKind::Vulkan;
    fn create_surface(
        &mut self,
        window: impl HasRawWindowHandle,
        size: impl Into<skia::ISize>,
    ) -> anyhow::Result<skia::Surface> {
        let size = size.into();
        let surface = unsafe {
            ash_window::create_surface(&self.api.entry, &self.api.instance, &window, None)?
        };
        let surface_fn = ash::extensions::khr::Surface::new(&self.api.entry, &self.api.instance);
        let swapchain_handle = self
            .api
            .create_swapchain(surface, (size.width as u32, size.height as u32))?;

        let image_info = ImageInfo::new_n32_premul(size, None);

        Ok(skia::Surface::new_render_target(
            &mut self.context,
            Budgeted::Yes,
            &image_info,
            None,
            gpu::SurfaceOrigin::TopLeft,
            None,
            false,
        )
        .unwrap())
    }
    fn present_surface(&mut self) {
        todo!()
    }
}

pub struct AshApi {
    pub(crate) queue_and_index: (vk::Queue, usize),
    pub(crate) device: ash::Device,
    pub(crate) physical_device: vk::PhysicalDevice,
    pub(crate) instance: Instance,
    pub(crate) entry: Entry,
}

impl Drop for AshApi {
    fn drop(&mut self) {
        // SAFETY: Explicit drop ordering
        drop(self.queue_and_index);
        drop(self.device);
        drop(self.physical_device);
        drop(self.instance);
        drop(self.entry);
    }
}

impl AshApi {
    pub fn new() -> anyhow::Result<Self> {
        let entry = unsafe { Entry::load()? };
        let instance = Self::create_instance(&entry)?;
        let (physical_device, queue_family_index) = Self::create_physical_device(&instance)?;
        let device = Self::create_logical_device(instance, physical_device, queue_family_index)?;

        let physical_device_properties =
            unsafe { instance.get_physical_device_properties(physical_device) };
        let device_name = unsafe {
            std::str::from_utf8_unchecked(std::mem::transmute(
                physical_device_properties.device_name,
            ))
        };
        println!("Device Name: {}", device_name);

        let queue_index: usize = 0;
        let queue: vk::Queue =
            unsafe { device.get_device_queue(queue_family_index as _, queue_index as _) };

        Ok(Self {
            queue_and_index: (queue, queue_index),
            device,
            physical_device,
            instance,
            entry,
        })
    }

    pub fn create_logical_device(
        instance: Instance,
        physical_device: PhysicalDevice,
        queue_family_index: usize,
    ) -> anyhow::Result<Device> {
        let extension_names =
            Self::get_required_windowing_extensions().chain(["VK_KHR_swapchain"].into_iter());
            
        let features = vk::PhysicalDeviceFeatures::default();
        let priorities = [1.0];
        let queue_info = [vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index as _)
            .queue_priorities(&priorities)
            .build()];
        let device_extension_names_raw = [];
        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_info)
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features);

        unsafe { Ok(instance.create_device(physical_device, &device_create_info, None)?) }
    }

    pub fn create_physical_device(instance: &Instance) -> anyhow::Result<(PhysicalDevice, usize)> {
        let physical_devices = unsafe { instance.enumerate_physical_devices() }
            .expect("Failed to enumerate Vulkan physical devices.");
        match physical_devices
            .iter()
            .map(|physical_device: &PhysicalDevice| {
                // TODO: look at the Flutter examples to see how to properly choose a valid device
                unsafe { instance.get_physical_device_queue_family_properties(*physical_device) }
                    .iter()
                    .enumerate()
                    .find_map(|(index, info)| {
                        let supports_graphic = info.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                        if supports_graphic {
                            Some((*physical_device, index))
                        } else {
                            None
                        }
                    })
            })
            .find_map(|v| v)
        {
            Some(out) => Ok(out),
            None => Err(Error::msg("Unable to find a suitable GPU")),
        }
    }

    pub fn create_instance(entry: &Entry) -> anyhow::Result<Instance> {
        let layer_names = [
            "VK_LAYER_LUNARG_standard_validation",
            "VK_LAYER_KHRONOS_validation",
        ]
        .into_iter();
        let app_desc = vk::ApplicationInfo::builder()
            .api_version(vk::make_api_version(0, 1, 0, 0))
            .application_name(CString::new("application_name").unwrap().as_c_str())
            .build();
        let instance_desc = vk::InstanceCreateInfo::builder()
            .application_info(&app_desc)
            .enabled_layer_names(&str_iter_collect(layer_names))
            .enabled_extension_names(&[]);
        unsafe { Ok(entry.create_instance(&instance_desc, None)?) }
    }

    pub fn create_swapchain(
        &self,
        surface: SurfaceKHR,
        (width, height): (u32, u32),
    ) -> anyhow::Result<SwapchainKHR> {
        let swapchain = Swapchain::new(&self.instance, &self.device);
        let create_info = SwapchainCreateInfoKHR::builder()
            .clipped(true)
            .surface(surface)
            .min_image_count(1)
            .image_format(Format::B8G8R8A8_SRGB)
            .image_color_space(ColorSpaceKHR::SRGB_NONLINEAR)
            .image_array_layers(1)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
            .present_mode(PresentModeKHR::FIFO)
            .image_extent(Extent2D { width, height });

        unsafe { Ok(swapchain.create_swapchain(&create_info, None)?) }
    }

    pub fn vulkan_version(&self) -> Option<(usize, usize, usize)> {
        let detected_version = self.entry.try_enumerate_instance_version().unwrap_or(None);
        detected_version.map(|ver| {
            (
                vk::api_version_major(ver).try_into().unwrap(),
                vk::api_version_minor(ver).try_into().unwrap(),
                vk::api_version_patch(ver).try_into().unwrap(),
            )
        })
    }
    pub unsafe fn get_proc(&self, of: gpu::vk::GetProcOf) -> Option<unsafe extern "system" fn()> {
        match of {
            gpu::vk::GetProcOf::Instance(instance, name) => {
                let ash_instance = vk::Instance::from_raw(instance as _);
                self.entry.get_instance_proc_addr(ash_instance, name)
            }
            gpu::vk::GetProcOf::Device(device, name) => {
                let ash_device = vk::Device::from_raw(device as _);
                self.instance.get_device_proc_addr(ash_device, name)
            }
        }
    }
    pub fn get_required_windowing_extensions() -> impl Iterator<Item = &'static str> {
        #[cfg(target_os = "windows")]
        {
            const WINDOWS_EXTS: [&'static str; 2] = [
                khr::Surface::name().to_str().unwrap(),
                khr::Win32Surface::name().to_str().unwrap(),
            ];
            WINDOWS_EXTS.into_iter()
        }
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            const WAYLAND_EXTS: [*const c_char; 2] = [
                khr::Surface::name().as_ptr(),
                khr::WaylandSurface::name().as_ptr(),
            ];
            &WAYLAND_EXTS
        }
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            const XLIB_EXTS: [*const c_char; 2] = [
                khr::Surface::name().as_ptr(),
                khr::XlibSurface::name().as_ptr(),
            ];
            &XLIB_EXTS
        }
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            const XCB_EXTS: [*const c_char; 2] = [
                khr::Surface::name().as_ptr(),
                khr::XcbSurface::name().as_ptr(),
            ];
            &XCB_EXTS
        }
        #[cfg(any(target_os = "android"))]
        {
            const ANDROID_EXTS: [*const c_char; 2] = [
                khr::Surface::name().as_ptr(),
                khr::AndroidSurface::name().as_ptr(),
            ];
            &ANDROID_EXTS
        }
        #[cfg(any(target_os = "macos"))]
        {
            const MACOS_EXTS: [*const c_char; 2] = [
                khr::Surface::name().as_ptr(),
                ext::MetalSurface::name().as_ptr(),
            ];
            &MACOS_EXTS
        }
        #[cfg(any(target_os = "ios"))]
        {
            const IOS_EXTS: [*const c_char; 2] = [
                khr::Surface::name().as_ptr(),
                ext::MetalSurface::name().as_ptr(),
            ];
            &IOS_EXTS
        }
    }
}
