use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use minifb::{Key, Window, WindowOptions};
use std::time::{Duration, Instant};
use std::f32::consts::PI;

mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod camera;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use camera::Camera;
use triangle::triangle;
use shaders::{vertex_shader, fragment_shader};
use fastnoise_lite::{FastNoiseLite, NoiseType, FractalType};

pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: f32,
}

fn create_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(42); // Cambiado seed
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise.set_fractal_type(Some(FractalType::FBm));
    noise.set_fractal_octaves(Some(4)); // Aumentado a 4
    noise.set_fractal_lacunarity(Some(2.5)); // Aumentado
    noise.set_fractal_gain(Some(0.6)); // Aumentado
    noise.set_frequency(Some(0.8)); // Mucho más alto
    noise
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,   1.0, 0.0,
        0.0,    0.0,   0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0,   0.0,   translation.x,
        0.0,   scale, 0.0,   translation.y,
        0.0,   0.0,   scale, translation.z,
        0.0,   0.0,   0.0,   1.0,
    );

    transform_matrix * rotation_matrix
}

fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_at(&eye, &center, &up)
}

fn create_perspective_matrix(window_width: f32, window_height: f32) -> Mat4 {
    let fov = 45.0 * PI / 180.0;
    let aspect_ratio = window_width / window_height;
    let near = 0.1;
    let far = 1000.0;

    perspective(fov, aspect_ratio, near, far)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], noise: &FastNoiseLite) {
    // Transformar vértices
    let transformed_vertices: Vec<Vertex> = vertex_array
        .iter()
        .map(|v| vertex_shader(v, uniforms, noise))
        .collect();

    // Rasterizar triángulos
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            let fragments = triangle(
                &transformed_vertices[i],
                &transformed_vertices[i + 1],
                &transformed_vertices[i + 2]
            );

            // Sombrear fragmentos
            for fragment in fragments {
                let x = fragment.position.x as usize;
                let y = fragment.position.y as usize;
                if x < framebuffer.width && y < framebuffer.height {
                    let shaded_color = fragment_shader(&fragment, &uniforms, noise);
                    framebuffer.set_current_color(shaded_color.to_hex());
                    framebuffer.point(x, y, fragment.depth);
                }
            }
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    println!("Initializing framebuffer...");
    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    
    println!("Creating window...");
    let mut window = Window::new(
        "Animated Star - Solar Activity",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    window.set_position(500, 500);
    window.update();

    framebuffer.set_background_color(0x000008);

    println!("Loading sphere model...");
    let obj_path = if std::path::Path::new("assets/sphere_smooth.obj").exists() {
        println!("Using smooth sphere (320 triangles)");
        "assets/sphere_smooth.obj"
    } else if std::path::Path::new("assets/sphere_medium.obj").exists() {
        println!("Using medium sphere (80 triangles)");
        "assets/sphere_medium.obj"
    } else if std::path::Path::new("assets/sphere_simple.obj").exists() {
        println!("Using simple sphere (20 triangles)");
        "assets/sphere_simple.obj"
    } else {
        println!("Using original sphere");
        "assets/sphere.obj"
    };
    
    let obj = Obj::load(obj_path).expect("Failed to load obj");
    let vertex_arrays = obj.get_vertex_array();
    let triangle_count = vertex_arrays.len() / 3;
    println!("Loaded {} vertices ({} triangles)", vertex_arrays.len(), triangle_count);

    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 85.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0)
    );

    let noise = create_noise();
    
    let mut time = 0.0;
    let rotation_speed = 0.12;
    let mut frame_count = 0;
    let mut last_fps_print = Instant::now();
    let mut fps_counter = 0;

    const MIN_DISTANCE: f32 = 85.0;
    const MAX_DISTANCE: f32 = 200.0;

    println!("\nStarting render loop...");
    println!("Controls:");
    println!("  W: Zoom in (minimum distance: {:.0})", MIN_DISTANCE);
    println!("  S: Zoom out (maximum distance: {:.0})", MAX_DISTANCE);
    println!("  A/D: Orbit left/right");
    println!("  Q/E: Orbit up/down");
    println!("  ESC: Exit\n");

    while window.is_open() {
        let frame_start = Instant::now();
        
        if window.is_key_down(Key::Escape) {
            break;
        }

        time += 0.016;
        frame_count += 1;
        fps_counter += 1;

        if last_fps_print.elapsed() >= Duration::from_secs(1) {
            let fps = fps_counter as f32 / last_fps_print.elapsed().as_secs_f32();
            let distance = (camera.eye - camera.center).magnitude();
            
            println!("FPS: {:.1} | Frame: {} | Time: {:.1}s | Distance: {:.1}", 
                     fps, frame_count, time, distance);
            fps_counter = 0;
            last_fps_print = Instant::now();
        }

        handle_input(&window, &mut camera);

        let distance = (camera.eye - camera.center).magnitude();
        if distance < MIN_DISTANCE {
            let direction = (camera.eye - camera.center).normalize();
            camera.eye = camera.center + direction * MIN_DISTANCE;
        } else if distance > MAX_DISTANCE {
            let direction = (camera.eye - camera.center).normalize();
            camera.eye = camera.center + direction * MAX_DISTANCE;
        }

        framebuffer.clear();

        let rotation = Vec3::new(0.0, time * rotation_speed, 0.0);

        let uniforms = Uniforms {
            model_matrix: create_model_matrix(Vec3::new(0.0, 0.0, 0.0), 12.0, rotation),
            view_matrix: create_view_matrix(camera.eye, camera.center, camera.up),
            projection_matrix: create_perspective_matrix(window_width as f32, window_height as f32),
            viewport_matrix: create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32),
            time,
        };

        render(&mut framebuffer, &uniforms, &vertex_arrays, &noise);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        let frame_time = frame_start.elapsed();
        if frame_time < frame_delay {
            std::thread::sleep(frame_delay - frame_time);
        }
    }
    
    println!("\nExiting. Total frames rendered: {}", frame_count);
}

fn handle_input(window: &Window, camera: &mut Camera) {
    let movement_speed = 1.5;
    let rotation_speed = PI / 50.0;

    // W = acercar (pero se detendrá en MIN_DISTANCE)
    if window.is_key_down(Key::W) {
        camera.zoom(movement_speed);
    }
    // S = alejar (pero se detendrá en MAX_DISTANCE)
    if window.is_key_down(Key::S) {
        camera.zoom(-movement_speed);
    }
    if window.is_key_down(Key::A) {
        camera.orbit(rotation_speed, 0.0);
    }
    if window.is_key_down(Key::D) {
        camera.orbit(-rotation_speed, 0.0);
    }
    if window.is_key_down(Key::Q) {
        camera.orbit(0.0, rotation_speed);
    }
    if window.is_key_down(Key::E) {
        camera.orbit(0.0, -rotation_speed);
    }
}
