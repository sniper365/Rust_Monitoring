use utils;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Memory {
    pub total: f32,
    pub free: f32,
    pub idle: f32,
    pub load: f32,
}

impl Memory {
    fn new(total: f32, free: f32, idle: f32, load: f32) -> Memory {
        Memory {
            total: total / 1024f32 / 1024f32,
            free: free / 1024f32 / 1024f32,
            idle: idle / 1024f32 / 1024f32,
            load: load
        }
    }
}

pub fn mem_information() -> Vec<Memory> {
    let info = utils::get_file_info("/proc/meminfo");

    let mut memory: Vec<Memory> = Vec::new();

    let (mut total, mut free, mut idle, mut load) = (0.0, 0.0, 0.0, 0.0);

    for line in info.lines() {
        let line_vec: Vec<&str> = line.split(":").collect();
        let value_vec: Vec<&str> = line_vec[1].trim().split(" ").collect();

        let name = line_vec[0].trim();
        let value = value_vec[0].trim();

        if name.starts_with("MemTotal") {
            total = value.parse::<f32>().unwrap();
        } else if name.starts_with("MemAvailable") {
            free = value.parse::<f32>().unwrap();
        } else if total > 0f32 && free > 0f32 {
            idle = total - free;
            load = 100.00f32 * idle / total;
        }
    }

    memory.push(Memory::new(total, free, idle, load));

    memory
}
