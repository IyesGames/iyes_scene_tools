# Helpers for working with Bevy Scenes

Version compatibility table:

|Bevy Version|Crate Version|
|------------|-------------|
|`0.8`       |`0.1`        |

## Known Issues

Bevy 0.8 has some reflection issues, see here for solutions:
 - [`enum` support](#enum-support)
 - [error: missing type registration for Affine3A](#math-types)

## What is this about?

For the uninitiated: Bevy Scenes are a way to store some predefined Bevy ECS
data (arbitrary entities with components on them) and be able to instantiate
them later, as many times as you want!

You can use Scenes for many use cases:
 - Loading your game levels/maps (or parts of them)
 - Preconfigured game units/modules (some other engines call this "prefabs")
 - Saving game state
 - …

Until now, creating Bevy scenes, and working with the Bevy scene format,
was very unapproachable. While Bevy makes it easy to use existing scenes in
your game (just spawn them with [`DynamicSceneBundle`]), there was no easy
way to create them. Bevy offers nothing built-in for easily exporting things
into a scene, and no APIs to help you create your scenes.

Thanks to this crate, you can now easily create your own scenes, containing
whatever you want, by exporting a custom selection of things from any Bevy app!

## Scene Export

You can create Bevy [`DynamicScene`]s that include whatever exact selection
of entities and components you want!

The selections can be done with a syntax similar to Bevy Queries.

This create will then copy the relevant data, based on your selections,
from your `World`, and create a scene from it!

There are two "modes" for component selection:
 - "all components": when you just select entities, without specifying components.
   When generating the scene, each entity will be scanned to autodetect
   all [compatible](#warning) components and include them in the scene
 - "explicit component list": you specify exactly what components to include
   (they may be required or optional), and only those will be exported
   ([incompatible types will be skipped](#warning))

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
    // (require A and B, only select entities that have them)
    // (C is optional, include it if it is present)
    (&ComponentA, &ComponentB, Option<&ComponentC>),
    // additional filter, to select only specific entities
    (With<IWantInMyScene>, Without<DevOnlyDoNotExport>),
>(&mut world);
```

If you want more flexibility, you can use [`SceneBuilder`], which lets you
accumulate multiple selections incrementally, and then create a scene with
everything you added. The component selection can be controlled with
per-entity granularity.

```rust
let mut builder = SceneBuilder::new(&mut world);

// include entities using query filter:
// all entities with `GameItem`
// all of their components will be included
builder.add_from_query_filter::<With<GameItem>>();

// only specific components for these entities
builder.add_with_components::<
    // the components to select
    (&Transform, &Health, &BaseStats, Option<&SpecialAbility>),
    // query filter to select entities
    Or<(With<Player>, With<Enemy>)>
>();

// also add some special entities
builder.add_entity(e);
builder.add_entities(&[magic1, magic2, magic3]);
builder.add_components_to_entities::<
    &Transform 
>(special_entities.iter());

// we can ignore some components;
// they will never be implicitly included, unless they were
// explicitly selected for specific entities
builder.ignore_components::<(&GlobalTransform, &ComputedVisibility)>();

// now that we have selected everything, make a scene from it!
let my_scene = builder.build_scene();
```

### Exporting to scene asset files

The above examples will create a [`DynamicScene`] instance. However, if you
are simply interested in creating asset files, there are convenience methods
for exporting directly to a Bevy Scene RON Asset file:

```rust
// the standalone (simple) functions:

// like `scene_from_query_components`, but takes a file path
scene_file_from_query_components::</* … */>(world, "my_scene.scn.ron")
    .expect("Scene file output failed");

// like `scene_from_query_filter`, but takes a file path
scene_file_from_query_filter::</* … */>(world, "my_scene2.scn.ron")
    .expect("Scene file output failed");

// for `SceneBuilder`:
let mut builder = SceneBuilder::new(world);
// ... add stuff ...
// instead of `.build_scene()`:
builder.export_to_file("fancy_scene.scn.ron")
    .expect("Scene file output failed");
```

All of the above methods also return the [`DynamicScene`] in the `Ok` result,
if the export was successful, in case you also want to do anything else with
the generated scene.

If you prefer not to use the convenience file export methods, you can output
to a scene asset file manually like this:

```rust
// create the scene, using any of the methods shown before
let my_scene = /* ... */;
// need the type registry
let type_registry = world.resource::<TypeRegistry>();
// output the contents as a String
let data = my_scene.serialize_ron(type_registry)
    .expect("Scene serialization failed");
// create a scene file (ending in `.scn.ron`)
std::fs::write("file.scn.ron", &data)
    .expect("Writing to file failed");
```

### Directly using a generated scene

If you want to generate a scene and use it straight away, without
exporting/loading asset files, here is how.

To use the generated scene in your app, it needs to be added to the app's
assets (the `Assets<DynamicScene>` resource), to get a handle.

There are convenience methods to do this for you, which return
`Handle<DynamicScene>` instead of the bare `DynamicScene`.

```rust
// the standalone (simple) functions:

// like `scene_from_query_components`, but adds it to the app for you
let handle = add_scene_from_query_components::</* … */>(world);

// like `scene_from_query_filter`, but adds it to the app for you
let handle = add_scene_from_query_filter::</* … */>(world);

// for `SceneBuilder`:
let mut builder = SceneBuilder::new(world);
// … add stuff …
// instead of `.build_scene()`:
let handle = builder.build_scene_and_add();
```

If you want to do it manually without the helper functions:

```rust
// get the `Assets<DynamicScene>` resource:
// (if we are in an exclusive system)
let mut assets = world.resource_mut::<Assets<DynamicScene>>();
// (in a regular system, you can use `ResMut<Assets<DynamicScene>>`)

// add it
let handle = assets.add(my_scene);
```

Later, you can spawn your scene from anywhere.

From a regular system:

```rust
commands.spawn_bundle(DynamicSceneBundle {
    scene: handle,
    ..default()
});
```

With direct World access:

```rust
world.spawn().insert_bundle(DynamicSceneBundle {
    scene: handle,
    ..default()
});
```

### Warning

**Warning!** You *must* ensure that your component types:
 - impl `Reflect`
 - reflect `Component`
 - are registered in the type registry

Otherwise, they will be silently ignored, and will be missing from your scene!

If you are serializing your scenes to asset files, you probably also want
`FromReflect`, or otherwise you will not be able to load your scenes later!

```rust
#[derive(Component, Default, Reflect, FromReflect)]
#[reflect(Component)]
struct MyComponent;
```

(note: Bevy requires either a `FromWorld` or a `Default` impl, to derive `Reflect`)

```rust
app.register_type::<MyComponent>();
```

This is required boilerplate, for all components that you want to use
with scenes! Otherwise, things will silently not work.

## Issues with Bevy 0.8

### Math Types

If you are exporting entities with `GlobalTransform` components, you will
get an error about a missing type registration for `Affine3A`.

Add this to workaround the issue:

```rust
app.register_type::<bevy::math::Affine3A>();
```

### Enum Support

If you are using Bevy release 0.8, note that it is missing support for
reflecting `enum`s. Many common component types are Rust `enum`s, so that
greatly limits what kinds of entities/data you can have in your scenes.

Bevy maintainers decided to omit it from 0.8, because the release was late
behind schedule. Enum reflection support was merged into Bevy shortly after
the release.

If you use Bevy `main`, it is supported.

Otherwise, if you want to use Bevy release 0.8, but add enum support, you could:
 - fork bevy (just locally clone the repo, or fork on github)
 - point it at the `v0.8.0` tag
 - cherry-pick commit `15826d6`
 - add a `patch` section to your `Cargo.toml`,
   so that 3rd-party plugins (incl this crate) use your Bevy

(All your 3rd-party plugins should still be compatible. This change is unlikely
to break anything.)

Example:

```sh
git clone https://github.com/bevyengine/bevy # (or your fork URL)
cd bevy
git checkout v0.8.0
git cherry-pick 15826d6
```

In your `Cargo.toml`:

```toml
[patch.crates-io]
bevy = { path = "../bevy" }

# for some other plugins, you might have to patch individual bevy crates:
bevy_ecs = { path = "../bevy/crates/bevy_ecs" }
bevy_app = { path = "../bevy/crates/bevy_app" }
bevy_time = { path = "../bevy/crates/bevy_time" }
bevy_utils = { path = "../bevy/crates/bevy_utils" }
bevy_asset = { path = "../bevy/crates/bevy_asset" }
# … and any others (refer to your dependencies' Cargo.toml) …
```

Alternatively, if you'd like, I also offer a 0.8-compatible branch with
reflection improvements, which has the above already set up for you:

```sh
git clone https://github.com/IyesGames/bevy
git checkout 0.8+reflect
```

or to use it directly from cargo:

```toml
[patch.crates-io]
bevy = { git = "https://github.com/IyesGames/bevy", branch = "0.8+reflect" }
```

## "Blueprints" Pattern

This is a recommendation for how to make your workflow more flexible, and
get the most usefulness out of Bevy scenes.

---

There are many component types in Bevy that represent internal state
computed at runtime, such as: `GlobalTransform`, `ComputedVisibility`,
`Interaction`, etc….

Their values don't need to be persisted in scenes. You might want to omit
them. This will also help your scenes be less bloated.

You might also want to omit other components of your choice, if you prefer
to set them up using code, or initialize them to defaults.

```rust
let mut builder = SceneBuilder::new(world);

// add our game entities
builder.add_from_query_filter::<With<Enemy>>();
builder.add_from_query_filter::<With<Player>>();
builder.add_from_query_filter::<With<Powerup>>();
// …

// for our UI Nodes, only persist hierarchy + `Style`, `UiColor`, `Text`, `Button`
builder.add_with_components::<
    (
      (Option<&Parent>, Option<&Children>),
      (&Style, &UiColor, Option<&Text>, Option<&Button>),
    ),
    With<Node>
>();

// never include these components in any entity
builder.ignore_components::<
    (&GlobalTransform, &Visibility, &ComputedVisibility, &CalculatedSize)
>();

let my_scene = builder.build_scene();
```

---

If you are creating such a "sparse" scene (we can call it "blueprint"),
that only has some of the components and is missing others, you can write
some code to populate the entities to "complete" their setup.

This is easily done using a system with an `Added` query filter. This way,
you detect when such entities are spawned into the world, and you can do
any additional setup on them using code.

```rust
// ensure everything with a transform has all the transform/visibility stuff
fn setup_spatial(
    mut commands: Commands,
    // detect anything that was just added and needs setup
    q_new: Query<
        (Entity, &Transform),
        (Added<Transform>, Without<GlobalTransform>)
    >,
) {
    for (e, transform) in q_new.iter() {
        commands.entity(e).insert_bundle(SpatialBundle {
            // preserve the transform
            transform,
            ..Default::default()
        });
    }
}

/// complete the setup of our UI
/// (btw, this could be the starting point for the development
/// of a nice automatic theming system ;) hehe)
fn setup_ui(
    mut commands: Commands,
    // detect anything that was just added and needs setup
    q_new: Query<
        (Entity, &Style, Option<&UiColor>, Option<&Text>, Option<&Button>),
        (Added<Style>, Without<Node>)
    >,
) {
    for (e, style, color, text, button) in q_new.iter() {
        if let Some(text) = text {
            commands.entity(e).insert_bundle(TextBundle {
                text: text.clone(),
                style: style.clone(),
                ..Default::default()
            });
        } else if let Some(_button) = button {
            // (`Button` is just a marker)
            commands.entity(e).insert_bundle(ButtonBundle {
                style: style.clone(),
                color: color.cloned().unwrap_or(UiColor(Color::NONE)),
                ..Default::default()
            });
        } else {
            // this is a generic ui node
            commands.entity(e).insert_bundle(NodeBundle {
                style: style.clone(),
                color: color.cloned().unwrap_or(UiColor(Color::NONE)),
                ..Default::default()
            });
        }
    }
}
```


[`DynamicScene`]: https://docs.rs/bevy/latest/bevy/scene/struct.DynamicScene.html
[`DynamicSceneBundle`]: https://docs.rs/bevy/latest/bevy/scene/struct.DynamicSceneBundle.html
[`SceneBuilder`]: https://docs.rs/iyes_scene_tools/latest/iyes_scene_tools/struct.SceneBuilder.html
