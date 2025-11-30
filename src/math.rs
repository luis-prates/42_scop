use std::ops::{Add, AddAssign, Div, Index, IndexMut, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vector2 { x, y }
    }

    pub fn zero() -> Self {
        Vector2 { x: 0.0, y: 0.0 }
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(self) -> Vector2 {
        let magnitude = self.magnitude();
        if magnitude != 0.0 {
            Vector2 {
                x: self.x / magnitude,
                y: self.y / magnitude,
            }
        } else {
            Vector2::zero()
        }
    }

    pub fn dot(&self, other: Vector2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn add(&self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn subtract(&self, other: Vector2) -> Vector2 {
        Vector2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Sub<Point3> for Point3 {
    type Output = Vector3;

    fn sub(self, rhs: Point3) -> Self::Output {
        Vector3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Add<Vector3> for Point3 {
    type Output = Point3;

    fn add(self, rhs: Vector3) -> Self::Output {
        Point3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign<Vector3> for Point3 {
    fn add_assign(&mut self, rhs: Vector3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

// Point3 operations
impl Point3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Point3 { x, y, z }
    }

    pub fn to_vec(&self) -> Vector3 {
        Vector3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Vector4 {
    /// The x component of the vector.
    pub x: f32,
    /// The y component of the vector.
    pub y: f32,
    /// The z component of the vector.
    pub z: f32,
    /// The w component of the vector.
    pub w: f32,
}

impl Vector4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Vector4 { x, y, z, w }
    }

    pub fn dot(self, rhs: Vector4) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w
    }
}

impl Mul<f32> for Vector4 {
    type Output = Vector4;

    fn mul(self, scalar: f32) -> Vector4 {
        Vector4 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
            w: self.w * scalar,
        }
    }
}

impl Add for Vector4 {
    type Output = Vector4;

    fn add(self, other: Vector4) -> Vector4 {
        Vector4 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4 {
    /// The first column of the matrix.
    pub x: Vector4,
    /// The second column of the matrix.
    pub y: Vector4,
    /// The third column of the matrix.
    pub z: Vector4,
    /// The fourth column of the matrix.
    pub w: Vector4,
}

// Vector 3 operations
impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vector3 { x, y, z }
    }

    pub fn zero() -> Self {
        Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn unit_y() -> Self {
        Vector3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
    }

    pub fn normalize(&self) -> Vector3 {
        let length = (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt();
        Vector3 {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        }
    }

    pub fn cross(self, rhs: Vector3) -> Vector3 {
        Vector3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn as_ptr(&self) -> *const f32 {
        // The vector is represented as a contiguous array of f32 values,
        // so we can obtain a pointer to the first element of the array.
        &self.x as *const f32
    }
}

impl Add<f32> for Vector3 {
    type Output = Vector3;

    fn add(self, rhs: f32) -> Self::Output {
        Vector3 {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
        }
    }
}

impl Sub<f32> for Vector3 {
    type Output = Vector3;

    fn sub(self, rhs: f32) -> Self::Output {
        Vector3 {
            x: self.x - rhs,
            y: self.y - rhs,
            z: self.z - rhs,
        }
    }
}

impl Mul<f32> for Vector3 {
    type Output = Vector3;

    fn mul(self, rhs: f32) -> Self::Output {
        Vector3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Div<f32> for Vector3 {
    type Output = Vector3;

    fn div(self, rhs: f32) -> Self::Output {
        if rhs != 0.0 {
            Vector3 {
                x: self.x / rhs,
                y: self.y / rhs,
                z: self.z / rhs,
            }
        } else {
            panic!("Attempted to divide by zero.")
        }
    }
}

impl AddAssign<Point3> for Point3 {
    fn add_assign(&mut self, rhs: Point3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Neg for Vector3 {
    type Output = Vector3;

    fn neg(self) -> Self::Output {
        Vector3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

// Matrix4 operations
impl Matrix4 {
    pub fn new(x: Vector4, y: Vector4, z: Vector4, w: Vector4) -> Self {
        Matrix4 { x, y, z, w }
    }

    pub fn identity() -> Self {
        Matrix4 {
            x: Vector4::new(1.0, 0.0, 0.0, 0.0),
            y: Vector4::new(0.0, 1.0, 0.0, 0.0),
            z: Vector4::new(0.0, 0.0, 1.0, 0.0),
            w: Vector4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn from_scale(scale: f32) -> Self {
        Matrix4 {
            x: Vector4::new(scale, 0.0, 0.0, 0.0),
            y: Vector4::new(0.0, scale, 0.0, 0.0),
            z: Vector4::new(0.0, 0.0, scale, 0.0),
            w: Vector4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn from_translation(translation: Vector3) -> Self {
        Matrix4 {
            x: Vector4::new(1.0, 0.0, 0.0, 0.0),
            y: Vector4::new(0.0, 1.0, 0.0, 0.0),
            z: Vector4::new(0.0, 0.0, 1.0, 0.0),
            w: Vector4::new(translation.x, translation.y, translation.z, 1.0),
        }
    }

    pub fn from_axis_angle(axis: Vector3, angle_degrees: f32) -> Self {
        let angle_radians = angle_degrees.to_radians();
        let cos_a = angle_radians.cos();
        let sin_a = angle_radians.sin();

        let one_minus_cos_a = 1.0 - cos_a;

        let x = axis.x;
        let y = axis.y;
        let z = axis.z;

        let xy = x * y;
        let xz = x * z;
        let yz = y * z;

        let x_squared = x * x;
        let y_squared = y * y;
        let z_squared = z * z;

        Matrix4 {
            x: Vector4::new(
                x_squared + (1.0 - x_squared) * cos_a,
                xy * one_minus_cos_a - z * sin_a,
                xz * one_minus_cos_a + y * sin_a,
                0.0,
            ),
            y: Vector4::new(
                xy * one_minus_cos_a + z * sin_a,
                y_squared + (1.0 - y_squared) * cos_a,
                yz * one_minus_cos_a - x * sin_a,
                0.0,
            ),
            z: Vector4::new(
                xz * one_minus_cos_a - y * sin_a,
                yz * one_minus_cos_a + x * sin_a,
                z_squared + (1.0 - z_squared) * cos_a,
                0.0,
            ),
            w: Vector4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn perspective(fov_degrees: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_degrees.to_radians() / 2.0).tan();
        let depth = far - near;

        Matrix4 {
            x: Vector4::new(f / aspect_ratio, 0.0, 0.0, 0.0),
            y: Vector4::new(0.0, f, 0.0, 0.0),
            z: Vector4::new(0.0, 0.0, -(far + near) / depth, -1.0),
            w: Vector4::new(0.0, 0.0, -(2.0 * far * near) / depth, 0.0),
        }
    }

    fn get_column(&self, col: usize) -> Vector4 {
        match col {
            0 => Vector4::new(self.x.x, self.y.x, self.z.x, self.w.x),
            1 => Vector4::new(self.x.y, self.y.y, self.z.y, self.w.y),
            2 => Vector4::new(self.x.z, self.y.z, self.z.z, self.w.z),
            3 => Vector4::new(self.x.w, self.y.w, self.z.w, self.w.w),
            _ => panic!("Invalid column index"),
        }
    }

    pub fn as_ptr(&self) -> *const f32 {
        &self.x.x as *const f32
    }

    pub fn from_cols(x: Vector4, y: Vector4, z: Vector4, w: Vector4) -> Self {
        Matrix4 { x, y, z, w }
    }
}

impl Mul for Matrix4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Matrix4 {
        {
            let a = self[0];
            let b = self[1];
            let c = self[2];
            let d = self[3];

            Matrix4::from_cols(
                a * rhs[0][0] + b * rhs[0][1] + c * rhs[0][2] + d * rhs[0][3],
                a * rhs[1][0] + b * rhs[1][1] + c * rhs[1][2] + d * rhs[1][3],
                a * rhs[2][0] + b * rhs[2][1] + c * rhs[2][2] + d * rhs[2][3],
                a * rhs[3][0] + b * rhs[3][1] + c * rhs[3][2] + d * rhs[3][3],
            )
        }
    }
}

impl Index<usize> for Matrix4 {
    type Output = Vector4;

    fn index(&self, i: usize) -> &Self::Output {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("Invalid row index"),
        }
    }
}

impl IndexMut<usize> for Matrix4 {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => panic!("Invalid row index"),
        }
    }
}

impl Index<usize> for Vector4 {
    type Output = f32;

    fn index(&self, i: usize) -> &Self::Output {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("Invalid component index"),
        }
    }
}

impl IndexMut<usize> for Vector4 {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => panic!("Invalid component index"),
        }
    }
}
