# Helpers for working with Bevy Scenes

Version compatibility table:

|Bevy Version|Crate Version|
|------------|-------------|
|`0.8`       |`0.1`        |

## What is this about?

For the uninitiated: Bevy Scenes are a way to store some predefined Bevy ECS
data (arbitrary entities with components on them) and be able to instantiate
them later, as many times as you want!

You can use Scenes for many use cases:
 - Loading your game levels/maps (or parts of them)
 - Preconfigured game units/modules (some other engines call this "prefabs")
 - Saving game state
 - …

Until now, creating Bevy scenes, and working with the Bevy
scene format, was very unapproachable. While Bevy makes
it easy to use existing scenes in your game (just spawn them with
[`DynamicSceneBundle`](https://docs.rs/bevy/latest/bevy/scene/struct.DynamicSceneBundle.html)),
there was no easy way to create them. Bevy offers nothing built-in for easily
exporting things into a scene, and no APIs to help you create your scenes.

Thanks to this crate, you can now easily create your own scenes, containing
whatever you want, by exporting a custom selection of things from any Bevy app!

## Scene Export

You can create Bevy [`DynamicScene`](https://docs.rs/bevy/latest/bevy/scene/struct.DynamicScene.html)s
that include whatever exact selection of entities and components you want!

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

If you would like to serialize it (say, to create scene asset files):

```rust
// need the type registry
let type_registry = world.resource::<TypeRegistry>();

// output the contents as a String
let data = my_scene.serialize_ron(type_registry).unwrap();

// create a scene file (ending in `.scn.ron`)
match std::fs::write(&path, &data) {
    Ok(()) => info!("Scene Saved to {:?}", path),
    Err(e) => error!("Could not save scene to {:?}: {}", path, e),
}
```

Or if you just want to use it directly:

```rust
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

// for our UI Nodes, only persist `Style`, `UiColor`, `Text`, `Button`
builder.add_with_components::<
    (&Style, &UiColor, Option<&Text>, Option<&Button>),
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
