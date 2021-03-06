# usch - A 🦀 [Demoscene](https://en.wikipedia.org/wiki/Demoscene) framework

Use at your own risk.

```
cargo run --example 01-raymarch
```

It won't be on cargo.io for a long long time, so if you want to play around:

```
[dependencies]
usch = { git = "https://github.com/kwarf/usch.git" }
```

## Hopes and dreams

### Stage one

- ✅ Support single-scene fragment shader demos (see [Kwarf/nothing](https://github.com/Kwarf/nothing) as an example)
- ✅ Live shader reloading
- ✅ Reasonably deterministic audio playback
- ✅ Time-scrubbing

### Stage two
- ❌ Real-time editable sync-tracker
- ❌ Built-in frame capture/pre-render export
- ❌ Multiple scenes
- ❌ OGG streaming (currently entirely decoded as a pre-calc stage)
- ❌ Suspend rendering while state is unchanged (🔋👍)

## FAQ

### I'm getting "cannot find native shaderc library on system; falling back to build from source". What do?

I'm using the [shaderc](https://crates.io/crates/shaderc) crate to compile GLSL shaders into SPIR-V.
This requires the C++ shaderc library, which ships with the [Vulkan SDK](https://www.lunarg.com/vulkan-sdk/).

If you don't want to, or can't build this automatically from source you can install the SDK,
then point to its location in a [config.toml](https://doc.rust-lang.org/cargo/reference/config.html)
file in a location you find appropriate, ex. `%USERPROFILE%\.cargo\config.toml`.

```toml
[env]
VULKAN_SDK = "C:\\VulkanSDK\\1.3.211.0"
```
