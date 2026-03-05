# scop_42

A Rust/OpenGL model viewer that loads OBJ geometry, parses MTL material texture references, decodes BMP textures, and renders everything in a GLFW window.

## What this project does

`scop_42` takes two CLI arguments:

1. a model path (`.obj`)
2. a texture path (`.bmp`)

At runtime, it:

- parses geometry + material assignments from OBJ,
- optionally parses MTL texture map declarations,
- builds a scene representation with per-vertex data,
- decodes BMP texture bytes into RGB pixels,
- uploads mesh/texture data to OpenGL,
- draws the model every frame with GLSL shaders and interactive controls.

---

## Quick start

### Build

```bash
cargo build
cargo build --release
```

### Run

```bash
cargo run -- resources/models/42.obj resources/textures/brickwall.bmp
```

Or with `make`:

```bash
make run MODEL=resources/models/42.obj TEXTURE=resources/textures/brickwall.bmp
```

---

## Controls

- `W / S`: translate model on Y
- `A / D`: translate model on X
- `Q / E`: translate model on Z
- `Enter`: toggle texture blend on/off (smooth transition)
- `K`: randomize tint color
- `Up / Down`: adjust generated triplanar texture scale
- `Mouse move`: camera look
- `Mouse wheel`: zoom
- `Esc`: quit

---

## Code walkthrough: from files on disk to pixels on screen

## 1) Entry point and argument validation

- `main()` calls `app::run_from_env()`. The app exits with an error code on failure.
- CLI parsing requires exactly two arguments and validates:
  - file exists,
  - file is a regular file,
  - extension is `.obj` for model and `.bmp` for texture,
  - file can be opened.

Relevant files:

- `src/main.rs`
- `src/app/mod.rs`
- `src/app/cli.rs`

## 2) OBJ parsing (`.obj`)

The OBJ parser (`src/loaders/obj/parse_obj.rs`) reads the file line-by-line and handles directives:

- `v x y z` → append position
- `vn x y z` → append normal
- `vt u v` → append UV
- `f ...` → parse polygon face vertices
- `usemtl name` → change current material group
- `mtllib ...` → record one or more MTL files to load later

### Face token parsing details

Face tokens like `1/2/3`, `1//3`, `-1/-1/-1` are parsed by `parse_face_vertex` in `src/loaders/obj/index.rs`:

- position index is mandatory,
- texcoord and normal indices are optional,
- positive OBJ indices are converted to 0-based,
- negative indices are resolved relative to the end (OBJ semantics),
- index `0` is rejected,
- out-of-bounds references return errors.

### Triangulation behavior

If a face has more than 3 vertices, the loader triangulates it (`triangulate=true` in `SceneModel` builder):

- preferred path: robust ear-clipping triangulation (`src/loaders/obj/triangulate.rs`),
- fallback: triangle fan if polygon is degenerate/self-intersecting/otherwise unsuitable.

If triangulation is disabled and a non-triangle appears, loading fails.

### Mesh assembly strategy

After faces are grouped by material, each triangle vertex is expanded into flat arrays:

- `positions`: always written,
- `normals`: filled from OBJ normal index or `[0,0,0]` if missing,
- `texcoords`: only written when **all** assembled vertices have UVs,
- `indices`: sequential 0..N-1 for the expanded vertex list.

That produces `ObjSceneData { objects, materials }` ready for scene conversion.

## 3) MTL parsing (`.mtl`)

Each `mtllib` file is parsed by `src/loaders/obj/parse_mtl.rs`.

Supported directives:

- `newmtl` → begin material
- `map_Kd` → diffuse texture
- `map_Ks` → specular texture
- `map_Bump` / `bump` → normal texture

The parser stores these into `ObjMaterialData` and silently ignores unknown directives.

## 4) Scene model construction (geometry + texture selection)

`src/scene/model_builder.rs` converts loader output into render-ready `SceneModel` data:

- validates array lengths (positions/normals/UV consistency),
- creates `Vertex` structs (`position`, `normal`, `tex_coords`, colors, etc.),
- sets `has_uv_mapping` depending on whether UVs exist.

### UV fallback when OBJ has no UVs

If OBJ lacks UVs, UVs are generated from XY bounding box normalization:

- `u = (x - min_x)/(max_x - min_x)`
- `v = (y - min_y)/(max_y - min_y)`

(With `0.5` fallback on flat dimensions.)

### Texture path resolution priority

For diffuse texture:

1. if CLI fallback texture argument is non-empty, it is used,
2. else if material has `map_Kd` and it is `.bmp`, resolve relative to model dir,
3. else error.

Specular/normal textures are included only when MTL paths exist and are `.bmp`.

## 5) BMP decoding (`.bmp`)

Texture loading in OpenGL goes through `upload_bmp_texture()` (`src/renderer/texture_gpu.rs`), which calls `bmp::open()` (`src/loaders/bmp/mod.rs`).

Decoder flow (`src/loaders/bmp/decoder.rs`):

1. validate BMP signature (`BM`),
2. read BMP header + DIB header,
3. validate supported formats:
   - versions: mainly v3/v4/v5 headers,
   - bpp: 1/4/8/24,
   - compression: uncompressed only,
4. read palette for indexed formats when needed,
5. decode pixel rows:
   - indexed path (`read_indexes`) for 1/4/8 bpp,
   - direct RGB path (`read_pixels`) for 24 bpp,
6. return `Image` containing width, height, and `Vec<Pixel {r,g,b}>`.

## 6) GPU upload

### Texture upload

`upload_bmp_texture()` converts `Vec<Pixel>` into a packed `Vec<u8>` RGB byte buffer and uploads it with OpenGL:

- `glTexImage2D(..., GL_RGB, GL_UNSIGNED_BYTE, ...)`
- mipmaps generated,
- wrap = `REPEAT`, min/mag filters = linear/mipmap linear,
- unpack alignment temporarily set to `1` to avoid row alignment issues.

### Mesh upload

`MeshGpu::new()` / `setup_mesh()` (`src/renderer/mesh_gpu.rs`) creates VAO/VBO/EBO and defines vertex attributes.
Notably used by shaders:

- location `0`: position
- location `1`: normal
- location `2`: texcoords
- location `6`: `new_color`

Each mesh also carries its textures with semantic kinds (`Diffuse`, `Specular`, `Normal`) for uniform naming.

## 7) Rendering loop and manipulation

`renderer::run()` (`src/renderer/runtime.rs`) manages window/context and draw loop:

- creates GLFW window + OpenGL context,
- compiles/links shaders from `resources/shaders/model.vs` and `model.fs`,
- uploads meshes/textures to GPU,
- each frame:
  - processes events and keyboard input,
  - updates transform, color state, texture blend state,
  - sets uniforms (`mixValue`, `generatedTexScale`, matrices),
  - draws meshes.

### Runtime manipulation

- `K` generates a random base color; scene recoloring updates `new_color` vertex data and re-uploads modified vertex buffers.
- `Enter` toggles textured blend target; `mixValue` interpolates over time for smooth transition.
- Up/Down changes triplanar scale used when UVs are missing.

## 8) Shader output path

### Vertex shader (`resources/shaders/model.vs`)

- passes UVs, object-space position, and `new_color` to fragment shader,
- computes clip-space position using `projection * view * model * vec4(aPos,1)`.

### Fragment shader (`resources/shaders/model.fs`)

- computes `colorView = vec4(newColor,1)`,
- computes `texturedView` via:
  - regular UV sample if mesh has UVs,
  - triplanar sampling from object position if UVs are generated,
- mixes both using `mix(colorView, texturedView, mixValue)`.

So final on-screen color is a blend between procedural face-shaded tint and texture sample.

---

## Project layout

- `src/app/` — app orchestration + CLI validation
- `src/loaders/obj/` — OBJ/MTL parsing + triangulation
- `src/loaders/bmp/` — BMP decoding
- `src/scene/` — scene/vertex construction + color shading helpers
- `src/renderer/` — OpenGL upload + render loop + input processing
- `resources/models/` — sample models/materials
- `resources/textures/` — sample BMP textures
- `resources/shaders/` — GLSL shaders

## Useful commands

```bash
make check
make test
make fmt
make clippy
```
