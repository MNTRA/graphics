use bevy_ecs::{
    event::Events,
    prelude::World,
    schedule::{Schedule, StageLabel, SystemStage},
};
use ::tracing::Level;

pub type EcsPluginCallback = fn(&mut World, &mut Schedule) -> ();
pub trait EcsPlugin: std::fmt::Debug {
    fn build(world: &mut World, schedule: &mut Schedule);
}

#[derive(StageLabel, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum CoreStages {
    EventUpdate,
    PreUpdate,
    Update,
    PostUpdate,
    Layout,
    Render,
    Present,
    Last,
}
pub struct EcsContext<D> {
    pub data: D,
    pub world: World,
    pub schedule: Schedule,
}

impl<D> EcsContext<D> {
    pub fn new(data: D) -> Self {
        tracing::debug_span!("EcsContext::new");
        let schedule = Schedule::default()
            .with_stage(CoreStages::EventUpdate, SystemStage::parallel())
            .with_stage_after(
                CoreStages::EventUpdate,
                CoreStages::PreUpdate,
                SystemStage::parallel(),
            )
            .with_stage_after(
                CoreStages::PreUpdate,
                CoreStages::Update,
                SystemStage::parallel(),
            )
            .with_stage_after(
                CoreStages::Update,
                CoreStages::PostUpdate,
                SystemStage::parallel(),
            )
            .with_stage_after(
                CoreStages::PostUpdate,
                CoreStages::Layout,
                SystemStage::parallel(),
            )
            .with_stage_after(
                CoreStages::Layout,
                CoreStages::Render,
                SystemStage::parallel(),
            )
            .with_stage_after(
                CoreStages::Render,
                CoreStages::Present,
                SystemStage::parallel(),
            )
            .with_stage_after(
                CoreStages::Present,
                CoreStages::Last,
                SystemStage::parallel(),
            );
        Self {
            data,
            world: World::default(),
            schedule,
        }
    }
    pub fn post_event<T>(&mut self, event: T)
    where
        T: Event,
    {
        tracing::event!(Level::DEBUG, event = format!("{:?}", event));
        if !self.world.contains_resource::<Events<T>>() {
            self.world.init_resource::<Events<T>>();
            self.schedule
                .add_system_to_stage(CoreStages::PreUpdate, Events::<T>::update_system);
        }
        let mut events = self
            .world
            .get_resource_mut::<Events<T>>()
            .expect("No Events<T> found");
        events.send(event);
    }

    pub fn add_plugin<P: EcsPlugin>(&mut self) {
        tracing::trace_span!("add_plugin");
        P::build(&mut self.world, &mut self.schedule);
    }
}

pub trait Event: std::fmt::Debug + Send + Sync + 'static {}
macro_rules! impl_event {
    ($($TYPE:ty),*) => {
        $(
            impl crate::Event for $TYPE {}
        )*
    }
}
impl_event!(
    String,
    &'static str,
    std::sync::Arc<str>,
    std::borrow::Cow<'static, str>,
    u8,
    i8,
    u16,
    i16,
    u32,
    i32,
    u64,
    i64,
    u128,
    i128,
    usize,
    isize
);

pub mod tracing {
    use std::panic;
    pub use tracing::*;
    use tracing_chrome::{ChromeLayerBuilder, FlushGuard};
    pub use tracing_error;
    pub use tracing_subscriber::prelude::*;

    pub trait TracingGuard {}
    impl TracingGuard for FlushGuard {}

    pub fn setup_tracing() -> impl TracingGuard {
        let old_handler = panic::take_hook();
        panic::set_hook(Box::new(move |infos| {
            println!("{}", tracing_error::SpanTrace::capture());
            old_handler(infos);
        }));
        let (chrome_layer, guard) = ChromeLayerBuilder::new().file("./chrome-trace").build();
        tracing_subscriber::registry().with(chrome_layer).init();
        guard
    }
}
