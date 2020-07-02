use super::{
    system::{System, ThreadLocalExecution},
    Resources, World,
};
use std::{borrow::Cow, collections::{HashSet, HashMap}};

#[derive(Default)]
pub struct Schedule {
    pub stages: HashMap<Cow<'static, str>, Vec<Box<dyn System>>>,
    pub stage_order: Vec<Cow<'static, str>>,
    system_names: HashSet<Cow<'static, str>>,
}

impl Schedule {
    pub fn add_stage(&mut self, stage: impl Into<Cow<'static, str>>) {
        let stage: Cow<str> = stage.into();
        if let Some(_) = self.stages.get(&stage) {
            panic!("Stage already exists: {}", stage);
        } else {
            self.stages.insert(stage.clone(), Vec::new());
            self.stage_order.push(stage);
        }
    }

    pub fn add_stage_after(
        &mut self,
        target: impl Into<Cow<'static, str>>,
        stage: impl Into<Cow<'static, str>>,
    ) {
        let target: Cow<str> = target.into();
        let stage: Cow<str> = stage.into();
        if let Some(_) = self.stages.get(&stage) {
            panic!("Stage already exists: {}", stage);
        }

        let target_index = self
            .stage_order
            .iter()
            .enumerate()
            .find(|(_i, stage)| **stage == target)
            .map(|(i, _)| i)
            .unwrap_or_else(|| panic!("Target stage does not exist: {}", target));

        self.stages.insert(stage.clone(), Vec::new());
        self.stage_order.insert(target_index + 1, stage);
    }

    pub fn add_stage_before(
        &mut self,
        target: impl Into<Cow<'static, str>>,
        stage: impl Into<Cow<'static, str>>,
    ) {
        let target: Cow<str> = target.into();
        let stage: Cow<str> = stage.into();
        if let Some(_) = self.stages.get(&stage) {
            panic!("Stage already exists: {}", stage);
        }

        let target_index = self
            .stage_order
            .iter()
            .enumerate()
            .find(|(_i, stage)| **stage == target)
            .map(|(i, _)| i)
            .unwrap_or_else(|| panic!("Target stage does not exist: {}", target));

        self.stages.insert(stage.clone(), Vec::new());
        self.stage_order.insert(target_index, stage);
    }

    pub fn add_system_to_stage(
        &mut self,
        stage_name: impl Into<Cow<'static, str>>,
        system: Box<dyn System>,
    ) -> &mut Self {
        let stage_name = stage_name.into();
        let systems = self
            .stages
            .get_mut(&stage_name)
            .unwrap_or_else(|| panic!("Stage does not exist: {}", stage_name));
        let system_name = Cow::Owned(system.name().to_string());
        if self.system_names.contains(&system_name) {
            panic!("System with name {} already exists", system_name);
        }
        self.system_names.insert(system_name);
        systems.push(system);

        self
    }

    pub fn run(&mut self, world: &mut World, resources: &mut Resources) {
        for stage_name in self.stage_order.iter() {
            if let Some(stage_systems) = self.stages.get_mut(stage_name) {
                for system in stage_systems.iter_mut() {
                    match system.thread_local_execution() {
                        ThreadLocalExecution::NextFlush => system.run(world, resources),
                        ThreadLocalExecution::Immediate => {
                            system.run(world, resources);
                            // NOTE: when this is made parallel a full sync is required here
                            system.run_thread_local(world, resources);
                        }
                    }
                }

                // "flush"
                // NOTE: when this is made parallel a full sync is required here
                for system in stage_systems.iter_mut() {
                    match system.thread_local_execution() {
                        ThreadLocalExecution::NextFlush => {
                            system.run_thread_local(world, resources)
                        }
                        ThreadLocalExecution::Immediate => { /* already ran immediate */ }
                    }
                }
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{Resources, Schedule, World};
//     use crate::{IntoForEachSystem, IntoQuerySystem};

//     #[test]
//     fn schedule() {
//         let mut world = World::new();
//         let mut resources = Resources::default();

//         world.spawn((1u32, 2u64));

//         let mut schedule = Schedule::default();
//         schedule.add_stage("A");
//         schedule.add_stage("B");

//         let xy_system = (|_x: &u32, _y: &mut u64| {

//         }).system();
//     }
// }
