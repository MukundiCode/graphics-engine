use std::f32::consts::PI;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::rect::{FPoint, FRect, Rect};
use sdl2::render::{WindowCanvas};
use nalgebra::*;
use log::{debug, log};
use rand::Rng;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const POINT_SIZE: f32 = 10.0;
pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let (points, faces) = get_vertices().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let z = 5.0;
    let mut angle = 1.0;
    let mut z_buffer = vec![f32::INFINITY; (WIDTH * HEIGHT) as usize];

    let mut translation_params = TranslationParams {
        dx: 0.0,
        dy: 0.0,
        dz: 10.0,
        angle_x: 0.0,
        angle_y: 0.0,
        angle_z: 0.0,
    };
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::RIGHT), ..} => {
                    translation_params.angle_x = translation_params.angle_x + PI * (1.0/60.0);
                },
                Event::KeyDown { keycode: Some(Keycode::LEFT), ..} => {
                    angle = angle - PI * (1.0/60.0);
                },
                Event::KeyDown { keycode: Some(Keycode::W), ..} => {
                    translation_params.dz = translation_params.dz + 0.3;
                },
                Event::KeyDown { keycode: Some(Keycode::S), ..} => {
                    translation_params.dz = translation_params.dz - 0.3;
                }
                _ => {}
            }
        }

        z_buffer.fill(f32::INFINITY);
        clear(&mut canvas);

        // for p in &points {
        //     point(&mut canvas, screen(project(translate_model(&translation_params, p.clone()))));
        // }

        let translation = Translation3::new(translation_params.dx, translation_params.dy, translation_params.dz).to_homogeneous();
        let rx = Rotation3::from_euler_angles(translation_params.angle_x, 0.0, 0.0).to_homogeneous();
        let ry = Rotation3::from_euler_angles(0.0, translation_params.angle_y, 0.0).to_homogeneous();
        let rz = Rotation3::from_euler_angles(0.0, 0.0, translation_params.angle_z).to_homogeneous();
        let model_matrix = translation * rz * ry * rx;

        // for face in &faces {
        //     for (index, _) in face.iter().enumerate() {
        //         let p1 = points[face[index] as usize];
        //         let p2 = points[face[(index+1)%face.len()] as usize];
        //         line(
        //             &mut canvas,
        //             screen(project(translate_model(&model_matrix, p1.clone()))),
        //             screen(project(translate_model(&model_matrix, p2.clone()))),
        //         );
        //     }
        // }
        let projected_points: Vec<P> = points.iter().map(|p| {
            screen(project(translate_model(&model_matrix, *p)))
        }).collect();

        // 2. Iterate through faces to get A, B, and C
        for face in &faces {
            // A face is a Vec<i32>, e.g., [10, 42, 15]
            if face.len() < 3 { continue; } // Safety check

            let a = projected_points[face[0] as usize];
            let b = projected_points[face[1] as usize];
            let c = projected_points[face[2] as usize];

            // Now you have your triangle!
            // You can pass a, b, and c into your rasterizer/color function.
            fill_triangle(&mut canvas, a, b, c, &mut z_buffer);
        }

        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

pub fn clear(canvas: &mut WindowCanvas) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
}

pub fn point (canvas: &mut WindowCanvas, p: P) {
    canvas.set_draw_color(Color::RGB(255, 210, 0));
    let fill_rect = FRect::new(p.x - POINT_SIZE/2.0, p.y - POINT_SIZE/2.0, POINT_SIZE, POINT_SIZE);
    canvas.fill_frect(fill_rect).unwrap();
}

pub fn pixel (canvas: &mut WindowCanvas, p: P, color: Color) {
    // canvas.set_draw_color(Color::RGB(255, 210, i as u8));
    canvas.set_draw_color(color);
    canvas.draw_fpoint(FPoint::new(p.x, p.y)).unwrap();
}

pub fn line (canvas: &mut WindowCanvas, p1: P, p2: P) {
    canvas.set_draw_color(Color::RGB(255, 210, 0));
    let p = FPoint::new(p1.x, p1.y);
    canvas.draw_fline(FPoint::new(p1.x, p1.y), FPoint::new(p2.x, p2.y));
}

pub fn screen(p: P) -> P {
    P {
        x: (p.x + 1.0)/2.0 * WIDTH as f32,
        y: (1.0 - (p.y + 1.0)/2.0) * HEIGHT as f32,
        z: p.z
    }
}

pub fn project(p: P) -> P {
    P {
        x: p.x / p.z,
        y: p.y / p.z,
        z: p.z
    }
}

pub fn translate(p: P, dz: f32) -> P {
    P {
        x: p.x,
        y: p.y,
        z: p.z + dz
    }
}

pub fn rotate_x(p: P, angle: f32) -> P {
    P {
        x: p.x * angle.cos() - p.z * angle.sin(),
        z: p.x * angle.sin() + p.z * angle.cos(),
        y: p.y
    }
}

pub fn translate_model(model_matrix: &Matrix4<f32>, p: P) -> P {
    let v = Vector4::new(p.x, p.y, p.z, 1.0);
    let result = model_matrix * v;

    P {
        x: result[0],
        y: result[1],
        z: result[2]
    }
}

pub fn get_vertices() -> io::Result<(Vec<P>, Vec<Vec<i32>>)> {
    let file = File::open("teapot.obj")?;
    let reader = BufReader::new(file);
    let mut vertices: Vec<P> = Vec::new();
    let mut faces: Vec<Vec<i32>> = Vec::new();

    for (_index, line) in reader.lines().enumerate() {
        let line = line?;
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.len() > 0 && words[0] == "v" {
            vertices.push(P {
                x: words[1].parse().expect("Not a valid number"),
                y: words[2].parse().expect("Not a valid number"),
                z: words[3].parse().expect("Not a valid number")
            });
        }
        if words.len() > 0 && words[0] == "f" {
            faces.push(words.iter().skip(1).map(|x| x.parse::<i32>().expect("Not a valid number") - 1).collect())
        }
    }
    Ok((vertices, faces))
}

fn edge_function(a: P, b: P, c: P) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}

pub fn fill_triangle(canvas: &mut WindowCanvas, v0: P, v1: P, v2: P, z_buffer: &mut Vec<f32>) {
    // 1. Find bounding box so we don't check every pixel on the whole screen
    let min_x = v0.x.min(v1.x).min(v2.x).floor() as i32;
    let max_x = v0.x.max(v1.x).max(v2.x).ceil() as i32;
    let min_y = v0.y.min(v1.y).min(v2.y).floor() as i32;
    let max_y = v0.y.max(v1.y).max(v2.y).ceil() as i32;

    let area = edge_function(v0, v1, v2);

    // 2. Loop through the bounding box
    let mut rng = rand::thread_rng();
    let color = Color::RGB(
        rng.gen_range(0..=255),
        rng.gen_range(0..=255),
        rng.gen_range(0..=255));
    for y in min_y..max_y {
        for x in min_x..max_x {
            // Check if point (x, y) is inside the triangle using edge functions
            // If yes, draw_point(x, y)
            let p = P { x: x as f32, y: y as f32, z: 0.0 };

            let w0 = edge_function(v1, v2, p) / area;
            let w1 = edge_function(v2, v0, p) / area;
            let w2 = edge_function(v0, v1, p) / area;

            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                // PIXEL IS INSIDE!
                // Calculate Z for Z-buffer:
                let z = 1.0 / (w0 / v0.z + w1 / v1.z + w2 / v2.z);

                let index = (y * WIDTH + x) as usize;
                let current_z = z_buffer[index];

                if (z < current_z) {
                    pixel(canvas, P {x: x as f32, y: y as f32, z: z as f32}, color);
                    z_buffer[index] = z;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct P {
    x: f32,
    y: f32,
    z: f32
}

pub struct TranslationParams {
    dx: f32,
    dy: f32,
    dz: f32,
    angle_x: f32,
    angle_y: f32,
    angle_z: f32,

}