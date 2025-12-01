use std::os::raw::c_void;

use crate::bmp_loader;

pub unsafe fn load_texture_bmp(texture_path: &str) -> u32 {
    let mut texture_id = 0;

    // Open BMP file
    println!("Loading texture from BMP file: {}", texture_path);
    let img = bmp_loader::open(texture_path).unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });

    let width = img.width;
    let height = img.height;

    // Convert Vec<Pixel> to flat Vec<u8> for OpenGL
    // OpenGL expects continuous RGB bytes, not a struct array
    let mut rgb_data: Vec<u8> = Vec::with_capacity((width * height * 3) as usize);
    for pixel in &img.data {
        rgb_data.push(pixel.r);
        rgb_data.push(pixel.g);
        rgb_data.push(pixel.b);
    }

    // Generate OpenGL texture
    unsafe {
        gl::GenTextures(1, &mut texture_id);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            width as i32,
            height as i32,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            rgb_data.as_ptr() as *const c_void,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);

        // Set texture parameters
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR_MIPMAP_LINEAR as i32,
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    };

    texture_id
}
