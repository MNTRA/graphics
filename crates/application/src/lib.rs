use std::ops::{Deref, DerefMut};

use utilities::{EcsContext, EcsPlugin, Event};
use windowing::{
    events::Create,
    window::{Window, WindowId},
    CallbackTranslator, EventLoop,
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
pub fn run_application<A, D>() -> !
where
    A: Application<Data = D> + 'static,
    D: Default + 'static,
{
    type Callback<A> = fn(ApplicationContext<'_, A>);
    struct CallbacksTranslator<A> {
        pub on_initalize: Callback<A>,
        pub on_shutdown: Callback<A>,
    }
    impl<A> CallbackTranslator<A> for CallbacksTranslator<A> {
        fn event_loop_started(&mut self, ctx: &mut EcsContext<A>) {
            (self.on_initalize)(ApplicationContext::new(ctx))
        }
        fn event_loop_closed(&mut self, ctx: &mut EcsContext<A>) {
            (self.on_shutdown)(ApplicationContext::new(ctx))
        }
    }
    EventLoop::new().run(
        A::Data::default(),
        CallbacksTranslator {
            on_initalize: A::initalize,
            on_shutdown: A::shutdown,
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
            ::application::run_application::<$APP_TYPE, <$APP_TYPE as Application>::Data>();
        }
    };
}
