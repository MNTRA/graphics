use bevy_ecs::{
    prelude::{Commands, Entity, In, NonSend, Query, QueryState, ResMut, With},
    system::{assert_is_system, SystemParam},
};
use renderer::{Surface, WindowDrawTarget};
use tao::event_loop::EventLoopWindowTarget;

use crate::window::{DynWindow, OsWindow, Window, WindowBundle, WindowContext};

use super::*;

pub trait Event: std::fmt::Debug + Send + Sync + 'static {}

#[derive(Debug)]
pub enum WindowEvent {
    Create {
        window: Box<dyn Window>,
        window_id: WindowId,
    },
    Repaint(WindowId),
    Delete(WindowId),
    Destroyed(WindowId),
    Resize {
        window_id: WindowId,
        bounds: (u32, u32),
    },
    CloseRequested(WindowId),
}

macro_rules! impl_event {
   ($($TYPE:ty),*) => {
      $(
         impl self::Event for $TYPE {}
      )*
   }
}

impl_event!(
    WindowEvent,
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

pub(crate) fn window_events_system(
    mut commands: Commands,
    mut window_events: ResMut<Events<WindowEvent>>,
    event_loop: NonSend<EventLoopWindowTarget<()>>,
    mut control_flow: ResMut<ControlFlow>,
    window_id_query: Query<(Entity, &WindowId), With<window::Marker>>,
    mut window_query: Query<
        (Entity, &mut OsWindow, &mut DynWindow, &mut Surface),
        With<window::Marker>,
    >,
) -> Vec<WindowId> {
    assert_is_system(window_events_system);
    let mut destroyed_windows = Vec::new();
    for event in window_events.drain() {
        match event {
            WindowEvent::Create { window, window_id } => {
                if window_id_query
                    .iter()
                    .find(|(_, &id)| id == window_id)
                    .is_some()
                {
                    panic!("Window already existed with {:?}", window_id);
                }
                match WindowBundle::new_from_boxed_window(
                    window,
                    window_id,
                    &*event_loop,
                    Entity::from_raw(0),
                ) {
                    Ok(window_bundle) => commands.spawn().insert_bundle(window_bundle),
                    Err(err) => panic!("{}", err),
                };
            }
            WindowEvent::Delete(_) => {}
            WindowEvent::Destroyed(_) => {
                if window_id_query.iter().count() == 0 {
                    *control_flow = ControlFlow::Exit;
                }
            }
            WindowEvent::Resize {
                window_id,
                bounds: _,
            } => {
                let entity = match window_id_query.iter().find(|(_, &id)| id == window_id) {
                    Some((entity, _)) => entity,
                    None => panic!("No Window Found for id: {:?}", window_id),
                };
                let (mut surface, os_window) = match window_query.get_mut(entity) {
                    Ok((_, os_window, _, surface)) => (surface, os_window),
                    Err(err) => panic!("{}", err),
                };

                // if the bounds are 0 then keep the old surface as skia will panic if
                // a surface is created with 0 bounds
                let (width, height) = os_window.get_draw_bounds();
                if width <= 0 || height <= 0 {
                    continue;
                }

                match Surface::new_cpu(&*os_window) {
                    Ok(new_surface) => *surface = new_surface,
                    Err(err) => panic!("{}", err),
                }
                os_window.request_redraw();
            }
            WindowEvent::CloseRequested(window_id) => {
                let should_close_window;
                let entity = if let Some((entity, _)) =
                    window_id_query.iter().find(|(_, &id)| id == window_id)
                {
                    let (mut os_window, mut dyn_window) = match window_query.get_mut(entity) {
                        Ok((_, os_window, dyn_window, _)) => (os_window, dyn_window),
                        Err(err) => panic!("{}", err),
                    };
                    should_close_window = dyn_window.close_requested(WindowContext {
                        os_window: &mut *os_window,
                    });

                    entity
                } else {
                    panic!("No Window Found for id: {:?}", window_id);
                };

                if should_close_window {
                    destroyed_windows.push(window_id);
                    commands.entity(entity).despawn();
                }
            }
            WindowEvent::Repaint(window_id) => {
                let entity = match window_id_query.iter().find(|(_, &id)| id == window_id) {
                    Some((entity, _)) => entity,
                    None => panic!("No Window Found for id: {:?}", window_id),
                };
                let (mut surface, os_window) = match window_query.get_mut(entity) {
                    Ok((_, os_window, _, surface)) => (surface, os_window),
                    Err(err) => panic!("{}", err),
                };
                match surface.present_surface(&*os_window) {
                    Ok(_) => {}
                    Err(err) => panic!("{}", err),
                };
            }
        }
    }
    destroyed_windows
}

pub(crate) fn destroyed_window(
    In(ids): In<Vec<WindowId>>,
    mut window_events: ResMut<Events<WindowEvent>>,
) {
    assert_is_system(destroyed_window);
    for window_id in ids {
        window_events.send(WindowEvent::Destroyed(window_id))
    }
}
