use crate::{
    resource::{self, PreparedRead, PreparedWrite, ResourceSet, ResourceTypeId, Resources},
    schedule::{ArchetypeAccess, Runnable},
    QuerySet, SubWorld, SystemAccess, SystemId,
};
use fxhash::FxHashMap;
use legion_core::{
    borrow::{AtomicRefCell, RefMut},
    command::CommandBuffer,
    storage::ComponentTypeId,
    world::{World, WorldId},
};
use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use tracing::{debug, info, span, Level};
#[derive(Debug)]
pub struct Resource<'a, T: 'a> {
    #[allow(dead_code)]
    // held for drop impl
    _marker: PhantomData<&'a ()>,
    value: *const T,
}

unsafe impl<'a, T: resource::Resource> Send for Resource<'a, T> {}
unsafe impl<'a, T: resource::Resource> Sync for Resource<'a, T> {}
impl<'a, T: 'a> Clone for Resource<'a, T> {
    #[inline(always)]
    fn clone(&self) -> Self { Resource::new(self.value) }
}

impl<'a, T: 'a> Resource<'a, T> {
    #[inline(always)]
    fn new(resource: *const T) -> Self {
        Self {
            value: resource,
            _marker: PhantomData::default(),
        }
    }

    #[inline(always)]
    pub fn map<K: 'a, F: FnMut(&T) -> &K>(&self, mut f: F) -> Resource<'a, K> {
        Resource::new(f(&self))
    }
}

impl<'a, T: 'a> Deref for Resource<'a, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target { unsafe { &*self.value } }
}

impl<'a, T: 'a> AsRef<T> for Resource<'a, T> {
    #[inline(always)]
    fn as_ref(&self) -> &T { unsafe { &*self.value } }
}

impl<'a, T: 'a> std::borrow::Borrow<T> for Resource<'a, T> {
    #[inline(always)]
    fn borrow(&self) -> &T { unsafe { &*self.value } }
}

impl<'a, T> PartialEq for Resource<'a, T>
where
    T: 'a + PartialEq,
{
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl<'a, T> Eq for Resource<'a, T> where T: 'a + Eq {}

impl<'a, T> PartialOrd for Resource<'a, T>
where
    T: 'a + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}
impl<'a, T> Ord for Resource<'a, T>
where
    T: 'a + Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.value.cmp(&other.value) }
}

impl<'a, T> Hash for Resource<'a, T>
where
    T: 'a + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) { self.value.hash(state); }
}

impl<'a, T: resource::Resource> ResourceSet for Resource<'a, T> {
    type PreparedResources = Resource<'a, T>;

    unsafe fn fetch_unchecked(resources: &Resources) -> Self::PreparedResources {
        let resource = resources
            .get::<T>()
            .unwrap_or_else(|| panic!("Failed to fetch resource!: {}", std::any::type_name::<T>()));
        Resource::new(resource.deref() as *const T)
    }
    fn read_types() -> Vec<ResourceTypeId> { vec![ResourceTypeId::of::<T>()] }
    fn write_types() -> Vec<ResourceTypeId> { Vec::new() }
}

#[derive(Debug)]
pub struct ResourceMut<'a, T: 'a> {
    // held for drop impl
    _marker: PhantomData<&'a mut ()>,
    value: *mut T,
}

unsafe impl<'a, T: resource::Resource> Send for ResourceMut<'a, T> {}
unsafe impl<'a, T: resource::Resource> Sync for ResourceMut<'a, T> {}
impl<'a, T: 'a> Clone for ResourceMut<'a, T> {
    #[inline(always)]
    fn clone(&self) -> Self { ResourceMut::new(self.value) }
}

impl<'a, T: 'a> ResourceMut<'a, T> {
    #[inline(always)]
    fn new(resource: *mut T) -> Self {
        Self {
            value: resource,
            _marker: PhantomData::default(),
        }
    }

    #[inline(always)]
    pub fn map_into<K: 'a, F: FnMut(&mut T) -> K>(mut self, mut f: F) -> ResourceMut<'a, K> {
        ResourceMut::new(&mut f(&mut self))
    }
}

impl<'a, T: 'a> Deref for ResourceMut<'a, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target { unsafe { &*self.value } }
}

impl<'a, T: 'a> DerefMut for ResourceMut<'a, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target { unsafe { &mut *self.value } }
}

impl<'a, T: 'a> AsRef<T> for ResourceMut<'a, T> {
    #[inline(always)]
    fn as_ref(&self) -> &T { unsafe { &*self.value } }
}

impl<'a, T: 'a> std::borrow::Borrow<T> for ResourceMut<'a, T> {
    #[inline(always)]
    fn borrow(&self) -> &T { unsafe { &*self.value } }
}

impl<'a, T> PartialEq for ResourceMut<'a, T>
where
    T: 'a + PartialEq,
{
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}
impl<'a, T> Eq for ResourceMut<'a, T> where T: 'a + Eq {}

impl<'a, T> PartialOrd for ResourceMut<'a, T>
where
    T: 'a + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}
impl<'a, T> Ord for ResourceMut<'a, T>
where
    T: 'a + Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering { self.value.cmp(&other.value) }
}

impl<'a, T> Hash for ResourceMut<'a, T>
where
    T: 'a + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) { self.value.hash(state); }
}

impl<'a, T: resource::Resource> ResourceSet for ResourceMut<'a, T> {
    type PreparedResources = ResourceMut<'a, T>;

    unsafe fn fetch_unchecked(resources: &Resources) -> Self::PreparedResources {
        let mut resource = resources
            .get_mut::<T>()
            .unwrap_or_else(|| panic!("Failed to fetch resource!: {}", std::any::type_name::<T>()));
        ResourceMut::new(resource.deref_mut() as *mut T)
    }
    fn read_types() -> Vec<ResourceTypeId> { vec![ResourceTypeId::of::<T>()] }
    fn write_types() -> Vec<ResourceTypeId> { Vec::new() }
}

impl<T: resource::Resource> ResourceSet for PreparedRead<T> {
    type PreparedResources = PreparedRead<T>;

    unsafe fn fetch_unchecked(resources: &Resources) -> Self::PreparedResources {
        let resource = resources
            .get::<T>()
            .unwrap_or_else(|| panic!("Failed to fetch resource!: {}", std::any::type_name::<T>()));
        PreparedRead::new(resource.deref() as *const T)
    }
    fn read_types() -> Vec<ResourceTypeId> { vec![ResourceTypeId::of::<T>()] }
    fn write_types() -> Vec<ResourceTypeId> { Vec::new() }
}

impl<T: resource::Resource> ResourceSet for PreparedWrite<T> {
    type PreparedResources = PreparedWrite<T>;

    unsafe fn fetch_unchecked(resources: &Resources) -> Self::PreparedResources {
        let mut resource = resources
            .get_mut::<T>()
            .unwrap_or_else(|| panic!("Failed to fetch resource!: {}", std::any::type_name::<T>()));
        PreparedWrite::new(resource.deref_mut() as *mut T)
    }
    fn read_types() -> Vec<ResourceTypeId> { Vec::new() }
    fn write_types() -> Vec<ResourceTypeId> { vec![ResourceTypeId::of::<T>()] }
}

/// The concrete type which contains the system closure provided by the user.  This struct should
/// not be instantiated directly, and instead should be created using `SystemBuilder`.
///
/// Implements `Schedulable` which is consumable by the `StageExecutor`, executing the closure.
///
/// Also handles caching of archetype information in a `BitSet`, as well as maintaining the provided
/// information about what queries this system will run and, as a result, its data access.
///
/// Queries are stored generically within this struct, and the `SystemQuery` types are generated
/// on each `run` call, wrapping the world and providing the set to the user in their closure.
pub struct FuncSystem<R, Q, F>
where
    R: ResourceSet,
    Q: QuerySet,
    F: FuncSystemFn<
        Resources = <R as ResourceSet>::PreparedResources,
        Queries = <Q as QuerySet>::Queries,
    >,
{
    pub name: SystemId,
    pub _resources: PhantomData<R>,
    pub queries: AtomicRefCell<Q>,
    pub run_fn: AtomicRefCell<F>,
    pub archetypes: ArchetypeAccess,

    // These are stored statically instead of always iterated and created from the
    // query types, which would make allocations every single request
    pub access: SystemAccess,

    // We pre-allocate a command buffer for ourself. Writes are self-draining so we never have to rellocate.
    pub command_buffer: FxHashMap<WorldId, AtomicRefCell<CommandBuffer>>,
}

impl<R, Q, F> Runnable for FuncSystem<R, Q, F>
where
    R: ResourceSet,
    Q: QuerySet,
    F: FuncSystemFn<
        Resources = <R as ResourceSet>::PreparedResources,
        Queries = <Q as QuerySet>::Queries,
    >,
{
    fn name(&self) -> &SystemId { &self.name }

    fn reads(&self) -> (&[ResourceTypeId], &[ComponentTypeId]) {
        (&self.access.resources.reads, &self.access.components.reads)
    }
    fn writes(&self) -> (&[ResourceTypeId], &[ComponentTypeId]) {
        (
            &self.access.resources.writes,
            &self.access.components.writes,
        )
    }

    fn prepare(&mut self, world: &World) {
        if let ArchetypeAccess::Some(bitset) = &mut self.archetypes {
            self.queries.get_mut().filter_archetypes(world, bitset);
        }
    }

    fn accesses_archetypes(&self) -> &ArchetypeAccess { &self.archetypes }

    fn command_buffer_mut(&self, world: WorldId) -> Option<RefMut<CommandBuffer>> {
        self.command_buffer.get(&world).map(|cmd| cmd.get_mut())
    }

    unsafe fn run_unsafe(&mut self, world: &World, resources: &Resources) {
        let span = span!(Level::INFO, "System", system = %self.name);
        let _guard = span.enter();

        debug!("Initializing");
        let resources = R::fetch_unchecked(resources);
        let mut queries = self.queries.get_mut();
        let mut prepared_queries = queries.prepare();
        let mut world_shim = SubWorld::new(world, &self.access.components, &self.archetypes);
        let cmd = self
            .command_buffer
            .entry(world.id())
            .or_insert_with(|| AtomicRefCell::new(CommandBuffer::new(world)));

        info!("Running");
        let mut borrow = self.run_fn.get_mut();
        borrow.deref_mut().run(
            &mut cmd.get_mut(),
            &mut world_shim,
            resources,
            &mut prepared_queries,
        );
    }
}

/// Supertrait used for defining systems. All wrapper objects for systems implement this trait.
///
/// This trait will generally not be used by users.
pub trait FuncSystemFn {
    type Resources;
    type Queries;

    fn run(
        &mut self,
        commands: &mut CommandBuffer,
        world: &mut SubWorld,
        resources: Self::Resources,
        queries: &mut Self::Queries,
    );
}

pub struct FuncSystemFnWrapper<
    R,
    Q,
    F: FnMut(&mut CommandBuffer, &mut SubWorld, R, &mut Q) + 'static,
>(pub F, pub PhantomData<(R, Q)>);

impl<F, R, Q> FuncSystemFn for FuncSystemFnWrapper<R, Q, F>
where
    F: FnMut(&mut CommandBuffer, &mut SubWorld, R, &mut Q) + 'static,
{
    type Resources = R;
    type Queries = Q;

    fn run(
        &mut self,
        commands: &mut CommandBuffer,
        world: &mut SubWorld,
        resources: Self::Resources,
        queries: &mut Self::Queries,
    ) {
        (self.0)(commands, world, resources, queries);
    }
}
