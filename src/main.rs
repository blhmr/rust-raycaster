use minifb::{Key, Window, WindowOptions};
use rusttype::{Font, Scale, point};

use std::fs::read;
use std::f32::consts::PI;

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

const MAP_WIDTH: usize = 8;
const MAP_HEIGHT: usize = 8;

const MAP: [&str; MAP_HEIGHT] = [
    "########",
    "#      #",
    "#  ##  #",
    "#      #",
    "#      #",
    "#  ##  #",
    "#      #",
    "########",
];

fn main() {
    let mut window = Window::new("Rust Raycaster + Text", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut buffer = vec![0u32; WIDTH * HEIGHT];
    window.set_target_fps(60);

    let font_data = read("/usr/share/fonts/Adwaita/AdwaitaSans-Regular.ttf").unwrap();
    let font = Font::try_from_vec(font_data).unwrap();
    let scale = Scale::uniform(14.0);

    let mut px = 3.0;
    let mut py = 3.0;
    let mut dir: f32 = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        buffer.fill(0x000000);

        // Controls
        if window.is_key_down(Key::Left) {
            dir -= 0.05;
        }
        if window.is_key_down(Key::Right) {
            dir += 0.05;
        }
        if window.is_key_down(Key::W) {
            let newx = px + dir.cos() * 0.1;
            let newy = py + dir.sin() * 0.1;
            if MAP[newy as usize].as_bytes()[newx as usize] != b'#' {
                px = newx;
                py = newy;
            }
        }
        if window.is_key_down(Key::S) {
            let newx = px - dir.cos() * 0.1;
            let newy = py - dir.sin() * 0.1;
            if MAP[newy as usize].as_bytes()[newx as usize] != b'#' {
                px = newx;
                py = newy;
            }
        }

        // Raycasting
        for x in 0..WIDTH {
            let camera_x = 2.0 * x as f32 / WIDTH as f32 - 1.0;
            let ray_dir = (dir + camera_x * PI / 4.0).rem_euclid(2.0 * PI);
            let (mut ray_x, mut ray_y) = (px, py);

            let step_x = ray_dir.cos() * 0.05;
            let step_y = ray_dir.sin() * 0.05;

            for depth in 0..100 {
                ray_x += step_x;
                ray_y += step_y;

                if ray_x < 0.0 || ray_y < 0.0 || ray_x >= MAP_WIDTH as f32 || ray_y >= MAP_HEIGHT as f32 {
                    break;
                }

                if MAP[ray_y as usize].as_bytes()[ray_x as usize] == b'#' {
                    let dist = ((ray_x - px).powi(2) + (ray_y - py).powi(2)).sqrt();
                
                    let wall_height = (HEIGHT as f32 / dist) as i32;
                    let half_height = (HEIGHT / 2) as i32;
                
                    let start = (half_height - wall_height / 2).max(0);
                    let end = (half_height + wall_height / 2).min(HEIGHT as i32 - 1);
                
                    let shade = 255u32.saturating_sub((dist * 20.0) as u32);
                    let color = (shade.min(255) << 16) | ((shade / 2).min(255) << 8);
                
                    for y in start..=end {
                        buffer[y as usize * WIDTH + x] = color;
                    }
                    break;
                }
                
            }
        }

        // Draw text with rusttype
        draw_text(&mut buffer, &font, scale, 5, 5, &format!("X: {:.2} Y: {:.2}", px, py), 0xFFFFFF);

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

// 🖋️ Draw UTF-8 text using rusttype into the pixel buffer
fn draw_text(buffer: &mut [u32], font: &rusttype::Font, scale: rusttype::Scale, x: usize, y: usize, text: &str, color: u32) {
    let v_metrics = font.v_metrics(scale);
    let offset = point(x as f32, y as f32 + v_metrics.ascent);

    let glyphs: Vec<_> = font.layout(text, scale, offset).collect();

    for glyph in glyphs {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, v| {
                let px = gx as i32 + bb.min.x;
                let py = gy as i32 + bb.min.y;
                if px >= 0 && py >= 0 && (px as usize) < WIDTH && (py as usize) < HEIGHT {
                    let alpha = v;
                    let rgb = color & 0xFFFFFF;
                    let final_color = blend(buffer[py as usize * WIDTH + px as usize], rgb, alpha);
                    buffer[py as usize * WIDTH + px as usize] = final_color;
                }
            });
        }
    }
}

fn blend(bg: u32, fg: u32, alpha: f32) -> u32 {
    let bg_r = (bg >> 16) & 0xFF;
    let bg_g = (bg >> 8) & 0xFF;
    let bg_b = bg & 0xFF;

    let fg_r = (fg >> 16) & 0xFF;
    let fg_g = (fg >> 8) & 0xFF;
    let fg_b = fg & 0xFF;

    let r = (fg_r as f32 * alpha + bg_r as f32 * (1.0 - alpha)) as u32;
    let g = (fg_g as f32 * alpha + bg_g as f32 * (1.0 - alpha)) as u32;
    let b = (fg_b as f32 * alpha + bg_b as f32 * (1.0 - alpha)) as u32;

    (r << 16) | (g << 8) | b
}
