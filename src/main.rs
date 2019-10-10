#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate hashindexed;
extern crate term_painter;
extern crate term;

use std::{thread, time};
use std::io::Stdout;

use term_painter::ToStyle;

mod utils;
mod cpu;
mod hdd;
mod memory;

fn write_cpu_information(cpu_info: &Vec<cpu::Processor>, term_buffer: &mut Box<term::Terminal<Output=Stdout> + Send>) -> i32 {
    let mut line_printed: i32 = 0;

    for processor in cpu_info.iter() {
        let load_grid_vec = utils::get_print_grid(&processor.load, 40);
        let temp: f32 = processor.temperature as f32;
        let temp_grid_vec = utils::get_print_grid(&temp, 40);

        let _ = writeln!(term_buffer, "CPU {}:", processor.id);
        line_printed += 1;

        // Write load parameters
        let _ = write!(term_buffer, "Load: {:7}[", " ");
        for grid in load_grid_vec.iter() {
            let color = utils::get_color_grid(&processor.load, 40f32, 80f32);
            let _ = write!(term_buffer, "{}", color.paint(grid));
        }
        // Prevent bug with percent, when clear terminal
        let _ = writeln!(term_buffer, "] {:5.1} % ({:4.0}Mhz)", processor.load, processor.frequency);
        line_printed += 1;

        // Write temperature parameters
        let _ = write!(term_buffer, "Temperature: [");
        for grid in temp_grid_vec.iter() {
            let temp: f32 = processor.temperature as f32;
            let color = utils::get_color_grid(&temp, 40f32, 65f32);
            let _ = write!(term_buffer, "{}", color.paint(grid));
        }
        let _ = writeln!(term_buffer, "] {:5.1} C", processor.temperature);
        line_printed += 1;
    }

    line_printed
}

fn write_mem_information(mem_info: &Vec<memory::Memory>, term_buffer: &mut Box<term::Terminal<Output=Stdout> + Send>) -> i32 {
    let mut line_printed: i32 = 0;

    for mem in mem_info.iter() {
        let load_grid_vec = utils::get_print_grid(&mem.load, 40);

        // Write load grid
        let _ = write!(term_buffer, "RAM: {:8}[", " ");
        for grid in load_grid_vec.iter() {
            let color = utils::get_color_grid(&mem.load, 40f32, 80f32);
            let _ = write!(term_buffer, "{}", color.paint(grid));
        }
        let _ = writeln!(term_buffer, "]");
        line_printed += 1;

        let line = format!("{:3.1} GiB / {:3.1} GiB ({:3.2}%)", mem.idle, mem.total, mem.load);
        let _ = writeln!(term_buffer, "{: ^1$}", line, 75);
        line_printed += 1;
    }

    line_printed
}

fn write_hdd_information(hdd_info: &Vec<hdd::HDD>, term_buffer: &mut Box<term::Terminal<Output=Stdout> + Send>) -> i32 {
    let mut line_printed: i32 = 0;

    for disk in hdd_info.iter() {
        let _ = writeln!(term_buffer, "/dev/{}:", disk.name);
        line_printed += 1;
        let _ = writeln!(term_buffer, "Size: {}", disk.size);
        line_printed += 1;

        if !disk.mount_point.is_empty() {
            let _ = writeln!(term_buffer, "Mount point: {}", disk.mount_point);
            line_printed += 1;
        }
        if !disk.fstype.is_empty() {
            let _ = writeln!(term_buffer, "FS Type: {}", disk.fstype);
            line_printed += 1;
        }

        let _ = writeln!(term_buffer, "");
        line_printed += 1;
    }

    line_printed
}

fn main() {
    let mut term_buffer = term::stdout().expect("Terminal not found");

    loop {
        let mut line_printed = 0;

        let cpu: Vec<cpu::Processor> = cpu::cpu_information();
        let memory: Vec<memory::Memory> = memory::mem_information();
        let hdd: Vec<hdd::HDD> = hdd::hdd_information();

        let _ = writeln!(term_buffer, "{:#^1$}", " CPU ", 75);
        line_printed += 1;

        // CPU INFORMATION
        line_printed += write_cpu_information(&cpu, &mut term_buffer);

        let _ = writeln!(term_buffer, "");
        line_printed += 1;

        let _ = writeln!(term_buffer, "{:#^1$}", " Memory ", 75);
        line_printed += 1;

        // MEMORY INFORMATION
        line_printed += write_mem_information(&memory, &mut term_buffer);

        let _ = writeln!(term_buffer, "");
        line_printed += 1;

        let _ = writeln!(term_buffer, "{:#^1$}", " HDD ", 75);
        line_printed += 1;

        // HDD INFORMATION
        line_printed += write_hdd_information(&hdd, &mut term_buffer);

        // This is needed to clear all output
        for _ in 0..line_printed {
            let _ = term_buffer.cursor_up();
        }

        thread::sleep(time::Duration::from_millis(1500 as u64));
    }
}
