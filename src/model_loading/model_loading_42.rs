extern crate glfw;

use glfw::{Action, Key};
use glfw::{Glfw, fail_on_errors};

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

// extern crate image;

// settings
const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

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
    let mut delta_time: f32; // time between current frame and last frame
    let mut last_frame: f32 = 0.0;

    // glfw: initialize and configure
    // ------------------------------
    let mut glfw =
        glfw::init(fail_on_errors!()).map_err(|e| format!("Failed to initialize GLFW: {}", e))?;
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // glfw window creation
    // --------------------
    let (mut window, events) = glfw
        .create_window(SCR_WIDTH, SCR_HEIGHT, "42 Scop", glfw::WindowMode::Windowed)
        .ok_or_else(|| "Failed to create GLFW window".to_string())?;

    window.make_current();
    //window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_scroll_polling(true);

    window.set_cursor_mode(glfw::CursorMode::Disabled);

    // gl: load all OpenGL function pointers
    // ---------------------------------------
    gl::load_with(|symbol| {
        glfw.get_proc_address_raw(symbol)
            .ok_or_else(|| format!("Failed to load OpenGL function: {:?}", symbol))
            .map(|ptr| ptr as *const _)
            .unwrap_or(std::ptr::null())
    });

    let (our_shader, mut our_model) = unsafe {
        gl::Enable(gl::DEPTH_TEST);
        // Disable face culling to render both front and back faces
        // This handles OBJ files with inconsistent winding orders
        gl::Disable(gl::CULL_FACE);

        let our_shader = Shader::new(
            "src/shaders/model_loading_42.vs",
            "src/shaders/model_loading_42.fs",
        )?;

        // load models
        let our_model = Model::new(model_path, texture_path)?;

        // draw in wireframe
        // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

        (our_shader, our_model)
    };

    let mut position = Vector3::new(0.0, 0.0, 0.0);
    let mut use_color = 0;
    let mut mix_value = 0.0;
    let mut last_time: f32 = 0.0;
    let mut new_mix = 0.0;

    // -----------
    while !window.should_close() {
        let current_frame = glfw.get_time() as f32;
        delta_time = current_frame - last_frame;
        last_frame = current_frame;

        // events
        // -----
        process_events(
            &events,
            &mut first_mouse,
            &mut last_x,
            &mut last_y,
            &mut camera,
        );

        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let use_texturing = c_str!("useTexturin");
            let use_mix = c_str!("mixValue");
            let use_new_mix = c_str!("newMix");

            process_local_input(
                &mut window,
                &mut position,
                delta_time,
                &glfw,
                &mut last_time,
                &mut our_model,
                (&mut new_mix, &mut mix_value, &mut use_color),
            );

            if use_color == 1 {
                mix_value += 0.005;
                new_mix += 0.005;
                mix_value = mix_value.clamp(0.0, 1.0);
                new_mix = new_mix.clamp(0.0, 1.0);
            } else {
                mix_value -= 0.005;
                mix_value = mix_value.clamp(0.0, 1.0);
            }

            gl::Uniform1i(
                gl::GetUniformLocation(our_shader.id, use_texturing.as_ptr()),
                use_color,
            );
            gl::Uniform1f(
                gl::GetUniformLocation(our_shader.id, use_mix.as_ptr()),
                mix_value,
            );
            gl::Uniform1f(
                gl::GetUniformLocation(our_shader.id, use_new_mix.as_ptr()),
                new_mix,
            );

            // be sure to activate shader when setting uniforms/drawing objects
            our_shader.use_program();

            let projection: Matrix4 = Matrix4::perspective(
                camera.zoom,
                SCR_WIDTH as f32 / SCR_HEIGHT as f32,
                0.1,
                100.0,
            );
            let view = camera.get_view_matrix();

            // get matrix's uniform location and set matrix
            our_shader.set_mat4(c_str!("view"), &view);
            our_shader.set_mat4(c_str!("projection"), &projection);

            // render the loaded model
            let (center_x, center_y, center_z) = our_model.get_center_all_axes();
            let angle = glfw.get_time() as f32 * 50.0;
            let mut model = Matrix4::from_scale(0.2);
            // let mut model = Matrix4::from_translation(Vector3::new(-center_x, -center_y, -center_z));
            model =
                model * Matrix4::from_translation(Vector3::new(position.x, position.y, position.z));
            model =
                model * Matrix4::from_axis_angle(Vector3::new(0.0, 1.0, 0.0).normalize(), angle);
            model =
                model * Matrix4::from_translation(Vector3::new(-center_x, -center_y, -center_z));

            our_shader.set_mat4(c_str!("model"), &model);
            our_model.draw(&our_shader);

            gl::Uniform1i(
                gl::GetUniformLocation(our_shader.id, use_texturing.as_ptr()),
                1,
            );
            gl::Uniform1f(gl::GetUniformLocation(our_shader.id, use_mix.as_ptr()), 0.0);
        }

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        window.swap_buffers();
        glfw.poll_events();
    }
    Ok(())
}

fn process_local_input(
    window: &mut glfw::Window,
    position: &mut Vector3,
    delta_time: f32,
    glfw: &Glfw,
    last_time: &mut f32,
    our_model: &mut Model,
    (new_mix, mix_value, use_color): (&mut f32, &mut f32, &mut i32),
) {
    let delay_time = 1.0;
    let velocity = 2.5 * delta_time;
    let current_time = glfw.get_time() as f32;
    if window.get_key(Key::Escape) == Action::Press {
        window.set_should_close(true)
    }
    macro_rules! handle_key {
        ($key:ident, $action:ident, $axis:ident, $polarity:expr) => {
            if window.get_key(Key::$key) == Action::$action {
                position.$axis += $polarity * velocity;
            }
        };
    }

    handle_key!(W, Press, y, 1.0);
    handle_key!(S, Press, y, -1.0);
    handle_key!(A, Press, x, -1.0);
    handle_key!(D, Press, x, 1.0);
    handle_key!(Q, Press, z, -1.0);
    handle_key!(E, Press, z, 1.0);

    macro_rules! adjust_mix_value {
        ($key:ident, $sign:expr) => {
            if window.get_key(Key::$key) == Action::Press {
                *mix_value += 0.01 * $sign;
                *mix_value = mix_value.clamp(0.0, 1.0);
                println!("mix value: {}", mix_value);
            }
        };
    }

    adjust_mix_value!(Down, -1.0);
    adjust_mix_value!(Up, 1.0);

    macro_rules! handle_event {
        ($key:ident, $action:ident, $value:expr) => {
            if window.get_key(Key::$key) == Action::$action
                && current_time - *last_time > delay_time
            {
                *$value ^= 1;
                *last_time = current_time;
            }
        };
    }

    handle_event!(Enter, Press, use_color);
    // handle_event!(K, Press, new_mix, "new mix");

    if window.get_key(Key::K) == Action::Press && current_time - *last_time > delay_time {
        *new_mix = 0.0;
        let mut rng = Rng::new();
        our_model.change_color(&Vector3::new(
            rng.gen_range_f32(0.0, 1.1),
            rng.gen_range_f32(0.0, 1.1),
            rng.gen_range_f32(0.0, 1.1),
        ));
        *last_time = current_time;
    }
}
