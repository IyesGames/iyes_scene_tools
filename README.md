# Helpers for working with Bevy Scenes

## Scene Export

The primary purpose of this crate is to give you a nice API for creating /
exporting Bevy Scenes.

You can now easily create Bevy `DynamicScene`s that include whatever
exact selection of entities and components you want!

The selections can be done with a syntax similar to Bevy Queries.

This create will then copy the relevant data, based on your selections,
from your `World`, and create a scene from it!

```rust
// quick: make a scene with all entities that match a given query filter
// (all components will be included)
let my_scene = scene_from_query_filter::<(
    With<ComponentA>,
    Without<ComponentB>,
)>(&mut world);

// quick: same thing, but only with specific components
let my_scene = scene_from_query_components::<
    // the components to include
    (&ComponentA, &ComponentB),
    // additional filter, to select only specific entities
    (With<IWantInMyScene>, Without<DevOnlyDoNotExport>),
>(&mut world);
```

Then you can just serialize it (say, to create scene asset files), or
spawn / instantiate it, etc…

```rust
// and now you can serialize it if you want
println!("{}", my_scene.serialize_ron(type_registry).unwrap());

// or add it to your app's assets, so you can use it
// (using `ResMut<Assets<DynamicScene>>`)
let handle = assets.add(my_scene);

// spawn it
commands.spawn_bundle(DynamicSceneBundle {
    scene: handle,
    ..default()
});
```

If you want more flexibility, you can use `SceneBuilder`, which lets you
accumulate multiple selections incrementally, and then create a scene with
everything you added.

```rust
let mut builder = SceneBuilder::new(&mut world);

// include entities using query filter:
// all entities with `GameItem`
// all of their components will be included
builder.add_from_query_filter::<With<GameItem>>();

// only specific components for these entities
builder.add_with_components::<
    // the components to select
    (&Transform, &Health),
    // query filter to select entities
    Or<(With<Player>, With<Enemy>)>
>();

// also add some special entities
builder.add_entity(e);
builder.add_entities(&[magic1, magic2, magic3]);
builder.add_components_to_entities::<
    &Transform 
>(special_entities.iter());

// now that we have selected everything, make a scene from it!
let my_scene = builder.build_scene();
```
