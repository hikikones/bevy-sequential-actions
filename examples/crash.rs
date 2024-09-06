use bevy_ecs::world::World;

fn main() {
    let mut world = World::new();
    let e = world.spawn_empty().id();
    world.entity_mut(e).world_scope(|world: &mut World| {
        world.despawn(e);
    });
}
