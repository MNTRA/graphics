use bevy_ecs::{
    event::Events,
    prelude::{Bundle, Component, Entity, Mut, World},
};
use derive_deref::{Deref, DerefMut};
use raw_window_handle::HasRawWindowHandle;
use renderer::WindowDrawTarget;
use std::sync::{atomic::AtomicU32, Mutex};
use tao::{
    event_loop::EventLoopWindowTarget,
    window::{Window as TaoWindow, WindowBuilder as TaoWindowBuilder, WindowId as TaoWindowId},
};

use crate::ShutdownEventLoop;

static CURRENT_WINDOW_ID: AtomicU32 = AtomicU32::new(0);
#[derive(Component, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct WindowId(u32);
impl WindowId {
    pub fn new() -> WindowId {
        WindowId(CURRENT_WINDOW_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

#[derive(Component, Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct TaoWindowIdWapper(pub(crate) TaoWindowId);

pub trait Window: std::fmt::Debug + Send + Sync + 'static {
    fn on_create(&self, _: WindowContext<'_>) {}
    fn close_requested(&mut self, _: WindowContext<'_>) -> bool {
        true
    }
    fn on_destroyed(&mut self, mut ctx: WindowContext<'_>) {
        ctx.post_shutdown_message();
    }
    // fn paint(&mut self, painter: impl Painter) {}
}

#[derive(Default, Debug)]
#[repr(transparent)]
pub struct WindowCallbacksManager(Vec<(Entity, WindowCallbacks)>);
impl WindowCallbacksManager {
    pub fn insert(&mut self, entity: Entity, callbacks: WindowCallbacks) {
        if self.contains(entity) {
            panic!("Window Callbacks already exist for this entity")
        }
        self.0.push((entity, callbacks));
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.0.iter().find(|(e, _)| *e == entity).is_some()
    }

    pub fn remove(&mut self, entity: Entity) {
        if let Some(pos) = self.0.iter().position(|(e, _)| *e == entity) {
            self.0.remove(pos);
        }
    }

    pub fn get(&self, entity: Entity) -> Option<&WindowCallbacks> {
        self.0.iter().find_map(
            |(e, dyn_window)| {
                if *e == entity {
                    Some(dyn_window)
                } else {
                    None
                }
            },
        )
    }
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut WindowCallbacks> {
        self.0.iter_mut().find_map(
            |(e, dyn_window)| {
                if *e == entity {
                    Some(dyn_window)
                } else {
                    None
                }
            },
        )
    }
}

#[derive(Debug, Deref, DerefMut)]
#[repr(transparent)]
pub struct WindowCallbacks(pub(crate) Box<dyn Window>);

#[derive(Component, Deref, DerefMut)]
#[repr(transparent)]
pub struct OsWindow(TaoWindow);

impl WindowDrawTarget for OsWindow {
    fn get_draw_bounds(&self) -> (u32, u32) {
        self.inner_size().into()
    }
}

#[derive(Component, Deref, DerefMut)]
#[repr(transparent)]
pub struct RootEntitiy(Entity);

#[derive(Component)]
#[repr(transparent)]
pub struct Marker;

#[derive(Bundle)]
pub struct WindowBundle {
    _m: Marker,
    pub(crate) id: WindowId,
    pub(crate) os_window: OsWindow,
    pub(crate) raw_id: TaoWindowIdWapper,
    pub(crate) root: RootEntitiy,
}

impl WindowBundle {
    pub fn new(
        id: WindowId,
        event_loop: &EventLoopWindowTarget<()>,
        root: Entity,
    ) -> anyhow::Result<Self> {
        let os_window = OsWindow(
            TaoWindowBuilder::new()
                .with_title("Untitled Window")
                .build(event_loop)?,
        );
        let raw_id = TaoWindowIdWapper(os_window.id());
        Ok(Self {
            _m: Marker,
            id,
            os_window,
            raw_id,
            root: RootEntitiy(root),
        })
    }
}

pub struct WindowContext<'w> {
    // pub(crate) os_window: &'a mut OsWindow,
    pub(crate) entity: Entity,
    pub(crate) world: &'w mut World,
}

impl WindowContext<'_> {
    pub fn set_title_text(&mut self, title: impl AsRef<str>) {
        self.get_underlying_os_window_mut()
            .set_title(title.as_ref());
    }

    pub fn post_shutdown_message(&mut self) {
        self.world
            .get_resource_mut::<Events<ShutdownEventLoop>>()
            .unwrap_or_else(|| panic!("No EvenEvents<ShutdownEventLoop> Resource"))
            .send_default()
    }

    // ===========================================================================

    fn get_underlying_os_window(&self) -> &OsWindow {
        self.world
            .get::<OsWindow>(self.entity)
            .expect("no OsWindow on this Enity")
    }
    fn get_underlying_os_window_mut(&mut self) -> Mut<OsWindow> {
        self.world
            .get_mut::<OsWindow>(self.entity)
            .expect("no OsWindow on this Enity")
    }
}

unsafe impl HasRawWindowHandle for OsWindow {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.0.raw_window_handle()
    }
}
