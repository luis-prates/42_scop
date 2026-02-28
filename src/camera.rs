use crate::math;

type Point3 = math::Point3;
type Vector3 = math::Vector3;
type Matrix4 = math::Matrix4;

// Default camera values
const YAW: f32 = -90.0;
const PITCH: f32 = 0.0;
const SPEED: f32 = 2.5;
const SENSITIVTY: f32 = 0.1;
const ZOOM: f32 = 45.0;

pub struct Camera {
    // Camera Attributes
    pub position: Point3,
    pub front: Vector3,
    pub up: Vector3,
    pub right: Vector3,
    pub world_up: Vector3,
    // Euler Angles
    pub yaw: f32,
    pub pitch: f32,
    // Camera options
    #[allow(dead_code)]
    pub movement_speed: f32,
    pub mouse_sensitivity: f32,
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Camera {
        let mut camera = Camera {
            position: Point3::new(0.0, 0.0, 0.0),
            front: Vector3::new(0.0, 0.0, -1.0),
            up: Vector3::zero(),    // initialized later
            right: Vector3::zero(), // initialized later
            world_up: Vector3::unit_y(),
            yaw: YAW,
            pitch: PITCH,
            movement_speed: SPEED,
            mouse_sensitivity: SENSITIVTY,
            zoom: ZOOM,
        };
        camera.update_camera_vectors();
        camera
    }
}

impl Camera {
    /// Returns the view matrix calculated using Eular Angles and the LookAt Matrix
    pub fn get_view_matrix(&self) -> Matrix4 {
        Camera::calculate_look_at_matrix(self.position, self.position + self.front, self.up)
    }

    fn calculate_look_at_matrix(position: Point3, target: Point3, world_up: Vector3) -> Matrix4 {
        // 1. Position = known
        // 2. Calculate cameraDirection
        let z_axis = (position - target).normalize();
        // 3. Get positive right axis vector
        let x_axis = world_up.normalize().cross(z_axis);
        // 4. Calculate camera up vector
        let y_axis = z_axis.cross(x_axis);

        // Create translation and rotation matrix
        // In glm we access elements as mat[col][row] due to column-major layout
        let mut translation = Matrix4::identity(); // Identity matrix by default
        translation[3][0] = -position.x; // Third column, first row
        translation[3][1] = -position.y;
        translation[3][2] = -position.z;

        let mut rotation = Matrix4::identity();
        rotation[0][0] = x_axis.x; // First column, first row
        rotation[1][0] = x_axis.y;
        rotation[2][0] = x_axis.z;
        rotation[0][1] = y_axis.x; // First column, second row
        rotation[1][1] = y_axis.y;
        rotation[2][1] = y_axis.z;
        rotation[0][2] = z_axis.x; // First column, third row
        rotation[1][2] = z_axis.y;
        rotation[2][2] = z_axis.z;

        // Return lookAt matrix as combination of translation and rotation matrix
        rotation * translation // Remember to read from right to left (first translation then rotation)
    }

    /// Processes input received from a mouse input system. Expects the offset value in both the x and y direction.
    pub fn process_mouse_movement(
        &mut self,
        mut xoffset: f32,
        mut yoffset: f32,
        constrain_pitch: bool,
    ) {
        xoffset *= self.mouse_sensitivity;
        yoffset *= self.mouse_sensitivity;

        self.yaw += xoffset;
        self.pitch += yoffset;

        // Make sure that when pitch is out of bounds, screen doesn't get flipped
        if constrain_pitch {
            self.pitch = self.pitch.clamp(-89.0, 89.0);
        }

        // Update Front, Right and Up Vectors using the updated Eular angles
        // self.update_camera_vectors();
    }

    // Processes input received from a mouse scroll-wheel event. Only requires input on the vertical wheel-axis
    pub fn process_mouse_scroll(&mut self, yoffset: f32) {
        if self.zoom >= 1.0 && self.zoom <= 45.0 {
            self.zoom -= yoffset;
        }
        self.zoom = self.zoom.clamp(1.0, 45.0);
    }

    /// Calculates the front vector from the Camera's (updated) Eular Angles
    fn update_camera_vectors(&mut self) {
        // Calculate the new Front vector
        let front = Vector3 {
            x: self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            y: self.pitch.to_radians().sin(),
            z: self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        };
        self.front = front.normalize();
        // Also re-calculate the Right and Up vector
        self.right = self.front.cross(self.world_up).normalize(); // Normalize the vectors, because their length gets closer to 0 the more you look up or down which results in slower movement.
        self.up = self.right.cross(self.front).normalize();
    }
}
