use nalgebra_glm::{Vec2, Vec3};
use crate::fragment::Fragment;
use crate::vertex::Vertex;
use crate::color::Color;

pub fn triangle(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Vec<Fragment> {
    let mut fragments = Vec::new();

    let a = v1.transformed_position;
    let b = v2.transformed_position;
    let c = v3.transformed_position;

    let (min_x, min_y, max_x, max_y) = calculate_bounding_box(&a, &b, &c);

    let light_dir = Vec3::new(0.0, 0.0, 1.0);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let point = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);
            
            if let Some((w1, w2, w3)) = barycentric_coordinates(&point, &a, &b, &c) {
                if w1 >= 0.0 && w1 <= 1.0 && 
                   w2 >= 0.0 && w2 <= 1.0 && 
                   w3 >= 0.0 && w3 <= 1.0 &&
                   (w1 + w2 + w3 - 1.0).abs() < 0.001 {
                    
                    let depth = w1 * a.z + w2 * b.z + w3 * c.z;
                    
                    let normal = (v1.transformed_normal * w1 + 
                                 v2.transformed_normal * w2 + 
                                 v3.transformed_normal * w3).normalize();
                    
                    let vertex_position = v1.position * w1 + v2.position * w2 + v3.position * w3;
                    
                    let intensity = normal.dot(&light_dir).max(0.0);
                    
                    fragments.push(Fragment::new(
                        Vec2::new(x as f32, y as f32),
                        depth,
                        Color::new(255, 255, 255),
                        normal,
                        vertex_position,
                        intensity,
                    ));
                }
            }
        }
    }

    fragments
}

fn calculate_bounding_box(v1: &Vec3, v2: &Vec3, v3: &Vec3) -> (i32, i32, i32, i32) {
    let min_x = v1.x.min(v2.x).min(v3.x).floor() as i32;
    let min_y = v1.y.min(v2.y).min(v3.y).floor() as i32;
    let max_x = v1.x.max(v2.x).max(v3.x).ceil() as i32;
    let max_y = v1.y.max(v2.y).max(v3.y).ceil() as i32;

    (min_x, min_y, max_x, max_y)
}

fn barycentric_coordinates(p: &Vec3, a: &Vec3, b: &Vec3, c: &Vec3) -> Option<(f32, f32, f32)> {
    let v0 = Vec3::new(b.x - a.x, b.y - a.y, 0.0);
    let v1 = Vec3::new(c.x - a.x, c.y - a.y, 0.0);
    let v2 = Vec3::new(p.x - a.x, p.y - a.y, 0.0);

    let dot00 = v0.dot(&v0);
    let dot01 = v0.dot(&v1);
    let dot11 = v1.dot(&v1);
    let dot20 = v2.dot(&v0);
    let dot21 = v2.dot(&v1);

    let denom = dot00 * dot11 - dot01 * dot01;

    if denom.abs() < 1e-8 {
        return None;
    }

    let inv_denom = 1.0 / denom;
    let w2 = (dot11 * dot20 - dot01 * dot21) * inv_denom;
    let w3 = (dot00 * dot21 - dot01 * dot20) * inv_denom;
    let w1 = 1.0 - w2 - w3;

    Some((w1, w2, w3))
}