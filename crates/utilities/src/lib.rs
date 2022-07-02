use bevy_ecs::{prelude::World, schedule::Schedule};

pub trait EcsPlugin {
    fn build(world: &mut World, schedule: &mut Schedule);
}
