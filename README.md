# Bevy Facial Mocap

This is a project build for storyteller.ai, to provide support for importing ARKit data into bevy, and using it to drive shape keys.

## Install

Add this to your dependencies alongside bevy:
```toml
bevy = "0.12"
bevy-facial-mocap = { git = "https://github.com/caladluin/bevy-facial-mocap" } #TODO - once ownership of the repo transfers, set this to the correct repository
```

If you add the `file_watcher` feature to bevy, this also supports hot reloading:
```toml
bevy = { version = "0.12", default-features = true, features = ["file_watcher"]}
```


## Use

Import the asset loader:

```rust
    app
        .init_asset_loader::<CSVAnimationLoader>()
```

and in code, you can load assets just like any normal animation clip:

```rust
let handle = asset_server.load("recorded_test.morph_anim.json");
let mut player = AnimationPlayer::default();
player.play(handle.clone()).repeat();
```


## Asset Preparation

To load a recording, you need 3 files:
- a `.csv` with the ARKit data
- a `.json` file with the mapping info
- a `.morph_anim.json` file that points to the `.csv`, the mapping file, and provides the name of the animation root. Ideally, it would also provide an FPS - though that can be computed if it is an integer value (so not for 60 or 30, but not for 29.97)

### .morph_anim.json structure
This is the structure of the animation file itself:

```json
{
    "csv": "./the_relative/path_to_the.csv",
    "root_name": "pascal_Head_or_whatever_root_entity_you_want_here",
    "fps": 4,
    "mapping": "the_relative/path_to_the/mapping.json"
}
```

### Mapping .json structure

The mapping .json is a json list/array. Each item is a sub-list where the first element is the mapping, and the second element is the name of the morph in the CSV file.

For example:

```json
[
    ["eyeBlinkLeft", "EyeBlinkLeft"],
    ["eyeLookDownLeft", "EyeLookDownLeft"]
]
```

For morphs, the sub lists need to be in the same order as the morphs in the GLTF files.

For bones, the first element of the sub-list is the mapping information. It provides the path to the bone (from root), the axis of the transform, and an optional offset or scale:

```json
    [{
        "path": ["pascal_Head"],
        "axis": "Y",
        "scale": 0.4
    }, "HeadYaw"],
    [{
        "path": ["pascal_Head"],
        "axis": "X",
        "offset": -45
    }, "HeadPitch"],
    [{
        "path": ["pascal_Head"],
        "axis": [0, 0, 1]
    }, "HeadRoll"]
```

The [example_mapping.json](assets/example_mapping.json) file contains a full example of the mapping. And the [simple example](examples/simple_example.rs) contains a full code sample you can try by running `cargo run --example simple_example`.