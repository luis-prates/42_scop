use std::os::raw::c_void;

use crate::bmp_loader;

pub fn load_texture_bmp(texture_path: &str) -> Result<u32, String> {
    let mut texture_id = 0;

    let img = bmp_loader::open(texture_path)
        .map_err(|error| format!("Failed to open BMP texture '{}': {}", texture_path, error))?;

    let width = img.width;
    let height = img.height;

    // Convert Vec<Pixel> to flat Vec<u8> for OpenGL.
    // OpenGL expects continuous RGB bytes, not a struct array.
    let mut rgb_data: Vec<u8> = Vec::with_capacity((width * height * 3) as usize);
    for pixel in &img.data {
        rgb_data.push(pixel.r);
        rgb_data.push(pixel.g);
        rgb_data.push(pixel.b);
    }

    unsafe {
        gl::GenTextures(1, &mut texture_id);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);

        let mut previous_unpack_alignment: i32 = 0;
        gl::GetIntegerv(gl::UNPACK_ALIGNMENT, &mut previous_unpack_alignment);

        // Tight RGB rows need 1-byte alignment; default OpenGL unpack alignment is 4.
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

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

        gl::PixelStorei(gl::UNPACK_ALIGNMENT, previous_unpack_alignment);

        // Set texture parameters.
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR_MIPMAP_LINEAR as i32,
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    }

    Ok(texture_id)
}
