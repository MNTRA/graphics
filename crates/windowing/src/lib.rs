pub mod events;
pub mod window;

use bevy_ecs::{
    event::Events,
    prelude::IntoChainSystem,
    schedule::{ParallelSystemDescriptorCoercion, Schedule, StageLabel, SystemStage},
    world::World,
};
use events::{Event, WindowEvent};
use tao::{
    event::{Event as TaoEvent, StartCause, WindowEvent as TaoWindowEvent},
    event_loop::{ControlFlow, EventLoop as TaoEventLoop},
};
use utilities::EcsPlugin;
use window::{TaoWindowIdWapper, Window, WindowId};

struct WindowingPlugin;
impl EcsPlugin for WindowingPlugin {
    fn build(world: &mut World, schedule: &mut Schedule) {
        world.init_resource::<Events<WindowEvent>>();
        schedule.stage(CoreStages::PreUpdate, |stage: &mut SystemStage| {
            const EVENT_UPDATE_SYS_LABEL: &str = "events-update";
            stage.add_system(Events::<WindowEvent>::update_system.label(EVENT_UPDATE_SYS_LABEL));
            stage.add_system(
                events::window_events_system
                    .chain(events::destroyed_window)
                    .after(EVENT_UPDATE_SYS_LABEL),
            );
            stage
        });
    }
}

#[derive(StageLabel, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum CoreStages {
    PreUpdate,
    Update,
    PostUpdate,
    Layout,
    Render,
}

pub struct ApplicationContext {
    world: World,
    schedule: Schedule,
}

impl ApplicationContext {
    pub(crate) fn new() -> Self {
        let schedule = Schedule::default()
            .with_stage(CoreStages::PreUpdate, SystemStage::parallel())
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
            );

        Self {
            world: World::default(),
            schedule,
        }
    }

    pub fn create_window(&mut self, window: impl Window) -> WindowId {
        let window_id = WindowId::new();
        self.post_event(WindowEvent::Create {
            window: Box::new(window),
            window_id,
        });
        window_id
    }

    pub fn post_event<T: Event>(&mut self, event: T) {
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
        P::build(&mut self.world, &mut self.schedule);
    }
}

pub struct EventLoop {
    event_loop: TaoEventLoop<()>,
}
impl EventLoop {
    pub fn new() -> Self {
        Self {
            event_loop: TaoEventLoop::new(),
        }
    }
    pub fn run(self, mut app: impl Application + 'static) -> ! {
        let mut ctx = ApplicationContext::new();
        self.event_loop.run(move |event, target, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                TaoEvent::NewEvents(StartCause::Init) => {
                    ctx.world.insert_non_send_resource(target.clone());
                    ctx.world.insert_resource(*control_flow);
                    ctx.add_plugin::<WindowingPlugin>();
                    app.initalize(&mut ctx);
                }
                TaoEvent::WindowEvent {
                    event: window_event,
                    window_id,
                    ..
                } => Self::dispatch_window_events(window_event, window_id, &mut ctx),
                TaoEvent::RedrawRequested(raw_id) => {
                    let id = find_window_id_from_raw_id(raw_id, &mut ctx);
                    ctx.post_event(WindowEvent::Repaint(id));
                }
                TaoEvent::MainEventsCleared => {
                    ctx.schedule.run_once(&mut ctx.world);
                    *control_flow = *ctx.world.get_resource::<ControlFlow>().unwrap();
                }
                _ => (),
            }
        })
    }

    fn dispatch_window_events(
        window_event: TaoWindowEvent,
        raw_id: tao::window::WindowId,
        ctx: &mut ApplicationContext,
    ) {
        match window_event {
            TaoWindowEvent::CloseRequested => {
                let id = find_window_id_from_raw_id(raw_id, ctx);
                ctx.post_event(WindowEvent::CloseRequested(id));
            }
            TaoWindowEvent::Destroyed => {
                // TODO: manually emit this event from the window event handler system
            }
            TaoWindowEvent::Resized(ps) => {
                let id = find_window_id_from_raw_id(raw_id, ctx);
                ctx.post_event(WindowEvent::Resize {
                    window_id: id,
                    bounds: ps.into(),
                })
            }
            _ => (),
        }
    }
}

fn find_window_id_from_raw_id(
    raw_id: tao::window::WindowId,
    ctx: &mut ApplicationContext,
) -> WindowId {
    match ctx
        .world
        .query::<(&WindowId, &TaoWindowIdWapper)>()
        .iter(&ctx.world)
        .find(|(_, &tid)| TaoWindowIdWapper(raw_id) == tid)
    {
        Some((&id, _)) => id,
        None => panic!("no window with id: {:?} found", raw_id),
    }
}

/// Runs the application
pub fn run_application(app: impl Application + 'static) -> ! {
    let event_loop = EventLoop::new();
    event_loop.run(app)
}

pub trait Application {
    fn initalize(&mut self, _ctx: &mut ApplicationContext) {}
    fn shutdown(&mut self, _ctx: &mut ApplicationContext) {}
}
