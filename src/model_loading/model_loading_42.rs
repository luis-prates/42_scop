extern crate glfw;

use glfw::fail_on_errors;
use glfw::{Action, Key};

use crate::camera;
use crate::common;
use crate::math;
use crate::model;
use crate::rng::Rng;
use crate::shader;

use self::glfw::Context;

extern crate gl;

use std::ffi::CStr;

use camera::Camera;
use common::process_events;
use math::{Matrix4, Point3, Vector3};
use model::Model;
use shader::Shader;

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;
const TEXTURE_BLEND_SPEED: f32 = 1.5;

pub fn start_renderer(model_path: &str, texture_path: &str) {
    if let Err(e) = run_renderer(model_path, texture_path) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_renderer(model_path: &str, texture_path: &str) -> Result<(), String> {
    let mut camera = Camera {
        position: Point3::new(0.0, 0.0, 3.0),
        ..Camera::default()
    };

    let mut first_mouse = true;
    let mut last_x: f32 = SCR_WIDTH as f32 / 2.0;
    let mut last_y: f32 = SCR_HEIGHT as f32 / 2.0;

    // timing
    let mut last_frame: f32 = 0.0;

    // glfw: initialize and configure
    let mut glfw =
        glfw::init(fail_on_errors!()).map_err(|e| format!("Failed to initialize GLFW: {}", e))?;
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // glfw window creation
    let (mut window, events) = glfw
        .create_window(SCR_WIDTH, SCR_HEIGHT, "42 Scop", glfw::WindowMode::Windowed)
        .ok_or_else(|| "Failed to create GLFW window".to_string())?;

    window.make_current();
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);
    window.set_cursor_mode(glfw::CursorMode::Disabled);

    // gl: load all OpenGL function pointers
    gl::load_with(|symbol| {
        glfw.get_proc_address_raw(symbol)
            .ok_or_else(|| format!("Failed to load OpenGL function: {:?}", symbol))
            .map(|ptr| ptr as *const _)
            .unwrap_or(std::ptr::null())
    });

    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        // Disable face culling to render both front and back faces.
        gl::Disable(gl::CULL_FACE);
    }

    let our_shader = Shader::new(
        "src/shaders/model_loading_42.vs",
        "src/shaders/model_loading_42.fs",
    )?;
    let mut our_model = Model::new(model_path, texture_path)?;

    let mut position = Vector3::new(0.0, 0.0, 0.0);

    // Start in colored view to match the subject semantics: Enter applies texture.
    let mut texture_enabled = false;
    let mut mix_value = 0.0;

    let mut texture_toggle_held = false;
    let mut color_change_held = false;

    while !window.should_close() {
        let current_frame = glfw.get_time() as f32;
        let delta_time = current_frame - last_frame;
        last_frame = current_frame;

        // events
        process_events(
            &events,
            &mut first_mouse,
            &mut last_x,
            &mut last_y,
            &mut camera,
        );

        process_local_input(
            &mut window,
            &mut position,
            delta_time,
            &mut our_model,
            &mut texture_enabled,
            &mut texture_toggle_held,
            &mut color_change_held,
        );

        let target_mix = if texture_enabled { 1.0 } else { 0.0 };
        let blend_step = TEXTURE_BLEND_SPEED * delta_time;
        if mix_value < target_mix {
            mix_value = (mix_value + blend_step).min(target_mix);
        } else if mix_value > target_mix {
            mix_value = (mix_value - blend_step).max(target_mix);
        }

        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            // Activate shader before writing uniforms.
            our_shader.use_program();
            our_shader.set_float(c_str!("mixValue"), mix_value);

            let projection: Matrix4 = Matrix4::perspective(
                camera.zoom,
                SCR_WIDTH as f32 / SCR_HEIGHT as f32,
                0.1,
                100.0,
            );
            let view = camera.get_view_matrix();

            our_shader.set_mat4(c_str!("view"), &view);
            our_shader.set_mat4(c_str!("projection"), &projection);

            // Render model centered around its geometric center.
            let (center_x, center_y, center_z) = our_model.get_center_all_axes();
            let angle = glfw.get_time() as f32 * 50.0;
            let mut model = Matrix4::from_scale(0.2);
            model =
                model * Matrix4::from_translation(Vector3::new(position.x, position.y, position.z));
            model =
                model * Matrix4::from_axis_angle(Vector3::new(0.0, 1.0, 0.0).normalize(), angle);
            model =
                model * Matrix4::from_translation(Vector3::new(-center_x, -center_y, -center_z));

            our_shader.set_mat4(c_str!("model"), &model);
            our_model.draw(&our_shader);
        }

        window.swap_buffers();
        glfw.poll_events();
    }

    Ok(())
}

fn process_local_input(
    window: &mut glfw::Window,
    position: &mut Vector3,
    delta_time: f32,
    our_model: &mut Model,
    texture_enabled: &mut bool,
    texture_toggle_held: &mut bool,
    color_change_held: &mut bool,
) {
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
    if enter_pressed && !*texture_toggle_held {
        *texture_enabled = !*texture_enabled;
    }
    *texture_toggle_held = enter_pressed;

    let color_pressed = window.get_key(Key::K) == Action::Press;
    if color_pressed && !*color_change_held {
        let mut rng = Rng::new();
        our_model.change_color(&Vector3::new(
            rng.gen_range_f32(0.0, 1.1),
            rng.gen_range_f32(0.0, 1.1),
            rng.gen_range_f32(0.0, 1.1),
        ));
    }
    *color_change_held = color_pressed;
}
