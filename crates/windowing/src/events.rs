use crate::window::{
    self, OsWindow, Window, WindowBundle, WindowCallbacks, WindowCallbacksManager, WindowContext,
    WindowId,
};
use bevy_ecs::{
    event::Events,
    prelude::{
        Added, Commands, Entity, EventReader, EventWriter, Mut, NonSend, Query, ResMut, With, World,
    },
    system::{assert_is_system, SystemState},
};
use derive_deref::{Deref, DerefMut};
use renderer::{Surface, WindowDrawTarget};
use smallvec::SmallVec;
use tao::event_loop::EventLoopWindowTarget;
use utilities::tracing;

pub const WINDOW_LIFECYCLE_SYSTEMS: &str = "window-lifecycle";
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
pub struct Destroy(pub Entity);

#[derive(Debug, Copy, Clone, Deref, DerefMut)]
pub struct Repaint(pub Entity);

#[derive(Debug, Copy, Clone, Deref, DerefMut)]
pub struct CloseRequested(pub Entity);

macro_rules! impl_event {
   ($($TYPE:ty),*) => {
      $(
         impl ::utilities::Event for $TYPE {}
      )*
   }
}

impl_event!(Create, Resize, Repaint, Destroy, CloseRequested);

pub(crate) type CreateWindowSystemState<'w, 's> = SystemState<(
    Query<'w, 's, (Entity, &'static WindowId), With<window::Marker>>,
    NonSend<'w, EventLoopWindowTarget<()>>,
    ResMut<'w, Events<Create>>,
)>;
pub(crate) fn create_window_system(world: &mut World) {
    // Guard to prevent unessersary work from being done.
    if world.get_resource::<Events<Create>>().unwrap().is_empty() {
        return;
    }

    tracing::debug_span!("create_window_system");
    world.resource_scope(|world, mut state: Mut<CreateWindowSystemState>| {
        let (window_id_query, event_loop, mut events) = state.get_mut(world);
        let bundles = events
            .drain()
            .map(|Create { window, window_id }| {
                #[cfg(debug_assertions)]
                if window_id_query
                    .iter()
                    .find(|(_, &id)| id == window_id)
                    .is_some()
                {
                    panic!("Window already existed with {:?}", window_id);
                }
                match WindowBundle::new(
                    window_id,
                    &*event_loop,
                    // TODO make this the windows root widget entity
                    Entity::from_raw(0),
                ) {
                    Ok(window_bundle) => (window_bundle, window),
                    Err(err) => panic!("{}", err),
                }
            })
            .collect::<SmallVec<[_; 4]>>();

        for (bundle, callbacks) in bundles {
            let entity = world.spawn().insert_bundle(bundle).id();

            callbacks.on_create(WindowContext { entity, world });

            let mut window_callback_manager = world
                .get_resource_mut::<WindowCallbacksManager>()
                .expect("No WindowCallbacksManager Resource");
            window_callback_manager.insert(entity, WindowCallbacks(callbacks));
        }
        state.apply(world);
    });
}

pub(crate) type DestroyWindowSystemState<'w, 's> =
    SystemState<(Commands<'w, 's>, EventReader<'w, 's, Destroy>)>;

pub(crate) fn destroy_window_system(world: &mut World) {
    // Guard to prevent unessersary work from being done.
    if world.get_resource::<Events<Destroy>>().unwrap().is_empty() {
        return;
    }

    tracing::debug_span!("destroy_window_system");
    world.resource_scope(|world, mut state: Mut<DestroyWindowSystemState>| {
        let (_, mut events) = state.get_mut(world);
        let entities = events
            .iter()
            .map(|Destroy(entity)| *entity)
            .collect::<SmallVec<[_; 4]>>();
        for entity in entities {
            world.resource_scope(|world, mut window_callbacks: Mut<WindowCallbacksManager>| {
                let callbacks = window_callbacks
                    .get_mut(entity)
                    .expect("No callbacks found for this entity");
                callbacks.on_destroyed(WindowContext { entity, world });
            });
            let (mut commands, _) = state.get_mut(world);
            commands.entity(entity).despawn();
        }
        state.apply(world);
    });
}

pub(crate) type CloseRequestedSystemState<'w, 's> = SystemState<(
    Commands<'w, 's>,
    EventReader<'w, 's, CloseRequested>,
    EventWriter<'w, 's, Destroy>,
)>;

pub(crate) fn windows_close_requested_system(world: &mut World) {
    // Guard to prevent unessersary work from being done.
    if world
        .get_resource::<Events<CloseRequested>>()
        .unwrap()
        .is_empty()
    {
        return;
    }
    tracing::debug_span!("windows_close_requested_system");

    world.resource_scope(|world, mut state: Mut<CloseRequestedSystemState>| {
        let (_, mut close_requested_events, _) = state.get_mut(world);
        let entities = close_requested_events
            .iter()
            .map(|CloseRequested(entity)| *entity)
            .collect::<SmallVec<[_; 4]>>();

        let mut windows_to_close = SmallVec::<[Entity; 4]>::new();
        for entity in entities {
            world.resource_scope(|world, mut window_callbacks: Mut<WindowCallbacksManager>| {
                let callbacks = window_callbacks
                    .get_mut(entity)
                    .expect("No callbacks found for this entity");
                if callbacks.close_requested(WindowContext { entity, world }) {
                    windows_to_close.push(entity);
                }
            });
        }
        let (_, _, mut destroy_events) = state.get_mut(world);
        windows_to_close.iter().for_each(|entity| {
            destroy_events.send(Destroy(*entity));
        });
        state.apply(world);
    });
}

pub(crate) fn create_surface_for_window_system(
    mut commands: Commands,
    new_windows: Query<Entity, Added<window::Marker>>,
    mut events: EventReader<Resize>,
    mut redraw_events: EventWriter<Repaint>,
    mut window_query: Query<&mut OsWindow, With<window::Marker>>,
) {
    assert_is_system(create_surface_for_window_system);
    tracing::debug_span!("create_surface_for_window_system");
    let windows_to_create_surfaces_for = events
        .iter()
        .map(
            |Resize {
                 window_id: _,
                 entity,
             }| *entity,
        )
        .chain(new_windows.iter());

    for entity in windows_to_create_surfaces_for {
        let os_window = match window_query.get_mut(entity) {
            Ok(os_window) => os_window,
            Err(err) => panic!("{}", err),
        };

        // if the bounds are 0 then keep the old surface as skia will panic if
        // a surface is created with 0 bounds
        let (width, height) = os_window.get_draw_bounds();
        if width <= 0 || height <= 0 {
            continue;
        }

        let surface = match Surface::new_cpu(&*os_window) {
            Ok(new_surface) => new_surface,
            Err(err) => panic!("{}", err),
        };
        commands.entity(entity).insert(surface);
        redraw_events.send(Repaint(entity));
    }
}

pub fn repaint_and_present_windows(
    mut events: EventReader<Repaint>,
    mut windows: Query<(&mut OsWindow, &mut Surface), With<window::Marker>>,
) {
    assert_is_system(repaint_and_present_windows);
    tracing::debug_span!("repaint_and_present_windows");
    for entity in events.iter().map(|&event| *event) {
        let (os_window, mut surface) = match windows.get_mut(entity) {
            Ok((os_window, surface)) => (os_window, surface),
            Err(_) => continue,
        };
        match surface.present_surface(&*os_window) {
            Ok(_) => {}
            Err(err) => panic!("{}", err),
        };
    }
}
