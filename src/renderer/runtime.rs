extern crate glfw;

use std::collections::HashMap;

use glfw::fail_on_errors;
use glfw::{Action, Key};

use crate::camera::Camera;
use crate::math::{Matrix4, Point3, Vector3};
use crate::renderer::input_events::process_events;
use crate::renderer::mesh_gpu::{GpuTexture, MeshGpu};
use crate::renderer::shader_program::ShaderProgram;
use crate::renderer::texture_gpu::upload_bmp_texture;
use crate::rng::Rng;
use crate::scene::SceneModel;

use self::glfw::Context;

extern crate gl;

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;
const TEXTURE_BLEND_SPEED: f32 = 1.5;
const DEFAULT_GENERATED_TEX_SCALE: f32 = 2.0;
const GENERATED_TEX_SCALE_STEP: f32 = 0.25;
const GENERATED_TEX_SCALE_MIN: f32 = 0.25;
const GENERATED_TEX_SCALE_MAX: f32 = 16.0;

struct InputState {
    texture_enabled: bool,
    texture_toggle_held: bool,
    color_change_held: bool,
    generated_tex_scale: f32,
    increase_scale_held: bool,
    decrease_scale_held: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            texture_enabled: false,
            texture_toggle_held: false,
            color_change_held: false,
            generated_tex_scale: DEFAULT_GENERATED_TEX_SCALE,
            increase_scale_held: false,
            decrease_scale_held: false,
        }
    }
}

pub fn run(mut scene_model: SceneModel) -> Result<(), String> {
    let mut camera = Camera {
        position: Point3::new(0.0, 0.0, 3.0),
        ..Camera::default()
    };

    let mut first_mouse = true;
    let mut last_x: f32 = SCR_WIDTH as f32 / 2.0;
    let mut last_y: f32 = SCR_HEIGHT as f32 / 2.0;
    let mut last_frame: f32 = 0.0;

    let mut glfw =
        glfw::init(fail_on_errors!()).map_err(|e| format!("Failed to initialize GLFW: {}", e))?;
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    let (mut window, events) = glfw
        .create_window(SCR_WIDTH, SCR_HEIGHT, "42 Scop", glfw::WindowMode::Windowed)
        .ok_or_else(|| "Failed to create GLFW window".to_string())?;

    window.make_current();
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);
    window.set_cursor_mode(glfw::CursorMode::Disabled);

    gl::load_with(|symbol| {
        glfw.get_proc_address_raw(symbol)
            .map(|ptr| ptr as *const _)
            .unwrap_or(std::ptr::null())
    });

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::Disable(gl::CULL_FACE);
    }

    let shader = ShaderProgram::new("resources/shaders/model.vs", "resources/shaders/model.fs")?;
    let mut gpu_meshes = build_gpu_meshes(&scene_model)?;

    let mut position = Vector3::new(0.0, 0.0, 0.0);
    let mut input_state = InputState::default();
    let mut mix_value = 0.0;

    while !window.should_close() {
        let current_frame = glfw.get_time() as f32;
        let delta_time = current_frame - last_frame;
        last_frame = current_frame;

        process_events(
            &events,
            &mut first_mouse,
            &mut last_x,
            &mut last_y,
            &mut camera,
        );

        if let Some(new_color) =
            process_local_input(&mut window, &mut position, delta_time, &mut input_state)
        {
            scene_model.change_color(&new_color);
            sync_gpu_vertices(&scene_model, &mut gpu_meshes)?;
        }

        let target_mix = if input_state.texture_enabled {
            1.0
        } else {
            0.0
        };
        let blend_step = TEXTURE_BLEND_SPEED * delta_time;
        if mix_value < target_mix {
            mix_value = (mix_value + blend_step).min(target_mix);
        } else if mix_value > target_mix {
            mix_value = (mix_value - blend_step).max(target_mix);
        }

        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        shader.bind();
        shader.set_float(c_str!("mixValue"), mix_value);
        shader.set_float(c_str!("generatedTexScale"), input_state.generated_tex_scale);

        let (framebuffer_width, framebuffer_height) = window.get_framebuffer_size();
        let clamped_height = framebuffer_height.max(1);
        let aspect_ratio = framebuffer_width.max(1) as f32 / clamped_height as f32;
        let projection: Matrix4 = Matrix4::perspective(camera.zoom, aspect_ratio, 0.1, 100.0);
        let view = camera.get_view_matrix();

        shader.set_mat4(c_str!("view"), &view);
        shader.set_mat4(c_str!("projection"), &projection);

        let (center_x, center_y, center_z) = scene_model.get_center_all_axes();
        let angle = glfw.get_time() as f32 * 50.0;
        let mut model = Matrix4::from_scale(0.2);
        model = model * Matrix4::from_translation(Vector3::new(position.x, position.y, position.z));
        model = model * Matrix4::from_axis_angle(Vector3::unit_y(), angle);
        model = model * Matrix4::from_translation(Vector3::new(-center_x, -center_y, -center_z));

        shader.set_mat4(c_str!("model"), &model);
        for mesh in &gpu_meshes {
            mesh.draw(&shader);
        }

        window.swap_buffers();
        glfw.poll_events();
    }

    Ok(())
}

fn build_gpu_meshes(scene_model: &SceneModel) -> Result<Vec<MeshGpu>, String> {
    let mut texture_cache: HashMap<String, u32> = HashMap::new();
    let mut gpu_meshes = Vec::with_capacity(scene_model.meshes.len());

    for scene_mesh in &scene_model.meshes {
        let mut textures = Vec::with_capacity(scene_mesh.textures.len());
        for texture in &scene_mesh.textures {
            let id = if let Some(existing) = texture_cache.get(&texture.path) {
                *existing
            } else {
                let uploaded = upload_bmp_texture(&texture.path)?;
                texture_cache.insert(texture.path.clone(), uploaded);
                uploaded
            };

            textures.push(GpuTexture {
                id,
                kind: texture.kind.clone(),
            });
        }

        let mesh = MeshGpu::new(
            scene_mesh.vertices.clone(),
            scene_mesh.indices.clone(),
            textures,
            scene_mesh.has_uv_mapping,
        )?;
        gpu_meshes.push(mesh);
    }

    Ok(gpu_meshes)
}

fn sync_gpu_vertices(scene_model: &SceneModel, gpu_meshes: &mut [MeshGpu]) -> Result<(), String> {
    if scene_model.meshes.len() != gpu_meshes.len() {
        return Err("Internal renderer error: scene/gpu mesh count mismatch".to_string());
    }

    for (scene_mesh, gpu_mesh) in scene_model.meshes.iter().zip(gpu_meshes.iter_mut()) {
        gpu_mesh.update_vertices(&scene_mesh.vertices);
    }

    Ok(())
}

fn process_local_input(
    window: &mut glfw::Window,
    position: &mut Vector3,
    delta_time: f32,
    input_state: &mut InputState,
) -> Option<Vector3> {
    let velocity = 2.5 * delta_time;

    if window.get_key(Key::Escape) == Action::Press {
        window.set_should_close(true)
    }
    if window.get_key(Key::W) == Action::Press {
        position.y += velocity;
    }
    if window.get_key(Key::S) == Action::Press {
        position.y -= velocity;
    }
    if window.get_key(Key::A) == Action::Press {
        position.x -= velocity;
    }
    if window.get_key(Key::D) == Action::Press {
        position.x += velocity;
    }
    if window.get_key(Key::Q) == Action::Press {
        position.z -= velocity;
    }
    if window.get_key(Key::E) == Action::Press {
        position.z += velocity;
    }

    let enter_pressed = window.get_key(Key::Enter) == Action::Press;
    if enter_pressed && !input_state.texture_toggle_held {
        input_state.texture_enabled = !input_state.texture_enabled;
    }
    input_state.texture_toggle_held = enter_pressed;

    let color_pressed = window.get_key(Key::K) == Action::Press;
    let color_change = if color_pressed && !input_state.color_change_held {
        let mut rng = Rng::new();
        Some(Vector3::new(
            rng.gen_range_f32(0.0, 1.1),
            rng.gen_range_f32(0.0, 1.1),
            rng.gen_range_f32(0.0, 1.1),
        ))
    } else {
        None
    };
    input_state.color_change_held = color_pressed;

    let up_pressed = window.get_key(Key::Up) == Action::Press;
    if up_pressed && !input_state.increase_scale_held {
        input_state.generated_tex_scale = (input_state.generated_tex_scale
            + GENERATED_TEX_SCALE_STEP)
            .clamp(GENERATED_TEX_SCALE_MIN, GENERATED_TEX_SCALE_MAX);
    }
    input_state.increase_scale_held = up_pressed;

    let down_pressed = window.get_key(Key::Down) == Action::Press;
    if down_pressed && !input_state.decrease_scale_held {
        input_state.generated_tex_scale = (input_state.generated_tex_scale
            - GENERATED_TEX_SCALE_STEP)
            .clamp(GENERATED_TEX_SCALE_MIN, GENERATED_TEX_SCALE_MAX);
    }
    input_state.decrease_scale_held = down_pressed;

    color_change
}
