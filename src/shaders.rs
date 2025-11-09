use nalgebra_glm::{Vec3, Vec4, Mat3, mat4_to_mat3};
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::Fragment;
use crate::color::Color;
use fastnoise_lite::FastNoiseLite;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms, noise: &FastNoiseLite) -> Vertex {
    let position = Vec4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );

    let noise_value = noise.get_noise_3d(
        vertex.position.x * 1.5,
        vertex.position.y * 1.5,
        vertex.position.z * 1.5 + uniforms.time * 0.3
    );

    let pulse = (uniforms.time * 1.8).sin() * 0.35 + 0.65;
    let displacement = noise_value * 0.08 * pulse;
    
    let displaced_position = Vec4::new(
        position.x + vertex.normal.x * displacement,
        position.y + vertex.normal.y * displacement,
        position.z + vertex.normal.z * displacement,
        1.0
    );

    let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * displaced_position;

    let w = transformed.w;
    let ndc_position = Vec4::new(
        transformed.x / w,
        transformed.y / w,
        transformed.z / w,
        1.0
    );

    let screen_position = uniforms.viewport_matrix * ndc_position;

    let model_mat3 = mat4_to_mat3(&uniforms.model_matrix);
    let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());
    let transformed_normal = normal_matrix * vertex.normal;

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
        transformed_normal,
    }
}

pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms, noise: &FastNoiseLite) -> Color {
    let pos = fragment.vertex_position;
    let time_factor = uniforms.time * 0.18;

    // Múltiples capas de ruido para simular turbulencia solar
    let base_noise = noise.get_noise_3d(
        pos.x * 2.2,
        pos.y * 2.2,
        pos.z * 2.2 + time_factor
    );

    let detail_noise = noise.get_noise_3d(
        pos.x * 5.0,
        pos.y * 5.0,
        pos.z * 5.0 + time_factor * 1.5
    );

    // Manchas solares con ruido de alta frecuencia
    let spots = noise.get_noise_3d(
        pos.x * 9.0,
        pos.y * 9.0,
        pos.z * 9.0 - time_factor * 0.7
    );

    // Pulsaciones para simular actividad solar
    let pulse1 = (uniforms.time * 2.2).sin() * 0.5 + 0.5;
    let pulse2 = (uniforms.time * 3.3 + 1.0).cos() * 0.5 + 0.5;

    // Combinar las capas
    let combined = base_noise * 0.35 + detail_noise * 0.35 + spots * 0.3;
    
    let normalized = (combined + 1.0) * 0.5;
    let temperature = normalized.powf(0.38).clamp(0.0, 1.0);

    let temp_pulsed = temperature * (0.75 + pulse1 * 0.25);

    // Mapear temperatura a colores (blanco caliente -> rojo frío)
    let color = if temp_pulsed > 0.88 {
        Color::new(255, 255, 248)
    } else if temp_pulsed > 0.75 {
        let t = (temp_pulsed - 0.75) / 0.13;
        Color::new(255, (242.0 - t * 12.0) as u8, (195.0 + t * 53.0) as u8)
    } else if temp_pulsed > 0.61 {
        let t = (temp_pulsed - 0.61) / 0.14;
        Color::new(255, (210.0 + t * 32.0) as u8, (90.0 + t * 105.0) as u8)
    } else if temp_pulsed > 0.47 {
        let t = (temp_pulsed - 0.47) / 0.14;
        Color::new((238.0 + t * 17.0) as u8, (165.0 + t * 45.0) as u8, (35.0 + t * 55.0) as u8)
    } else if temp_pulsed > 0.34 {
        let t = (temp_pulsed - 0.34) / 0.13;
        Color::new((205.0 + t * 33.0) as u8, (115.0 + t * 50.0) as u8, (18.0 + t * 17.0) as u8)
    } else if temp_pulsed > 0.22 {
        let t = (temp_pulsed - 0.22) / 0.12;
        Color::new((165.0 + t * 40.0) as u8, (60.0 + t * 55.0) as u8, (8.0 + t * 10.0) as u8)
    } else {
        // Manchas solares oscuras
        let t = temp_pulsed / 0.22;
        Color::new((80.0 + t * 85.0) as u8, (15.0 + t * 45.0) as u8, (3.0 + t * 5.0) as u8)
    };

    // Emisión basada en temperatura
    let base_emission = if temp_pulsed > 0.75 {
        1.5
    } else if temp_pulsed > 0.55 {
        1.15
    } else if temp_pulsed > 0.35 {
        0.9
    } else {
        0.65
    };
    
    let pulsating_emission = base_emission * (0.92 + pulse2 * 0.08);
    
    // Efecto de corona en los bordes
    let view_dir = fragment.vertex_position.normalize();
    let normal = fragment.normal.normalize();
    let fresnel = (1.0 - view_dir.dot(&normal).abs()).powf(3.2);
    let edge_glow = fresnel * 0.3 * pulse1;
    
    color * (pulsating_emission + edge_glow)
}