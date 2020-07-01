use crate::alloc::boxed::Box;
use crate::alloc::vec::Vec;
use crate::{DynamicBundle, World};
use std::sync::Mutex;
use alloc::sync::Arc;

pub enum Command {
    WriteWorld(Box<dyn WorldWriter>),
}

pub trait WorldWriter: Send + Sync {
    fn write(self: Box<Self>, world: &mut World);
}

pub struct Spawn<T>
where
    T: DynamicBundle + Send + Sync + 'static,
{
    value: T,
}

impl<T> WorldWriter for Spawn<T>
where
    T: DynamicBundle + Send + Sync + 'static,
{
    fn write(self: Box<Self>, world: &mut World) {
        world.spawn(self.value);
    }
}

#[derive(Default, Clone)]
pub struct CommandBuffer {
    commands: Arc<Mutex<Vec<Command>>>,
}

impl CommandBuffer {
    pub fn spawn(&mut self, components: impl DynamicBundle + Send + Sync + 'static) -> &mut Self {
        self.commands.lock().unwrap()
            .push(Command::WriteWorld(Box::new(Spawn { value: components })));
        self
    }

    pub fn apply(self, world: &mut World) {
        let mut commands = self.commands.lock().unwrap();
        for command in commands.drain(..) {
            match command {
                Command::WriteWorld(writer) => {
                    writer.write(world);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::alloc::vec::Vec;
    use crate::alloc::*;
    use crate::{CommandBuffer, World};

    #[test]
    fn command_buffer() {
        let mut world = World::default();
        let mut command_buffer = CommandBuffer::default();
        command_buffer.spawn((1u32, 2u64));
        command_buffer.apply(&mut world);
        let results = world
            .query::<(&u32, &u64)>()
            .iter()
            .map(|(a, b)| (*a, *b))
            .collect::<Vec<_>>();
        assert_eq!(results, vec![(1u32, 2u64)])
    }
}
