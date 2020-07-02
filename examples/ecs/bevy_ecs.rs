use ecs::*;

struct Position(f32);
struct Velocity(f32);
struct NewThing(f32);
struct MyResource(f32);
struct OtherResource(f32);

fn velocity_system(velocity: &Velocity) {
    println!("velocity {}", velocity.0);
}

fn move_system(velocity: &Velocity, entity: Entity) {
    println!("move {} {:?}", velocity.0, entity);
}

fn query_system(mut query: Query<(&Velocity, &Position, Entity)>) {
    for (velocity, position, entity_id) in &mut query.iter() {
        println!("{:?} {} {}", entity_id, velocity.0, position.0);
    }
}

fn command_buffer_query_system(
    mut command_buffer: CommandBuffer,
    _query: Query<(&Velocity, &Position, Entity)>,
) {
    command_buffer.spawn((NewThing(1.0), Position(1.0)));
}

fn get_thing(thing: &NewThing) {
    println!("thing {}", thing.0);
}

fn resource_foreach_system(
    my_resource: Res<MyResource>,
    _other_resource: ResMut<OtherResource>,
    _velocity: &Velocity,
) {
    println!("resource {}", my_resource.0);
}

fn main() {
    let mut world = World::default();
    let mut resources = Resources::default();
    

    world.spawn((Velocity(1.0), Position(1.0)));
    world.spawn((Velocity(2.0), Position(2.0)));

    resources.insert(MyResource(123.0));
    resources.insert(MyResource(456.0));
    resources.insert(OtherResource(2.0));
    if let Ok(my_resource) = resources.get::<MyResource>() {
        println!("{}", my_resource.0);
    }

    let (mut my_resource, other_resource) =
        resources.query::<(ResMut<MyResource>, Res<OtherResource>)>();
    my_resource.0 = 1.0;
    println!("resources: {} {}", my_resource.0, other_resource.0);

    let mut system = velocity_system.system();
    system.run(&world, &resources);

    let mut system = move_system.system();
    system.run(&world, &resources);

    let mut system = query_system.system();
    system.run(&world, &resources);

    let mut system = command_buffer_query_system.system();
    system.run(&world, &resources);
    system.run_thread_local(&mut world, &mut resources);

    for new_thing in world.query::<&NewThing>().iter() {
        println!("new thing {} ", new_thing.0);
    }

    let mut system = resource_foreach_system.system();
    system.run(&world, &resources);

    let mut schedule = Schedule::default();
    schedule.add_stage("update");
    schedule.add_stage("render");

    schedule.add_system_to_stage("update", velocity_system.system());
    schedule.add_system_to_stage("update", move_system.system());
    schedule.add_system_to_stage("update", command_buffer_query_system.system());
    schedule.add_system_to_stage("render", query_system.system());
    schedule.add_system_to_stage("render", get_thing.system());

    schedule.run(&mut world, &mut resources);
}
