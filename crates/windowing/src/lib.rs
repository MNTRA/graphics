pub mod events;
pub mod window;

use bevy_ecs::{
    event::Events,
    prelude::{Entity, IntoChainSystem},
    schedule::{ParallelSystemDescriptorCoercion, Schedule, SystemStage},
    world::World,
};
use events::{Create, Resize, WindowEvent};
use tao::{
    event::{Event as TaoEvent, StartCause, WindowEvent as TaoWindowEvent},
    event_loop::{ControlFlow, EventLoop as TaoEventLoop},
};
use utilities::{CoreStages, EcsContext, EcsPlugin};
use window::{TaoWindowIdWapper, WindowId};

use crate::events::Repaint;

pub struct WindowingPlugin;
impl EcsPlugin for WindowingPlugin {
    fn build(world: &mut World, schedule: &mut Schedule) {
        world.init_resource::<Events<Create>>();
        world.init_resource::<Events<Resize>>();
        world.init_resource::<Events<Repaint>>();
        world.init_resource::<Events<WindowEvent>>();

        const EVENT_UPDATE_SYS_LABEL: &str = "events-update";
        schedule.stage(CoreStages::EventUpdate, |stage: &mut SystemStage| {
            stage.add_system(Events::<WindowEvent>::update_system.label(EVENT_UPDATE_SYS_LABEL));
            stage
        });
        schedule.stage(CoreStages::PreUpdate, |stage: &mut SystemStage| {
            stage.add_system(events::create_window_system.after(EVENT_UPDATE_SYS_LABEL));
            stage.add_system(
                events::window_events_system
                    .chain(events::destroyed_window)
                    .after(events::CREATE_WINDOWS_SYS_LABEL),
            );
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
                        ctx.post_event(Repaint { window_id, entity });
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
        match window_event {
            TaoWindowEvent::CloseRequested => {
                let id = find_window_id_from_raw_id(raw_id, ctx);
                ctx.post_event(WindowEvent::CloseRequested(id));
            }
            TaoWindowEvent::Destroyed => {
                // TODO: manually emit this event from the window event handler system
            }
            TaoWindowEvent::Resized(_) => {
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
