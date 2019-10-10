use std::path::Path;
use std::io;
use std::io::Read;
use std::fs::File;

use term_painter::Color::*;
use term_painter::Color;


fn get_file(path: &Path, s: &mut String) -> io::Result<()> {
    let mut f = try!(File::open(path));
    try!(f.read_to_string(s));
    Ok(())
}

pub fn get_file_info(path: &str) -> String {
    let mut info = String::new();
    let path = Path::new(path);
    let error = get_file(path, &mut info);
    if error.is_err() {
        let msg = format!("Could not get information from {:?}", path);
        panic!(msg);
    }

    info
}

pub fn vec_sum(vectors: Vec<&str>) -> f32 {
    let mut sum: f32 = 0.00;

    for vector in vectors.iter() {
        sum += vector.trim().parse::<f32>().unwrap();
    }
    sum
}

pub fn get_print_grid(load: &f32, max_grid: i32) -> Vec<&str> {
    let grid_count = load / (100f32 / max_grid as f32);

    let mut grid_vec: Vec<&str> = Vec::new();
    for i in 0..max_grid as i32 {
        if i < grid_count as i32 {
            grid_vec.push("#");
        } else {
            grid_vec.push(" ");
        }
    }

    grid_vec
}

pub fn get_color_grid(load: &f32, min_val: f32, avg_val: f32) -> Color {
    let color;

    if load < &min_val {
        color = BrightGreen;
    } else if load >= &min_val && load < &avg_val {
        color = BrightYellow;
    } else {
        color = BrightRed;
    }

    color
}
