use bevy_ecs::prelude::{Bundle, Component, Entity};
use derive_deref::{Deref, DerefMut};
use raw_window_handle::HasRawWindowHandle;
use std::sync::atomic::AtomicU32;
use tao::{
    event_loop::EventLoopWindowTarget,
    window::{Window as TaoWindow, WindowBuilder as TaoWindowBuilder, WindowId as TaoWindowId},
};

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

    // fn paint(&mut self, painter: impl Painter) {}
}

#[derive(Component, Deref, DerefMut)]
#[repr(transparent)]
pub struct OsWindow(TaoWindow);

#[derive(Component, Deref, DerefMut)]
#[repr(transparent)]
pub struct RootEntitiy(Entity);

#[derive(Component, Deref, DerefMut)]
#[repr(transparent)]
pub struct DynWindow(Box<dyn Window>);

#[derive(Component)]
#[repr(transparent)]
pub struct Marker;

#[derive(Bundle)]
pub struct WindowBundle {
    _m: Marker,
    id: WindowId,
    window: DynWindow,
    os_window: OsWindow,
    raw_id: TaoWindowIdWapper,
    root: RootEntitiy,
    // surface: Surface,
}

impl WindowBundle {
    pub fn new_from_boxed_window(
        window: Box<dyn Window>,
        id: WindowId,
        event_loop: &EventLoopWindowTarget<()>,
        root: Entity,
    ) -> anyhow::Result<Self> {
        let mut os_window = OsWindow(
            TaoWindowBuilder::new()
                .with_title("Untitled Window")
                .build(event_loop)?,
        );
        window.on_create(WindowContext {
            os_window: &mut os_window,
        });
        let raw_id = TaoWindowIdWapper(os_window.id());
        Ok(Self {
            _m: Marker,
            id,
            window: DynWindow(window),
            os_window,
            raw_id,
            root: RootEntitiy(root),
        })
    }
}

pub struct WindowContext<'a> {
    pub(crate) os_window: &'a mut OsWindow,
}

impl WindowContext<'_> {
    pub fn set_window_title(&mut self, title: impl AsRef<str>) {
        self.os_window.set_title(title.as_ref());
    }
}

unsafe impl HasRawWindowHandle for OsWindow {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.0.raw_window_handle()
    }
}
