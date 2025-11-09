use nalgebra_glm::{Vec2, Vec3};
use crate::color::Color;

#[derive(Clone)]
pub struct Fragment {
    pub position: Vec2,
    pub depth: f32,
    pub color: Color,
    pub normal: Vec3,
    pub vertex_position: Vec3,
    pub intensity: f32,
}

impl Fragment {
    pub fn new(position: Vec2, depth: f32, color: Color, normal: Vec3, vertex_position: Vec3, intensity: f32) -> Self {
        Fragment {
            position,
            depth,
            color,
            normal,
            vertex_position,
            intensity,
        }
    }
}