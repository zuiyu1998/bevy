use crate::alloc::string::{String, ToString};
use crate::*;
use crate::{system::System, World};
use core::{marker::PhantomData, ptr::NonNull};
use entities::EntityMeta;
use resource_query::{FetchResource, ResourceQuery};
use resources::Resources;
use std::boxed::Box;
use system::ThreadLocalExecution;

pub struct SystemFn<F>
where
    F: FnMut(CommandBuffer, &World, &Resources) + Send + Sync,
{
    pub func: F,
    pub command_buffer: CommandBuffer,
    pub thread_local_execution: ThreadLocalExecution,
    pub name: String,
    // TODO: add dependency info here
}

impl<F> System for SystemFn<F>
where
    F: FnMut(CommandBuffer, &World, &Resources) + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn thread_local_execution(&self) -> ThreadLocalExecution {
        self.thread_local_execution
    }

    fn run(&mut self, world: &World, resources: &Resources) {
        (self.func)(self.command_buffer.clone(), world, resources);
    }

    fn run_thread_local(&mut self, world: &mut World, _resources: &mut Resources) {
        let command_buffer = core::mem::replace(&mut self.command_buffer, CommandBuffer::default());
        command_buffer.apply(world);
    }
}

#[doc(hidden)]
pub trait IntoForEachSystem<CommandBuffer, R, C> {
    fn system(self) -> Box<dyn System>;
}

macro_rules! impl_into_foreach_system {
    (($($command_buffer: ident)*), ($($resource: ident),*), ($($component: ident),*)) => {
        impl<Func, $($resource,)* $($component,)*> IntoForEachSystem<($($command_buffer,)*), ($($resource,)*), ($($component,)*)> for Func
        where
            Func:
                FnMut($($command_buffer,)* $($resource,)* $($component,)*) +
                FnMut(
                    $($command_buffer,)*
                    $(<<$resource as ResourceQuery>::Fetch as FetchResource>::Item,)*
                    $(<<$component as Query>::Fetch as Fetch>::Item,)*)+
                Send + Sync + 'static,
            $($component: Query,)*
            $($resource: ResourceQuery,)*
        {
            #[allow(non_snake_case)]
            #[allow(unused_variables)]
            fn system(mut self) -> Box<dyn System> {
                Box::new(SystemFn {
                    command_buffer: CommandBuffer::default(),
                    thread_local_execution: ThreadLocalExecution::NextFlush,
                    name: core::any::type_name::<Self>().to_string(),
                    func: move |command_buffer, world, resources| {
                        let ($($resource,)*) = resources.query::<($($resource,)*)>();
                        for ($($component,)*) in world.query::<($($component,)*)>().iter() {
                            fn_call!(self, ($($command_buffer, command_buffer)*), ($($resource),*), ($($component),*))
                        }
                    },
                })
            }
        }
    };
}

#[doc(hidden)]
pub struct WorldQuery<'a, Q: Query> {
    world: &'a World,
    _marker: PhantomData<Q>,
}

#[doc(hidden)]
impl<'a, Q: Query> WorldQuery<'a, Q> {
    // TODO: allow getting components here that match the query
    pub fn iter(&mut self) -> QueryBorrow<'_, Q> {
        self.world.query::<Q>()
    }
}

#[doc(hidden)]
pub trait IntoQuerySystem<CommandBuffer, R, Q> {
    fn system(self) -> Box<dyn System>;
}

macro_rules! impl_into_query_system {
    (($($command_buffer: ident)*), ($($resource: ident),*), ($($query: ident),*)) => {
        impl<Func, $($resource,)* $($query,)*> IntoQuerySystem<($($command_buffer,)*), ($($resource,)*), ($($query,)*)> for Func where
            Func:
                FnMut($($command_buffer,)* $($resource,)* $(WorldQuery<$query>,)*) +
                FnMut(
                    $($command_buffer,)*
                    $(<<$resource as ResourceQuery>::Fetch as FetchResource>::Item,)*
                    $(WorldQuery<$query>,)*) +
                Send + Sync +'static,
            $($query: Query,)*
            $($resource: ResourceQuery,)*
        {
            #[allow(non_snake_case)]
            #[allow(unused_variables)]
            fn system(mut self) -> Box<dyn System> {
                Box::new(SystemFn {
                    command_buffer: CommandBuffer::default(),
                    thread_local_execution: ThreadLocalExecution::NextFlush,
                    name: core::any::type_name::<Self>().to_string(),
                    func: move |command_buffer, world, resources| {
                        let ($($resource,)*) = resources.query::<($($resource,)*)>();
                        $(let $query = WorldQuery::<$query> {
                            world,
                            _marker: PhantomData::default(),
                        };)*

                        fn_call!(self, ($($command_buffer, command_buffer)*), ($($resource),*), ($($query),*))
                    },
                })
            }
        }
    };
}

macro_rules! fn_call {
    ($self:ident, ($($command_buffer: ident, $command_buffer_var: ident)*), ($($resource: ident),*), ($($a: ident),*)) => {
        $self($($command_buffer_var.clone(),)* $($resource.clone(),)* $($a,)*)
    };
    ($self:ident, (), ($($resource: ident),*), ($($a: ident),*)) => {
        $self($($resource.clone(),)* $($a,)*)
    };
}

macro_rules! impl_into_query_systems {
    (($($resource: ident,)*), ($($query: ident),*)) => {
        #[rustfmt::skip]
        impl_into_query_system!((), ($($resource),*), ($($query),*));
        #[rustfmt::skip]
        impl_into_query_system!((CommandBuffer), ($($resource),*), ($($query),*));
    }
}

macro_rules! impl_into_foreach_systems {
    (($($resource: ident,)*), ($($component: ident),*)) => {
        #[rustfmt::skip]
        impl_into_foreach_system!((), ($($resource),*), ($($component),*));
        #[rustfmt::skip]
        impl_into_foreach_system!((CommandBuffer), ($($resource),*), ($($component),*));
    }
}

macro_rules! impl_into_systems {
    ($($resource: ident),*) => {
        #[rustfmt::skip]
        impl_into_foreach_systems!(($($resource,)*), (A));
        #[rustfmt::skip]
        impl_into_foreach_systems!(($($resource,)*), (A,B));
        #[rustfmt::skip]
        impl_into_foreach_systems!(($($resource,)*), (A,B,C));
        #[rustfmt::skip]
        impl_into_foreach_systems!(($($resource,)*), (A,B,C,D));
        #[rustfmt::skip]
        impl_into_foreach_systems!(($($resource,)*), (A,B,C,D,E));
        #[rustfmt::skip]
        impl_into_foreach_systems!(($($resource,)*), (A,B,C,D,E,F));
        #[rustfmt::skip]
        impl_into_foreach_systems!(($($resource,)*), (A,B,C,D,E,F,G));
        #[rustfmt::skip]
        impl_into_foreach_systems!(($($resource,)*), (A,B,C,D,E,F,G,H));

        #[rustfmt::skip]
        impl_into_query_systems!(($($resource,)*), ());
        #[rustfmt::skip]
        impl_into_query_systems!(($($resource,)*), (A));
        #[rustfmt::skip]
        impl_into_query_systems!(($($resource,)*), (A,B));
        #[rustfmt::skip]
        impl_into_query_systems!(($($resource,)*), (A,B,C));
        #[rustfmt::skip]
        impl_into_query_systems!(($($resource,)*), (A,B,C,D));
        #[rustfmt::skip]
        impl_into_query_systems!(($($resource,)*), (A,B,C,D,E));
        #[rustfmt::skip]
        impl_into_query_systems!(($($resource,)*), (A,B,C,D,E,F));
    };
}

#[rustfmt::skip]
impl_into_systems!();
#[rustfmt::skip]
impl_into_systems!(Ra);
#[rustfmt::skip]
impl_into_systems!(Ra,Rb);
#[rustfmt::skip]
impl_into_systems!(Ra,Rb,Rc);
#[rustfmt::skip]
impl_into_systems!(Ra,Rb,Rc,Rd);
#[rustfmt::skip]
impl_into_systems!(Ra,Rb,Rc,Rd,Re);
#[rustfmt::skip]
impl_into_systems!(Ra,Rb,Rc,Rd,Re,Rf);
#[rustfmt::skip]
impl_into_systems!(Ra,Rb,Rc,Rd,Re,Rf,Rg);
#[rustfmt::skip]
impl_into_systems!(Ra,Rb,Rc,Rd,Re,Rf,Rg,Rh);
#[rustfmt::skip]
impl_into_systems!(Ra,Rb,Rc,Rd,Re,Rf,Rg,Rh,Ri);
#[rustfmt::skip]
impl_into_systems!(Ra,Rb,Rc,Rd,Re,Rf,Rg,Rh,Ri,Rj);

#[derive(Copy, Clone, Debug)]
pub struct EntityFetch(NonNull<u32>);

impl Query for Entity {
    type Fetch = EntityFetch;
}

impl<'a> Fetch<'a> for EntityFetch {
    type Item = Entity;
    fn access(_archetype: &Archetype) -> Option<Access> {
        Some(Access::Iterate)
    }
    fn borrow(_archetype: &Archetype) {}
    #[inline]
    unsafe fn get(archetype: &'a Archetype, offset: usize) -> Option<Self> {
        Some(EntityFetch(NonNull::new_unchecked(
            archetype.entities().as_ptr().add(offset),
        )))
    }
    fn release(_archetype: &Archetype) {}

    #[inline]
    unsafe fn next(&mut self, meta: &[EntityMeta]) -> Self::Item {
        let id = self.0.as_ptr();
        self.0 = NonNull::new_unchecked(id.add(1));
        Entity {
            id: *id,
            generation: meta[*id as usize].generation,
        }
    }
}

pub struct ThreadLocalSystem<F>
where
    F: ThreadLocalSystemFn,
{
    func: F,
    name: String,
    // TODO: add dependency info here
}
impl<F> ThreadLocalSystem<F>
where
    F: ThreadLocalSystemFn,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            name: core::any::type_name::<F>().to_string(),
        }
    }
}
pub trait ThreadLocalSystemFn {
    fn run(&mut self, world: &mut World, resource: &mut Resources);
}

impl<F> ThreadLocalSystemFn for F
where
    F: FnMut(&mut World, &mut Resources) + 'static,
{
    fn run(&mut self, world: &mut World, resources: &mut Resources) {
        self(world, resources);
    }
}

impl<F> System for ThreadLocalSystem<F>
where
    F: ThreadLocalSystemFn + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn thread_local_execution(&self) -> ThreadLocalExecution {
        ThreadLocalExecution::Immediate
    }

    fn run(&mut self, _world: &World, _resources: &Resources) {}

    fn run_thread_local(&mut self, world: &mut World, resources: &mut Resources) {
        self.func.run(world, resources);
    }
}
