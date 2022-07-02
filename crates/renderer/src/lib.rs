pub mod colors;
mod drivers;

use bevy_ecs::{
    prelude::Component,
    schedule::{ParallelSystemDescriptorCoercion, SystemStage},
};
use colors::Colour;
use derive_deref::{Deref, DerefMut};
use drivers::SkiaDriver;
use raw_window_handle::HasRawWindowHandle;
use utilities::{CoreStages, EcsPlugin};

pub struct RenderObject {}

pub trait WindowDrawTarget: HasRawWindowHandle + Send + Sync + 'static {
    fn get_draw_bounds(&self) -> (u32, u32);
}

pub struct RenderingPlugin;
impl EcsPlugin for RenderingPlugin {
    fn build(_: &mut bevy_ecs::prelude::World, schedule: &mut bevy_ecs::schedule::Schedule) {
        use windowing_compat::*;
        schedule.stage(CoreStages::Update, |stage: &mut SystemStage| {
            const CREATE_SURFACE_SYS_LABEL: &str = "create-surface";
            stage.add_system(create_surface_for_window_system.label(CREATE_SURFACE_SYS_LABEL));
            stage.add_system(present_surface_to_window.after(CREATE_SURFACE_SYS_LABEL));
            stage
        });
    }
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

pub mod windowing_compat {
    use bevy_ecs::{
        prelude::{Added, Commands, Entity, EventReader, EventWriter, Query, With},
        system::assert_is_system,
    };
    use windowing::{
        events::{Repaint, Resize},
        window::{self, OsWindow, WindowId},
    };

    use crate::{Surface, WindowDrawTarget};

    impl WindowDrawTarget for OsWindow {
        fn get_draw_bounds(&self) -> (u32, u32) {
            self.inner_size().into()
        }
    }

    pub(crate) fn create_surface_for_window_system(
        mut commands: Commands,
        new_windows: Query<Entity, Added<window::Marker>>,
        mut events: EventReader<Resize>,
        mut redraw_events: EventWriter<Repaint>,
        mut window_query: Query<(&WindowId, &mut OsWindow), With<window::Marker>>,
    ) {
        assert_is_system(create_surface_for_window_system);
        let windows_to_create_surfaces_for = events
            .iter()
            .map(
                |Resize {
                     window_id: _,
                     entity,
                 }| *entity,
            )
            .chain(new_windows.iter());

        for entity in windows_to_create_surfaces_for {
            let (window_id, os_window) = match window_query.get_mut(entity) {
                Ok((window_id, os_window)) => (*window_id, os_window),
                Err(err) => panic!("{}", err),
            };

            // if the bounds are 0 then keep the old surface as skia will panic if
            // a surface is created with 0 bounds
            let (width, height) = os_window.get_draw_bounds();
            if width <= 0 || height <= 0 {
                continue;
            }

            let surface = match Surface::new_cpu(&*os_window) {
                Ok(new_surface) => new_surface,
                Err(err) => panic!("{}", err),
            };
            commands.entity(entity).insert(surface);
            redraw_events.send(Repaint { window_id, entity });
        }
    }

    pub fn present_surface_to_window(
        mut events: EventReader<Repaint>,
        mut windows: Query<(&mut OsWindow, &mut Surface), With<window::Marker>>,
    ) {
        for Repaint {
            window_id: _,
            entity,
        } in events.iter()
        {
            let (os_window, mut surface) = match windows.get_mut(*entity) {
                Ok((os_window, surface)) => (os_window, surface),
                Err(_) => continue,
            };
            match surface.present_surface(&*os_window) {
                Ok(_) => {}
                Err(err) => panic!("{}", err),
            };
        }
    }
}
