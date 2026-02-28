// #![deny(warnings)]
// #![cfg_attr(test, deny(warnings))]
#![allow(dead_code)]

use std::convert::AsRef;
use std::fmt;
use std::fs;
use std::io::{Cursor, Read};
use std::iter::Iterator;
use std::path::Path;

use crate::loaders::bmp::decoder;

// Expose decoder's public types, structs, and enums
pub use decoder::BmpResult;

/// Macro to generate a `Pixel` from `r`, `g` and `b` values.
#[macro_export]
macro_rules! px {
    ($r:expr, $g:expr, $b:expr) => {
        Pixel {
            r: $r as u8,
            g: $g as u8,
            b: $b as u8,
        }
    };
}

#[macro_export]
macro_rules! file_size {
    ($bpp:expr, $width:expr, $height:expr) => {{
        let header_size = 2 + 12 + 40;
        // find row size in bytes, round up to 4 bytes (padding)
        let row_size = (($bpp as f32 * $width as f32 + 31.0) / 32.0).floor() as u32 * 4;
        (header_size as u32, $height as u32 * row_size)
    }};
}

/// Common color constants accessible by names.
/// The pixel data used in the `Image`.
/// It has three values for the `red`, `blue` and `green` color channels, respectively.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Pixel {
    /// Creates a new `Pixel`.
    pub fn new(r: u8, g: u8, b: u8) -> Pixel {
        Pixel { r, g, b }
    }
}

/// Displays the rgb values as an rgb color triple
impl fmt::Display for Pixel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rgb({}, {}, {})", self.r, self.g, self.b)
    }
}

/// Displays the rgb values as an upper-case 24-bit hexadecimal number
impl fmt::UpperHex for Pixel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// Displays the rgb values as a lower-case 24-bit hexadecimal number
impl fmt::LowerHex for Pixel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BmpVersion {
    Two,
    Three,
    ThreeNT,
    Four,
    Five,
}

impl BmpVersion {
    pub fn from_dib_header(dib_header: &BmpDibHeader) -> Option<BmpVersion> {
        match dib_header.header_size {
            12 => Some(BmpVersion::Two),
            40 if dib_header.compress_type == 3 => Some(BmpVersion::ThreeNT),
            40 => Some(BmpVersion::Three),
            108 => Some(BmpVersion::Four),
            124 => Some(BmpVersion::Five),
            _ => None,
        }
    }
}

impl AsRef<str> for BmpVersion {
    fn as_ref(&self) -> &str {
        match *self {
            BmpVersion::Two => "BMP Version 2",
            BmpVersion::Three => "BMP Version 3",
            BmpVersion::ThreeNT => "BMP Version 3 NT",
            BmpVersion::Four => "BMP Version 4",
            BmpVersion::Five => "BMP Version 5",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompressionType {
    Uncompressed,
    Rle8bit,
    Rle4bit,
    // Only for BMP version 4
    BitfieldsEncoding,
}

impl CompressionType {
    pub fn from_u32(val: u32) -> CompressionType {
        match val {
            1 => CompressionType::Rle8bit,
            2 => CompressionType::Rle4bit,
            3 => CompressionType::BitfieldsEncoding,
            _ => CompressionType::Uncompressed,
        }
    }
}

impl AsRef<str> for CompressionType {
    fn as_ref(&self) -> &str {
        match *self {
            CompressionType::Rle8bit => "RLE 8-bit",
            CompressionType::Rle4bit => "RLE 4-bit",
            CompressionType::BitfieldsEncoding => "Bitfields Encoding",
            CompressionType::Uncompressed => "Uncompressed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BmpHeader {
    pub file_size: u32,
    pub creator1: u16,
    pub creator2: u16,
    pub pixel_offset: u32,
}

impl BmpHeader {
    pub fn new(header_size: u32, data_size: u32) -> BmpHeader {
        BmpHeader {
            file_size: header_size + data_size,
            creator1: 0, /* Unused */
            creator2: 0, /* Unused */
            pixel_offset: header_size,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BmpDibHeader {
    pub header_size: u32,
    pub width: i32,
    pub height: i32,
    pub num_planes: u16,
    pub bits_per_pixel: u16,
    pub compress_type: u32,
    pub data_size: u32,
    pub hres: i32,
    pub vres: i32,
    pub num_colors: u32,
    pub num_imp_colors: u32,
}

impl BmpDibHeader {
    pub fn new(width: i32, height: i32) -> BmpDibHeader {
        let (_, pixel_array_size) = file_size!(24, width, height);
        BmpDibHeader {
            header_size: 40,
            width,
            height,
            num_planes: 1,
            bits_per_pixel: 24,
            compress_type: 0,
            data_size: pixel_array_size,
            hres: 1000,
            vres: 1000,
            num_colors: 0,
            num_imp_colors: 0,
        }
    }
}

/// The image type provided by the library.
///
/// It exposes functions to initialize or read BMP images from disk, common modification of pixel
/// data, and saving to disk.
///
/// The image is accessed in row-major order from top to bottom,
/// where point (0, 0) is defined to be in the upper left corner of the image.
///
/// Currently, only uncompressed BMP images are supported.
#[derive(Clone, Eq, PartialEq)]
pub struct Image {
    pub header: BmpHeader,
    pub dib_header: BmpDibHeader,
    pub color_palette: Option<Vec<Pixel>>,
    pub width: u32,
    pub height: u32,
    pub padding: u32,
    pub data: Vec<Pixel>,
}

impl Image {
    /// Returns a new BMP Image with the `width` and `height` specified. It is initialized to
    /// a black image by default.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut img = bmp::Image::new(100, 80);
    /// ```
    pub fn new(width: u32, height: u32) -> Image {
        let mut data = Vec::with_capacity((width * height) as usize);
        for _ in 0..width * height {
            data.push(px!(0, 0, 0));
        }

        let (header_size, data_size) = file_size!(24, width, height);
        Image {
            header: BmpHeader::new(header_size, data_size),
            dib_header: BmpDibHeader::new(width as i32, height as i32),
            color_palette: None,
            width,
            height,
            padding: width % 4,
            data,
        }
    }

    /// Returns the `width` of the Image.
    #[inline]
    pub fn get_width(&self) -> u32 {
        self.width
    }

    /// Returns the `height` of the Image.
    #[inline]
    pub fn get_height(&self) -> u32 {
        self.height
    }

    /// Set the pixel value at the position of `width` and `height`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut img = bmp::Image::new(100, 80);
    /// img.set_pixel(10, 10, bmp::consts::RED);
    /// ```
    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, val: Pixel) {
        self.data[((self.height - y - 1) * self.width + x) as usize] = val;
    }

    /// Returns the pixel value at the position of `width` and `height`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let img = bmp::Image::new(100, 80);
    /// assert_eq!(bmp::consts::BLACK, img.get_pixel(10, 10));
    /// ```
    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> Pixel {
        self.data[((self.height - y - 1) * self.width + x) as usize]
    }

    /// Returns a new `ImageIndex` that iterates over the image dimensions in top-bottom order.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut img = bmp::Image::new(100, 100);
    /// for (x, y) in img.coordinates() {
    ///     img.set_pixel(x, y, bmp::consts::BLUE);
    /// }
    /// ```
    #[inline]
    pub fn coordinates(&self) -> ImageIndex {
        ImageIndex::new(self.width, self.height)
    }
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Image")
            .field("header", &self.header)
            .field("dib_header", &self.dib_header)
            .field("color_palette", &self.color_palette)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("padding", &self.padding)
            .finish()
    }
}

/// An `Iterator` returning the `x` and `y` coordinates of an image.
///
/// It supports iteration over an image in row-major order,
/// starting from in the upper left corner of the image.
#[derive(Clone, Copy)]
pub struct ImageIndex {
    width: u32,
    height: u32,
    x: u32,
    y: u32,
}

impl ImageIndex {
    fn new(width: u32, height: u32) -> ImageIndex {
        ImageIndex {
            width,
            height,
            x: 0,
            y: 0,
        }
    }
}

impl Iterator for ImageIndex {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<(u32, u32)> {
        if self.x < self.width && self.y < self.height {
            let this = Some((self.x, self.y));
            self.x += 1;
            if self.x == self.width {
                self.x = 0;
                self.y += 1;
            }
            this
        } else {
            None
        }
    }
}

/// Utility function to load an `Image` from the file specified by `path`.
/// It uses the `from_reader` function internally to decode the `Image`.
/// Returns a `BmpResult`, either containing an `Image` or a `BmpError`.
///
/// # Example
///
/// ```ignore
/// let img = bmp::open("test/rgbw.bmp").unwrap_or_else(|e| {
///    panic!("Failed to open: {}", e);
/// });
/// ```
pub fn open<P: AsRef<Path>>(path: P) -> BmpResult<Image> {
    let mut f = fs::File::open(path)?;
    from_reader(&mut f)
}

/// Attempts to construct a new `Image` from the given reader.
/// Returns a `BmpResult`, either containing an `Image` or a `BmpError`.
pub fn from_reader<R: Read>(source: &mut R) -> BmpResult<Image> {
    let mut bytes = Vec::new();
    source.read_to_end(&mut bytes)?;

    let mut bmp_data = Cursor::new(bytes);
    decoder::decode_image(&mut bmp_data)
}
