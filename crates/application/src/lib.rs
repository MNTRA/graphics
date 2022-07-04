use std::ops::{Deref, DerefMut};

use bevy_ecs::{event::Events, schedule::SystemStage};
use events::{shutdown_system, Shutdown};
use utilities::{tracing, CoreStages, EcsContext, EcsPlugin, Event};
use windowing::{
    events::Create,
    window::{Window, WindowId},
    CallbackTranslator, ControlFlow, EventLoop,
};

pub struct ApplicationContext<'a, T>(&'a mut EcsContext<T>);

impl<'a, T> ApplicationContext<'a, T> {
    pub(crate) fn new(ctx: &'a mut EcsContext<T>) -> Self {
        Self(ctx)
    }
    pub fn create_window(&mut self, window: impl Window) -> WindowId {
        let window_id = WindowId::new();
        self.0.post_event(Create {
            window: Box::new(window),
            window_id,
        });
        window_id
    }
    pub fn post_event<E>(&mut self, event: E)
    where
        E: Event,
    {
        self.0.post_event(event);
    }
    pub fn add_plugin<P: EcsPlugin>(&mut self) {
        self.0.add_plugin::<P>()
    }
}

impl<'a, T> Deref for ApplicationContext<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0.data
    }
}
impl<'a, T> DerefMut for ApplicationContext<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.data
    }
}

/// Runs the application
pub fn entry_point<A, D>() -> !
where
    A: Application<Data = D> + 'static,
    D: Default + 'static,
{
    tracing::debug_span!("application::entry_point");

    type Callback<A> = fn(ApplicationContext<'_, A>);
    struct CallbackTranslatorImpl<A> {
        pub event_loop_started: Callback<A>,
        pub event_loop_closed: Callback<A>,
    }
    impl<A> CallbackTranslator<A> for CallbackTranslatorImpl<A> {
        fn event_loop_started(&mut self, ctx: &mut EcsContext<A>) {
            (self.event_loop_started)(ApplicationContext::new(ctx))
        }
        fn event_loop_closed(&mut self, ctx: &mut EcsContext<A>) {
            (self.event_loop_closed)(ApplicationContext::new(ctx))
        }
    }
    EventLoop::new().run(
        A::Data::default(),
        CallbackTranslatorImpl {
            event_loop_started: A::initalize,
            event_loop_closed: A::shutdown,
        },
    );
}

pub trait Application {
    type Data: Default;
    fn initalize(_: ApplicationContext<'_, Self::Data>) {}
    fn shutdown(_: ApplicationContext<'_, Self::Data>) {}
}

#[macro_export]
macro_rules! run_app {
    ($APP_TYPE:ty) => {
        fn main() -> ! {
            let _guard = ::utilities::tracing::setup_tracing();
            ::application::entry_point::<$APP_TYPE, <$APP_TYPE as Application>::Data>();
        }
    };
}

#[derive(Debug)]
pub struct ApplicationPlugin;
impl EcsPlugin for ApplicationPlugin {
    fn build(world: &mut bevy_ecs::prelude::World, schedule: &mut bevy_ecs::schedule::Schedule) {
        world.init_resource::<Events<Shutdown>>();
        world.init_resource::<ControlFlow>();
        schedule.stage(CoreStages::EventUpdate, |stage: &mut SystemStage| {
            stage.add_system(Events::<Shutdown>::update_system);
            stage
        });
        schedule.stage(CoreStages::PreUpdate, |stage: &mut SystemStage| {
            stage.add_system(shutdown_system);
            stage
        });
    }
}

mod events {
    use bevy_ecs::{
        prelude::{Component, EventReader, ResMut},
        system::assert_is_system,
    };
    use windowing::ControlFlow;

    #[derive(Component)]
    pub struct Shutdown;

    pub fn shutdown_system(
        mut control_flow: ResMut<ControlFlow>,
        shutdown_events: EventReader<Shutdown>,
    ) {
        assert_is_system(shutdown_system);
        if !shutdown_events.is_empty() {
            *control_flow = ControlFlow::ExitWithCode(0);
        }
    }
}
