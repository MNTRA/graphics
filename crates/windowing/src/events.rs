use bevy_ecs::{
    prelude::{Commands, Entity, In, NonSend, Query, ResMut, With},
    system::assert_is_system, event::Events,
};
use tao::event_loop::{EventLoopWindowTarget, ControlFlow};
use crate::window::{DynWindow, OsWindow, Window, WindowBundle, WindowContext, WindowId, self};

#[derive(Debug)]
pub enum WindowEvent {
    Destroyed(WindowId),
    CloseRequested(WindowId),
}

pub const CREATE_WINDOWS_SYS_LABEL: &str = "create-windows";
#[derive(Debug)]
pub struct Create {
    pub window: Box<dyn Window>,
    pub window_id: WindowId,
}

#[derive(Debug, Copy, Clone)]
pub struct Resize {
    pub window_id: WindowId,
    pub entity: Entity,
}

#[derive(Debug, Copy, Clone)]
pub struct Repaint {
    pub window_id: WindowId,
    pub entity: Entity,
}

macro_rules! impl_event {
   ($($TYPE:ty),*) => {
      $(
         impl ::utilities::Event for $TYPE {}
      )*
   }
}

impl_event!(
    WindowEvent,
    Create,
    Resize,
    Repaint
);

pub(crate) fn create_window_system(
    mut commands: Commands,
    mut events: ResMut<Events<Create>>,
    event_loop: NonSend<EventLoopWindowTarget<()>>,
    window_id_query: Query<(Entity, &WindowId), With<window::Marker>>,
) {
    assert_is_system(create_window_system);
    for Create { window, window_id } in events.drain() {
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
}

pub(crate) fn window_events_system(
    mut commands: Commands,
    mut window_events: ResMut<Events<WindowEvent>>,
    mut control_flow: ResMut<ControlFlow>,
    window_id_query: Query<(Entity, &WindowId), With<window::Marker>>,
    mut window_query: Query<(Entity, &mut OsWindow, &mut DynWindow), With<window::Marker>>,
) -> Vec<WindowId> {
    assert_is_system(window_events_system);
    let mut destroyed_windows = Vec::new();
    for event in window_events.drain() {
        match event {
            WindowEvent::Destroyed(_) => {
                if window_id_query.iter().count() == 0 {
                    *control_flow = ControlFlow::Exit;
                }
            }
            WindowEvent::CloseRequested(window_id) => {
                let should_close_window;
                let entity = if let Some((entity, _)) =
                    window_id_query.iter().find(|(_, &id)| id == window_id)
                {
                    let (mut os_window, mut dyn_window) = match window_query.get_mut(entity) {
                        Ok((_, os_window, dyn_window)) => (os_window, dyn_window),
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
