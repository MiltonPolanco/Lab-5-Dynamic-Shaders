use nalgebra_glm::Vec3;

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coords: Vec3,
    pub color: Vec3,
    pub transformed_position: Vec3,
    pub transformed_normal: Vec3,
}

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3, tex_coords: Vec3) -> Self {
        Vertex {
            position,
            normal,
            tex_coords,
            color: Vec3::new(1.0, 1.0, 1.0),
            transformed_position: Vec3::zeros(),
            transformed_normal: Vec3::zeros(),
        }
    }
}