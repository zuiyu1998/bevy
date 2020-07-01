use crate::{Resources, World};

#[derive(Copy, Clone)]
pub enum ThreadLocalExecution {
    Immediate,
    NextFlush,
} 

pub trait System: Send + Sync {
    fn name(&self) -> &str;
    fn thread_local_execution(&self) -> ThreadLocalExecution;
    fn run(&mut self, world: &World, resources: &Resources);
    fn run_thread_local(&mut self, world: &mut World, resources: &mut Resources);
}
