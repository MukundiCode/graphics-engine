use nalgebra::*;
use sdl2::pixels::Color;
use sdl2::render::WindowCanvas;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::time::Duration;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::f32::consts::PI;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let (points, faces) = get_vertices().unwrap();

    let window = video_subsystem.window("Rust Software Rasterizer", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Z-Buffer initialized to Infinity
    let mut z_buffer = vec![f32::INFINITY; (WIDTH * HEIGHT) as usize];

    let mut params = TranslationParams {
        dx: 0.0, dy: 0.0, dz: 15.0,
        angle_x: 0.0, angle_y: 0.0, angle_z: 0.0,
    };

    let light_pos = Vector3::new(10.0, 20.0, -10.0);

    'running: loop {
        let move_speed = 0.2;
        let rotation_speed = 0.05;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,

                // Translation (Movement)
                Event::KeyDown { keycode: Some(Keycode::W), .. } => params.dz -= move_speed,
                Event::KeyDown { keycode: Some(Keycode::S), .. } => params.dz += move_speed,
                Event::KeyDown { keycode: Some(Keycode::A), .. } => params.dx -= move_speed,
                Event::KeyDown { keycode: Some(Keycode::D), .. } => params.dx += move_speed,
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => params.dy -= move_speed,
                Event::KeyDown { keycode: Some(Keycode::E), .. } => params.dy += move_speed,

                // Rotation
                Event::KeyDown { keycode: Some(Keycode::Up), .. }    => params.angle_x -= rotation_speed,
                Event::KeyDown { keycode: Some(Keycode::Down), .. }  => params.angle_x += rotation_speed,
                Event::KeyDown { keycode: Some(Keycode::Left), .. }  => params.angle_y -= rotation_speed,
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => params.angle_y += rotation_speed,
                Event::KeyDown { keycode: Some(Keycode::Z), .. }     => params.angle_z -= rotation_speed,
                Event::KeyDown { keycode: Some(Keycode::X), .. }     => params.angle_z += rotation_speed,

                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        z_buffer.fill(f32::INFINITY);

        // 1. Create Model Matrix
        let translation = Translation3::new(params.dx, params.dy, params.dz).to_homogeneous();
        let rotation = Rotation3::from_euler_angles(params.angle_x, params.angle_y, params.angle_z).to_homogeneous();
        let model_matrix = translation * rotation;

        // 2. Pre-process Vertices
        let world_points: Vec<Vector3<f32>> = points.iter().map(|p| {
            let v = Vector4::new(p.x, p.y, p.z, 1.0);
            let res = model_matrix * v;
            Vector3::new(res.x, res.y, res.z)
        }).collect();

        let screen_points: Vec<P> = world_points.iter().map(|wp| {
            screen(project(P { x: wp.x, y: wp.y, z: wp.z }))
        }).collect();

        // 3. Render Faces
        for face in &faces {
            if face.len() < 3 { continue; }

            let i0 = face[0] as usize;
            let i1 = face[1] as usize;
            let i2 = face[2] as usize;

            // Pass both World (for light) and Screen (for drawing) coordinates
            fill_triangle(
                &mut canvas,
                screen_points[i0], screen_points[i1], screen_points[i2],
                world_points[i0], world_points[i1], world_points[i2],
                &mut z_buffer,
                light_pos
            );
        }

        canvas.present();
        std::thread::sleep(Duration::from_millis(16));
    }
}

pub fn fill_triangle(
    canvas: &mut WindowCanvas,
    s0: P, s1: P, s2: P,
    w0: Vector3<f32>, w1: Vector3<f32>, w2: Vector3<f32>,
    z_buffer: &mut Vec<f32>,
    light_pos: Vector3<f32>
) {
    // --- LIGHTING (WORLD SPACE) ---
    let e1 = w1 - w0;
    let e2 = w2 - w0;
    let normal = e1.cross(&e2).normalize();

    // Directional or Point Light? Let's go with a Point Light feel
    let light_dir = (light_pos - w0).normalize();
    let intensity = normal.dot(&light_dir).max(0.0);

    // Basic gold-ish material
    let color = Color::RGB(
        (255.0 * intensity) as u8,
        (200.0 * intensity) as u8,
        (50.0 * intensity) as u8
    );

    // --- RASTERIZATION (SCREEN SPACE) ---
    let min_x = (s0.x.min(s1.x).min(s2.x).floor() as i32).clamp(0, WIDTH - 1);
    let max_x = (s0.x.max(s1.x).max(s2.x).ceil() as i32).clamp(0, WIDTH - 1);
    let min_y = (s0.y.min(s1.y).min(s2.y).floor() as i32).clamp(0, HEIGHT - 1);
    let max_y = (s0.y.max(s1.y).max(s2.y).ceil() as i32).clamp(0, HEIGHT - 1);

    let area = edge_function(s0, s1, s2);
    if area.abs() < 0.0001 { return; }

    canvas.set_draw_color(color);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = P { x: x as f32, y: y as f32, z: 0.0 };

            let w0_bary = edge_function(s1, s2, p) / area;
            let w1_bary = edge_function(s2, s0, p) / area;
            let w2_bary = edge_function(s0, s1, p) / area;

            if w0_bary >= 0.0 && w1_bary >= 0.0 && w2_bary >= 0.0 {
                // Interpolate Screen-Z
                let z = w0_bary * s0.z + w1_bary * s1.z + w2_bary * s2.z;

                let idx = (y * WIDTH + x) as usize;
                if z < z_buffer[idx] {
                    z_buffer[idx] = z;
                    canvas.draw_point(sdl2::rect::Point::new(x, y)).unwrap();
                }
            }
        }
    }
}

// --- UTILS ---

fn edge_function(a: P, b: P, c: P) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

pub fn project(p: P) -> P {
    P { x: p.x / p.z, y: p.y / p.z, z: p.z }
}

pub fn screen(p: P) -> P {
    P {
        x: (p.x + 1.0) * 0.5 * WIDTH as f32,
        y: (1.0 - (p.y + 1.0) * 0.5) * HEIGHT as f32,
        z: p.z
    }
}

#[derive(Clone, Copy)]
pub struct P { pub x: f32, pub y: f32, pub z: f32 }

pub struct TranslationParams {
    pub dx: f32, pub dy: f32, pub dz: f32,
    pub angle_x: f32, pub angle_y: f32, pub angle_z: f32,
}

pub fn get_vertices() -> io::Result<(Vec<P>, Vec<Vec<i32>>)> {
    let file = File::open("cottage_obj.obj")?;
    let reader = BufReader::new(file);
    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.is_empty() { continue; }
        match words[0] {
            "v" => vertices.push(P {
                x: words[1].parse().unwrap(),
                y: words[2].parse().unwrap(),
                z: words[3].parse().unwrap(),
            }),
            "f" => faces.push(words[1..].iter()
                .map(|w| w.split('/').next().unwrap().parse::<i32>().unwrap() - 1)
                .collect()),
            _ => {}
        }
    }
    Ok((vertices, faces))
}