use std::path::Path;

use bevy::prelude::*;
use bevy::ecs::all_tuples;
use bevy::ecs::system::SystemState;
use bevy::ecs::component::ComponentId;
use bevy::ecs::query::ReadOnlyWorldQuery;
use bevy::scene::DynamicEntity;
use bevy::utils::{HashMap, HashSet};

use thiserror::Error;

/// Error when exporting scene to a file
#[derive(Error, Debug)]
pub enum SceneExportError {
    #[error("Bevy Scene serialization to RON format failed")]
    Ron(#[from] ron::Error),
    #[error("Error writing to output file")]
    Io(#[from] std::io::Error),
}

/// Create a Bevy Dynamic Scene with specific entities and components.
///
/// The two generic parameters are treated the same way as with Bevy `Query`.
///
/// The created scene will include only the entities that match the query,
/// and only the set of components that are included in the query and impl `Reflect`.
///
/// If you want to include all components, try:
///  - [`scene_from_query`]
///
/// If what you need cannot be expressed with just a query,
/// try [`SceneBuilder`].
pub fn scene_from_query_components<Q, F>(
    world: &mut World,
) -> DynamicScene
where
    Q: ComponentList,
    F: ReadOnlyWorldQuery + 'static,
{
    let mut ss = SystemState::<Query<Entity, (Q::QueryFilter, F)>>::new(world);

    let type_registry = world.get_resource::<AppTypeRegistry>()
        .expect("The World provided for scene generation does not contain a TypeRegistry")
        .read();

    let q = ss.get(world);

    let entities = q.iter().map(|entity| {
        let get_reflect_by_id = |id|
            world.components()
                .get_info(id)
                .and_then(|info| type_registry.get(info.type_id().unwrap()))
                .and_then(|reg| reg.data::<ReflectComponent>())
                .and_then(|rc| rc.reflect(world.entity(entity)))
                .map(|c| c.clone_value());

        // TODO: avoid this allocation somehow?
        let mut ids = HashSet::new();
        Q::do_component_ids(world, &mut |id| {ids.insert(id);});

        let components = ids.into_iter()
            .filter_map(get_reflect_by_id)
            .collect();

        DynamicEntity {
            entity: entity.index(),
            components,
        }
    }).collect();

    DynamicScene {
        entities,
    }
}

/// Convenience wrapper for [`scene_from_query_components`] to output to file
///
/// Creates a file in the Bevy Scene RON format. Path should end in `.scn.ron`.
///
/// On success (if both scene generation and file output succeed), will return
/// the generated [`DynamicScene`], just in case you need it.
pub fn scene_file_from_query_components<Q, F>(
    world: &mut World,
    path: impl AsRef<Path>,
) -> Result<DynamicScene, SceneExportError>
where
    Q: ComponentList,
    F: ReadOnlyWorldQuery + 'static,
{
    let scene = scene_from_query_components::<Q, F>(world);
    let type_registry = world.get_resource::<AppTypeRegistry>()
        .expect("The World provided for scene generation does not contain a TypeRegistry");
    let data = scene.serialize_ron(type_registry)?;
    std::fs::write(path, &data)?;
    Ok(scene)
}

/// Convenience wrapper for [`scene_from_query_components`] to add the scene to the app's assets collection
///
/// Returns an asset handle that can be used for spawning the scene, (with [`DynamicSceneBundle`]).
pub fn add_scene_from_query_components<Q, F>(
    world: &mut World,
) -> Handle<DynamicScene>
where
    Q: ComponentList,
    F: ReadOnlyWorldQuery + 'static,
{
    let scene = scene_from_query_components::<Q, F>(world);
    let mut assets = world.get_resource_mut::<Assets<DynamicScene>>()
        .expect("World does not have an Assets<DynamicScene> to add the new scene to");
    assets.add(scene)
}

/// Create a Bevy Dynamic Scene with specific entities.
///
/// The generic parameter is used as a `Query` filter.
///
/// The created scene will include only the entities that match the query
/// filter provided. All components that impl `Reflect` will be included.
///
/// If you only want specific components, try:
///  - [`scene_from_query_components`]
///
/// If what you need cannot be expressed with just a query filter,
/// try [`SceneBuilder`].
pub fn scene_from_query_filter<F>(
    world: &mut World,
) -> DynamicScene
where
    F: ReadOnlyWorldQuery + 'static,
{
    let mut ss = SystemState::<Query<Entity, F>>::new(world);

    let type_registry = world.get_resource::<AppTypeRegistry>()
        .expect("The World provided for scene generation does not contain a TypeRegistry")
        .read();

    let q = ss.get(world);

    let entities = q.iter().map(|entity| {
        let get_reflect_by_id = |id|
            world.components()
                .get_info(id)
                .and_then(|info| type_registry.get(info.type_id().unwrap()))
                .and_then(|reg| reg.data::<ReflectComponent>())
                .and_then(|rc| rc.reflect(world.entity(entity)))
                .map(|c| c.clone_value());

        let components = world.entities()
            .get(entity)
            .and_then(|eloc| world.archetypes().get(eloc.archetype_id))
            .into_iter()
            .flat_map(|a| a.components())
            .filter_map(get_reflect_by_id)
            .collect();

        DynamicEntity {
            entity: entity.index(),
            components,
        }
    }).collect();

    DynamicScene {
        entities,
    }
}

/// Convenience wrapper for [`scene_from_query_filter`] to output to file
///
/// Creates a file in the Bevy Scene RON format. Path should end in `.scn.ron`.
///
/// On success (if both scene generation and file output succeed), will return
/// the generated [`DynamicScene`], just in case you need it.
pub fn scene_file_from_query_filter<F>(
    world: &mut World,
    path: impl AsRef<Path>,
) -> Result<DynamicScene, SceneExportError>
where
    F: ReadOnlyWorldQuery + 'static,
{
    let scene = scene_from_query_filter::<F>(world);
    let type_registry = world.get_resource::<AppTypeRegistry>()
        .expect("The World provided for scene generation does not contain a TypeRegistry");
    let data = scene.serialize_ron(type_registry)?;
    std::fs::write(path, &data)?;
    Ok(scene)
}

/// Convenience wrapper for [`scene_from_query_filter`] to add the scene to the app's assets collection
///
/// Returns an asset handle that can be used for spawning the scene, (with [`DynamicSceneBundle`]).
pub fn add_scene_from_query_filter<F>(
    world: &mut World,
) -> Handle<DynamicScene>
where
    F: ReadOnlyWorldQuery + 'static,
{
    let scene = scene_from_query_filter::<F>(world);
    let mut assets = world.get_resource_mut::<Assets<DynamicScene>>()
        .expect("World does not have an Assets<DynamicScene> to add the new scene to");
    assets.add(scene)
}

enum ComponentSelection {
    All,
    ByIds(HashSet<ComponentId>),
}

/// Flexible tool for creating Bevy scenes
///
/// You can select what entities from your `World` you would like
/// to include in the scene, by adding them using the various methods.
///
/// For each entity, you can choose whether you would like to include
/// all components (that impl `Reflect`) or just a specific set.
///
/// See the documentation of the various methods for more info.
///
/// After you are done adding entities and components, you can call
/// `.build_scene(...)` to create a [`DynamicScene`] with everything
/// that was selected.
pub struct SceneBuilder<'w> {
    world: &'w mut World,
    ec: HashMap<Entity, ComponentSelection>,
    ignored: HashSet<ComponentId>,
}

impl<'w> SceneBuilder<'w> {
    /// Create a new scene builder
    ///
    /// The entities and components of the created scene will come from
    /// the provided `world`.
    pub fn new(world: &'w mut World) -> SceneBuilder<'w> {
        SceneBuilder {
            world,
            ec: Default::default(),
            ignored: Default::default(),
        }
    }

    /// Add components to the set of components to be ignored
    ///
    /// This applies only to entities without explicit component selections.
    ///
    /// If you have explicitly added any of them to specific entities, they
    /// will still be exported to the scene.
    ///
    /// If an entity was added in "all components" mode, then `.build_scene()`
    /// will skip any of these components that it encounters.
    pub fn ignore_components<Q>(&mut self) -> &mut Self
    where
        Q: ComponentList,
    {
        Q::do_component_ids(self.world, &mut |id| {self.ignored.insert(id);});
        self
    }

    /// Add all entities that match the given query filter
    ///
    /// This method allows you to select entities in a way similar to
    /// using Bevy query filters.
    ///
    /// All components of each entity will be included.
    ///
    /// If you want to only include specific components, try:
    ///  - [`add_with_components`]
    pub fn add_from_query_filter<F>(&mut self) -> &mut Self
    where
        F: ReadOnlyWorldQuery + 'static,
    {
        let mut ss = SystemState::<Query<Entity, F>>::new(self.world);
        let q = ss.get(self.world);
        for e in q.iter() {
            self.ec.insert(e, ComponentSelection::All);
        }
        self
    }

    /// Add a specific entity
    ///
    /// The entity ID provided will be added, if it has not been already.
    ///
    /// All components of the entity will be included.
    ///
    /// If you want to only include specific components, try:
    ///  - [`add_components_to_entity`]
    pub fn add_entity(&mut self, e: Entity) -> &mut Self {
        self.ec.insert(e, ComponentSelection::All);
        self
    }

    /// Include the specified components on a given entity ID
    ///
    /// The entity ID provided will be added, if it has not been already.
    ///
    /// The components listed in `Q` will be added its component selection.
    ///
    /// If you want to select all components, try:
    ///  - [`add_entity`]
    pub fn add_components_to_entity<Q>(&mut self, e: Entity) -> &mut Self
    where
        Q: ComponentList,
    {
        if let Some(item) = self.ec.get_mut(&e) {
            if let ComponentSelection::ByIds(c) = item {
                Q::do_component_ids(self.world, &mut |id| {c.insert(id);});
            }
        } else {
            let mut c = HashSet::default();
            Q::do_component_ids(self.world, &mut |id| {c.insert(id);});
            self.ec.insert(e, ComponentSelection::ByIds(c));
        }
        self
    }

    /// Add entities by ID
    ///
    /// The entity IDs provided will be added, if they have not been already.
    ///
    /// All components of each entity will be included.
    ///
    /// If you want to only include specific components, try:
    ///  - [`add_components_to_entities`]
    pub fn add_entities<I>(&mut self, entities: I) -> &mut Self
    where
        I: IntoIterator<Item = Entity>,
    {
        for e in entities {
            self.ec.insert(e, ComponentSelection::All);
        }
        self
    }

    /// Include the specified components to entities with ID
    ///
    /// The entity IDs provided will be added, if they have not been already.
    ///
    /// The components listed in `Q` will be added their component selections.
    ///
    /// If you want to select all components, try:
    ///  - [`add_entities`]
    pub fn add_components_to_entities<I, Q>(&mut self, entities: I) -> &mut Self
    where
        I: IntoIterator<Item = Entity>,
        Q: ComponentList,
    {
        for e in entities {
            if let Some(item) = self.ec.get_mut(&e) {
                if let ComponentSelection::ByIds(c) = item {
                    Q::do_component_ids(self.world, &mut |id| {c.insert(id);});
                }
            } else {
                let mut c = HashSet::default();
                Q::do_component_ids(self.world, &mut |id| {c.insert(id);});
                self.ec.insert(e, ComponentSelection::ByIds(c));
            }
        }
        self
    }

    /// Add specific components to entities that match a query filter
    ///
    /// This method allows you to select entities in a way similar to
    /// using Bevy query filters.
    ///
    /// The components listed in `Q` will be added to each of the entities.
    ///
    /// If you want to select all components, try:
    ///  - [`add_from_query_filter`]
    pub fn add_with_components<Q, F>(&mut self) -> &mut Self
    where
        Q: ComponentList,
        F: ReadOnlyWorldQuery + 'static,
    {
        let mut ss = SystemState::<Query<Entity, (Q::QueryFilter, F)>>::new(self.world);
        let q = ss.get(self.world);
        for e in q.iter() {
            if let Some(item) = self.ec.get_mut(&e) {
                if let ComponentSelection::ByIds(c) = item {
                    Q::do_component_ids(self.world, &mut |id| {c.insert(id);});
                }
            } else {
                let mut c = HashSet::default();
                Q::do_component_ids(self.world, &mut |id| {c.insert(id);});
                self.ec.insert(e, ComponentSelection::ByIds(c));
            }
        }
        self
    }

    /// Build a [`DynamicScene`] with the selected entities and components
    ///
    /// Everything that was added to the builder (using the various `add_*`
    /// methods) will be included in the scene.
    ///
    /// All the relevant data will be copied from the `World` that was provided
    /// when the [`SceneBuilder`] was created.
    pub fn build_scene(&self) -> DynamicScene {
        let type_registry = self.world.get_resource::<AppTypeRegistry>()
            .expect("The World provided to the SceneBuilder does not contain a TypeRegistry")
            .read();

        let entities = self.ec.iter().map(|(entity, csel)| {
            let get_reflect_by_id = |id|
                self.world.components()
                    .get_info(id)
                    .and_then(|info| type_registry.get(info.type_id().unwrap()))
                    .and_then(|reg| reg.data::<ReflectComponent>())
                    .and_then(|rc| rc.reflect(self.world.entity(*entity)))
                    .map(|c| c.clone_value());

            let components = match csel {
                ComponentSelection::All => {
                    self.world.entities()
                        .get(*entity)
                        .and_then(|eloc| self.world.archetypes().get(eloc.archetype_id))
                        .into_iter()
                        .flat_map(|a| a.components())
                        .filter(|id| !self.ignored.contains(&id))
                        .filter_map(get_reflect_by_id)
                        .collect()
                },
                ComponentSelection::ByIds(ids) => {
                    ids.iter()
                        .cloned()
                        .filter_map(get_reflect_by_id)
                        .collect()
                },
            };

            DynamicEntity {
                entity: entity.index(),
                components,
            }
        }).collect();

        DynamicScene {
            entities,
        }
    }

    /// Convenience method: build the scene and serialize to file
    ///
    /// Creates a file in the Bevy Scene RON format. Path should end in `.scn.ron`.
    ///
    /// On success (if both scene generation and file output succeed), will return
    /// the generated [`DynamicScene`], just in case you need it.
    pub fn export_to_file(&self, path: impl AsRef<Path>) -> Result<DynamicScene, SceneExportError> {
        let scene = self.build_scene();
        let type_registry = self.world.get_resource::<AppTypeRegistry>()
            .expect("The World provided to the SceneBuilder does not contain a TypeRegistry");
        let data = scene.serialize_ron(type_registry)?;
        std::fs::write(path, &data)?;
        Ok(scene)
    }

    /// Convenience method: build the scene and add to the app's asset collection
    ///
    /// Returns an asset handle that can be used for spawning the scene, (with [`DynamicSceneBundle`]).
    pub fn build_scene_and_add(&mut self) -> Handle<DynamicScene> {
        let scene = self.build_scene();
        let mut assets = self.world.get_resource_mut::<Assets<DynamicScene>>()
            .expect("World does not have an Assets<DynamicScene> to add the new scene to");
        assets.add(scene)
    }
}

/// Represents a selection of components to export into a scene.
///
/// Works similar to Bevy's queries, but only immutable access
/// (`&T`), optional access (`Option<&T>`), and tuples to combine
/// multiple types, are supported.
pub trait ComponentList {
    type QueryFilter: ReadOnlyWorldQuery + 'static;
    fn do_component_ids<F: FnMut(ComponentId)>(world: &World, f: &mut F);
}

impl<T: Component + Reflect> ComponentList for &T {
    type QueryFilter = With<T>;
    #[inline]
    fn do_component_ids<F: FnMut(ComponentId)>(world: &World, f: &mut F) {
        if let Some(id) = world.component_id::<T>() {
            f(id);
        }
    }
}

impl<T: Component + Reflect> ComponentList for Option<&T> {
    type QueryFilter = ();
    #[inline]
    fn do_component_ids<F: FnMut(ComponentId)>(world: &World, f: &mut F) {
        if let Some(id) = world.component_id::<T>() {
            f(id);
        }
    }
}

macro_rules! componentlist_impl {
    ($($x:ident),*) => {
        impl<$($x: ComponentList),*> ComponentList for ($($x,)*) {
            type QueryFilter = ($($x::QueryFilter,)*);
            #[inline]
            fn do_component_ids<F: FnMut(ComponentId)>(_world: &World, _f: &mut F) {
                $($x::do_component_ids(_world, _f);)*
            }
        }
    };
}

all_tuples!(componentlist_impl, 0, 15, T);

#[cfg(test)]
mod test {
}
