use nalgebra_glm::Vec3;
use crate::vertex::Vertex;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct Obj {
    vertices: Vec<Vec3>,
    normals: Vec<Vec3>,
    texcoords: Vec<Vec3>,
    faces: Vec<(Vec3, Vec3, Vec3)>,
}

impl Obj {
    pub fn load(filename: &str) -> Result<Self, std::io::Error> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut texcoords = Vec::new();
        let mut faces = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "v" if parts.len() >= 4 => {
                    vertices.push(Vec3::new(
                        parts[1].parse().unwrap_or(0.0),
                        parts[2].parse().unwrap_or(0.0),
                        parts[3].parse().unwrap_or(0.0),
                    ));
                }
                "vn" if parts.len() >= 4 => {
                    normals.push(Vec3::new(
                        parts[1].parse().unwrap_or(0.0),
                        parts[2].parse().unwrap_or(0.0),
                        parts[3].parse().unwrap_or(0.0),
                    ));
                }
                "vt" if parts.len() >= 3 => {
                    texcoords.push(Vec3::new(
                        parts[1].parse().unwrap_or(0.0),
                        parts[2].parse().unwrap_or(0.0),
                        0.0,
                    ));
                }
                "f" if parts.len() >= 4 => {
                    let parse_face = |s: &str| -> Vec3 {
                        let indices: Vec<&str> = s.split('/').collect();
                        Vec3::new(
                            indices.get(0).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                            indices.get(1).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                            indices.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.0),
                        )
                    };

                    let face1 = parse_face(parts[1]);
                    let face2 = parse_face(parts[2]);
                    let face3 = parse_face(parts[3]);

                    faces.push((face1, face2, face3));
                }
                _ => {}
            }
        }

        Ok(Obj {
            vertices,
            normals,
            texcoords,
            faces,
        })
    }

    pub fn get_vertex_array(&self) -> Vec<Vertex> {
        let mut vertex_array = Vec::new();

        for face in &self.faces {
            for i in 0..3 {
                let face_indices = if i == 0 { face.0 } else if i == 1 { face.1 } else { face.2 };
                
                let position = if face_indices.x > 0.0 && (face_indices.x as usize) <= self.vertices.len() {
                    self.vertices[(face_indices.x - 1.0) as usize]
                } else {
                    Vec3::zeros()
                };

                let texcoord = if face_indices.y > 0.0 && (face_indices.y as usize) <= self.texcoords.len() {
                    self.texcoords[(face_indices.y - 1.0) as usize]
                } else {
                    Vec3::zeros()
                };

                let normal = if face_indices.z > 0.0 && (face_indices.z as usize) <= self.normals.len() {
                    self.normals[(face_indices.z - 1.0) as usize]
                } else {
                    Vec3::zeros()
                };

                vertex_array.push(Vertex::new(position, normal, texcoord));
            }
        }

        vertex_array
    }
}