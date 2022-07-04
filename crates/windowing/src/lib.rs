pub mod events;
pub mod window;

use bevy_ecs::{
    event::Events,
    prelude::{Entity, EventReader, IntoExclusiveSystem, ResMut},
    schedule::{Schedule, SystemSet, SystemStage},
    world::World, system::assert_is_system,
};
use events::{
    create_surface_for_window_system, repaint_and_present_windows, CloseRequested,
    CloseRequestedSystemState, Create, CreateWindowSystemState, Destroy, DestroyWindowSystemState,
    Resize,
};
pub use tao::event_loop::ControlFlow;
use tao::{
    event::{Event as TaoEvent, StartCause, WindowEvent as TaoWindowEvent},
    event_loop::EventLoop as TaoEventLoop,
};
use utilities::{tracing, CoreStages, EcsContext, EcsPlugin, Event};
use window::{TaoWindowIdWapper, WindowCallbacksManager, WindowId};

use crate::events::Repaint;

#[derive(Debug)]
pub struct WindowingPlugin;
impl EcsPlugin for WindowingPlugin {
    fn build(world: &mut World, schedule: &mut Schedule) {
        // Resources ========================================================

        world.init_resource::<Events<Create>>();
        world.init_resource::<Events<Resize>>();
        world.init_resource::<Events<Repaint>>();
        world.init_resource::<Events<Destroy>>();
        world.init_resource::<Events<ShutdownEventLoop>>();
        world.init_resource::<Events<CloseRequested>>();

        world.init_resource::<WindowCallbacksManager>();
        {
            let state = CreateWindowSystemState::new(world);
            world.insert_resource(state);
        }
        {
            let state = DestroyWindowSystemState::new(world);
            world.insert_resource(state);
        }
        {
            let state = CloseRequestedSystemState::new(world);
            world.insert_resource(state);
        }

        // Systems ========================================================

        schedule.stage(CoreStages::EventUpdate, |stage: &mut SystemStage| {
            stage.add_system(Events::<Create>::update_system);
            stage.add_system(Events::<Resize>::update_system);
            stage.add_system(Events::<Repaint>::update_system);
            stage.add_system(Events::<Destroy>::update_system);
            stage.add_system(Events::<CloseRequested>::update_system);
            stage.add_system(Events::<ShutdownEventLoop>::update_system);
            stage
        });
        schedule.stage(CoreStages::PreUpdate, |stage: &mut SystemStage| {
            stage.add_system_set(
                SystemSet::new()
                    .with_system(events::create_window_system.exclusive_system())
                    .label(events::WINDOW_LIFECYCLE_SYSTEMS),
            );
            stage.add_system_set(
                SystemSet::new()
                    .with_system(events::windows_close_requested_system.exclusive_system())
                    .after(events::WINDOW_LIFECYCLE_SYSTEMS),
            );
            stage
        });
        schedule.stage(CoreStages::PostUpdate, |stage: &mut SystemStage| {
            stage.add_system(events::destroy_window_system.exclusive_system());
            stage
        });
        schedule.stage(CoreStages::Update, |stage: &mut SystemStage| {
            stage.add_system(create_surface_for_window_system);
            stage
        });
        schedule.stage(CoreStages::Present, |stage: &mut SystemStage| {
            stage.add_system(repaint_and_present_windows);
            stage
        });
        schedule.stage(CoreStages::Last, |stage: &mut SystemStage| {
            stage.add_system(shutdown_eventloop_system);
            stage
        });
    }
}

pub trait CallbackTranslator<D> {
    fn event_loop_started(&mut self, ctx: &mut EcsContext<D>);
    fn event_loop_closed(&mut self, world: &mut EcsContext<D>);
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
    pub fn run<D: 'static>(
        self,
        data: D,
        mut callbacks: impl CallbackTranslator<D> + 'static,
    ) -> ! {
        tracing::debug_span!("EventLoop::run");

        let mut ctx = EcsContext::new(data);
        self.event_loop.run(move |event, target, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                TaoEvent::NewEvents(StartCause::Init) => {
                    ctx.world.insert_non_send_resource(target.clone());
                    ctx.world.insert_resource(*control_flow);
                    callbacks.event_loop_started(&mut ctx);
                }
                TaoEvent::WindowEvent {
                    event: window_event,
                    window_id,
                    ..
                } => Self::dispatch_window_events(window_event, window_id, &mut ctx),
                TaoEvent::RedrawRequested(raw_id) => {
                    let window_id = find_window_id_from_raw_id(raw_id, &mut ctx);
                    if let Some(entity) = ctx
                        .world
                        .query::<(Entity, &WindowId)>()
                        .iter(&mut ctx.world)
                        .find_map(|(entity, &id)| if window_id == id { Some(entity) } else { None })
                    {
                        ctx.post_event(Repaint(entity));
                    }
                }
                TaoEvent::MainEventsCleared => {
                    ctx.schedule.run_once(&mut ctx.world);
                    *control_flow = *ctx.world.get_resource::<ControlFlow>().unwrap();
                }
                TaoEvent::LoopDestroyed => {
                    callbacks.event_loop_closed(&mut ctx);
                }
                _ => (),
            }
        })
    }

    fn dispatch_window_events<D>(
        window_event: TaoWindowEvent,
        raw_id: tao::window::WindowId,
        ctx: &mut EcsContext<D>,
    ) {
        tracing::debug_span!("EventLoop::dispatch_window_events");
        match window_event {
            TaoWindowEvent::CloseRequested => {
                tracing::debug_span!("EventLoop::dispatch_window_events", "CloseRequested");
                let window_id = find_window_id_from_raw_id(raw_id, ctx);
                if let Some(entity) = ctx
                    .world
                    .query::<(Entity, &WindowId)>()
                    .iter(&mut ctx.world)
                    .find_map(|(entity, &id)| if window_id == id { Some(entity) } else { None })
                {
                    ctx.post_event(CloseRequested(entity));
                }
            }
            TaoWindowEvent::Destroyed => {
                // TODO: manually emit this event from the window event handler system
            }
            TaoWindowEvent::Resized(_) => {
                tracing::debug_span!("EventLoop::dispatch_window_events", "Resized");
                let window_id = find_window_id_from_raw_id(raw_id, ctx);
                if let Some(entity) = ctx
                    .world
                    .query::<(Entity, &WindowId)>()
                    .iter(&mut ctx.world)
                    .find_map(|(entity, &id)| if window_id == id { Some(entity) } else { None })
                {
                    ctx.post_event(Resize { window_id, entity });
                }
            }
            _ => (),
        }
    }
}

fn find_window_id_from_raw_id<D>(
    raw_id: tao::window::WindowId,
    ctx: &mut EcsContext<D>,
) -> WindowId {
    tracing::debug_span!("windowing::find_window_id_from_raw_id");

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

#[derive(Default, Debug)]
struct ShutdownEventLoop;
impl Event for ShutdownEventLoop {}

fn shutdown_eventloop_system(
    mut control_flow: ResMut<ControlFlow>,
    events: EventReader<ShutdownEventLoop>,
) {
    assert_is_system(shutdown_eventloop_system);
    if !events.is_empty() {
        *control_flow = ControlFlow::ExitWithCode(0);
    }
}
