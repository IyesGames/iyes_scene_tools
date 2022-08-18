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
    // (require A and B, only select entities that have them)
    // (C is optional, include it if it is present)
    (&ComponentA, &ComponentB, Option<&ComponentC>),
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

// now that we have selected everything, make a scene from it!
let my_scene = builder.build_scene();
```

### Warning

**Warning!** You *must* ensure that your component types:
 - impl `Reflect`
 - reflect `Component`
 - be registered in the type registry

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

This is incredibly unfortunate, but Bevy maintainers decided to omit it from
the release, because the release was late behind schedule.

Enum reflection support was merged into Bevy shortly after the release.

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
