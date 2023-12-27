use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::mem;
use std::path::Path;
use std::os::raw::c_void;

use crate::bmp_loader;

#[repr(packed)]
struct BMPFileHeader {
    file_type: u16,
    file_size: u32,
    reserved1: u16,
    reserved2: u16,
    offset: u32,
}

impl Default for BMPFileHeader {
    fn default() -> Self {
        BMPFileHeader {
            file_type: 0x4D42,
            file_size: 0,
            reserved1: 0,
            reserved2: 0,
            offset: 0,
        }
    }
}

#[repr(packed)]
struct BMPInfoHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bit_count: u16,
    compression: u32,
    size_image: u32,
    x_pixels_per_meter: i32,
    y_pixels_per_meter: i32,
    colors_used: u32,
    colors_important: u32,
}

impl Default for BMPInfoHeader {
    fn default() -> Self {
        BMPInfoHeader {
            size: 0,
            width: 0,
            height: 0,
            planes: 1,
            bit_count: 0,
            compression: 0,
            size_image: 0,
            x_pixels_per_meter: 0,
            y_pixels_per_meter: 0,
            colors_used: 0,
            colors_important: 0,
        }
    }
}

#[repr(packed)]
struct BMPColorHeader {
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
    alpha_mask: u32,
    color_space_type: u32,
}

impl Default for BMPColorHeader {
    fn default() -> Self {
        BMPColorHeader {
            red_mask: 0x00ff0000,
            green_mask: 0x0000ff00,
            blue_mask: 0x000000ff,
            alpha_mask: 0xff000000,
            color_space_type: 0x73524742,
        }
    }
}

pub struct BMP {
    file_header: BMPFileHeader,
    info_header: BMPInfoHeader,
    color_header: BMPColorHeader,
    data: Vec<u8>,
}

impl BMP {
    fn read(&mut self, filepath: &str) -> io::Result<()> {
        let mut file = File::open(filepath)?;

        unsafe {
            file.read_exact(unsafe {
                std::slice::from_raw_parts_mut(
                    &mut self.file_header as *mut _ as *mut u8,
                    mem::size_of::<BMPFileHeader>(),
                )
            })?;
            if self.file_header.file_type != 0x4D42 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Unrecognized file format"));
            }

            file.read_exact(unsafe {
                std::slice::from_raw_parts_mut(
                    &mut self.info_header as *mut _ as *mut u8,
                    mem::size_of::<BMPInfoHeader>(),
                )
            })?;

            if self.info_header.bit_count == 32 {
                if self.info_header.size >= mem::size_of::<BMPInfoHeader>() as u32
                    + mem::size_of::<BMPColorHeader>() as u32
                {
                    file.read_exact(unsafe {
                        std::slice::from_raw_parts_mut(
                            &mut self.color_header as *mut _ as *mut u8,
                            mem::size_of::<BMPColorHeader>(),
                        )
                    })?;
                    self.check_color_header();
                } else {
                    eprintln!(
                        "Warning! The file \"{}\" does not seem to contain a color header!",
                        filepath
                    );
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unrecognized file format",
                    ));
                }
            }
			// else {
			// 	self.info_header.size = mem::size_of::<BMPInfoHeader>() as u32;
			// 	self.file_header.offset = mem::size_of::<BMPFileHeader>() as u32 + mem::size_of::<BMPInfoHeader>() as u32;
			// }
			// self.file_header.file_size = self.file_header.offset;

            file.seek(SeekFrom::Start(self.file_header.offset as u64))?;

            if self.info_header.bit_count == 32 {
				self.info_header.size = mem::size_of::<BMPInfoHeader>() as u32
                    + mem::size_of::<BMPColorHeader>() as u32;
				self.file_header.offset = mem::size_of::<BMPFileHeader>() as u32
					+ mem::size_of::<BMPInfoHeader>() as u32 + mem::size_of::<BMPColorHeader>() as u32;
			} else {
				self.info_header.size = mem::size_of::<BMPInfoHeader>() as u32;
				self.file_header.offset = mem::size_of::<BMPFileHeader>() as u32
				+ mem::size_of::<BMPInfoHeader>() as u32;
			}
			self.file_header.file_size = self.file_header.offset;

            if self.info_header.height < 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "The program can only treat BMP images with the origin in the bottom left corner",
                ));
            }

			let row_stride = (self.info_header.width * self.info_header.bit_count as i32 / 8) as usize;
			let new_stride = self.make_stride_aligned(4) as usize;
			let mut padding_row: Vec<u8> = vec![0; new_stride - row_stride];
			
			self.data.resize(
				self.info_header.width as usize
					* self.info_header.height as usize
					* self.info_header.bit_count as usize / 8,
				0,
			);

			if self.info_header.width % 4 == 0 {
				file.read_exact(
					&mut self.data
				)?;
			} else {
				for y in 0..self.info_header.height {
					file.read_exact(
						&mut self.data[row_stride * y as usize..row_stride * (y + 1) as usize],
					)?;
					file.read_exact(&mut padding_row)?;
				}
				self.file_header.file_size += self.data.len() as u32
					+ self.info_header.height as u32 * padding_row.len() as u32;
			}
			
			// for y in 0..self.info_header.height as usize {
			// 	file.read_exact(
			// 		&mut self.data[row_stride * y..row_stride * (y + 1)],
			// 	)?;
			// 	file.read_exact(&mut padding_row)?;
			// }
			
			// self.file_header.file_size +=
			// 	self.data.len() as u32 + (self.info_header.height as u32 * padding_row.len() as u32);
			
        }

        Ok(())
    }

    fn make_stride_aligned(&self, align_stride: u32) -> u32 {
        let mut new_stride = self.info_header.width as u32 * (self.info_header.bit_count as u32 / 8);
        while new_stride % align_stride != 0 {
            new_stride += 1;
        }
        new_stride as u32
    }

    fn check_color_header(&self) {
        let expected_color_header = BMPColorHeader {
            red_mask: 0x00ff0000,
            green_mask: 0x0000ff00,
            blue_mask: 0x000000ff,
            alpha_mask: 0xff000000,
            color_space_type: 0x73524742,
        };

        if self.color_header.red_mask != expected_color_header.red_mask
            || self.color_header.green_mask != expected_color_header.green_mask
            || self.color_header.blue_mask != expected_color_header.blue_mask
            || self.color_header.alpha_mask != expected_color_header.alpha_mask
        {
            panic!("Unexpected color mask format! The program expects the pixel data to be in the BGRA format");
        }

        if self.color_header.color_space_type != expected_color_header.color_space_type {
            panic!("Unexpected color space type! The program expects sRGB values");
        }
    }
}

impl BMP {
    pub fn new(filepath: &str) -> io::Result<Self> {
        let mut bmp = BMP {
            file_header: Default::default(),
            info_header: Default::default(),
            color_header: Default::default(),
            data: Vec::new(),
        };

        bmp.read(filepath)?;

        Ok(bmp)
    }
}

pub unsafe fn load_texture_bmp(path: &str) -> u32 {
    let mut texture_id = 0;

    // Open BMP file
	let img = bmp_loader::open(path).unwrap_or_else(|e| {
		panic!("Failed to open: {}", e);
	});

    // let img = BMP::new(path).expect("Texture failed to load");
	// let width = img.info_header.width as i32;
	// let height = img.info_header.height as i32;
	// let data_size = (width * height * (img.info_header.bit_count as i32 / 8)) as usize;


	let width = img.width;
	let height = img.height;
	let data_size = width * height * img.dib_header.bits_per_pixel as u32 / 8;

	let data = img.data.clone();

	println!("Data size is {}", data.len() == data_size as usize);

    // Generate OpenGL texture
    gl::GenTextures(1, &mut texture_id);
    gl::BindTexture(gl::TEXTURE_2D, texture_id);
	//println!("Test: {:?}", &data[0] as *const u8 as *const c_void);
	println!("Test2: {:?}", data.as_ptr() as *const c_void);
    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, width as i32, height as i32, 0, gl::RGB, gl::UNSIGNED_BYTE, data.as_ptr() as *const c_void);
    gl::GenerateMipmap(gl::TEXTURE_2D);

    // Set texture parameters
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

    texture_id
}