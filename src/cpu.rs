use std::sync::Mutex;
use std::process::Command;

use hashindexed::{HashIndexed, KeyComparator};

use utils;


#[derive(Debug, Copy, Clone, PartialEq)]
struct ProcessorIdle {
    idle: f32,
    total: f32,
    core_id: u8,
}

struct ProcessorComparator;

impl KeyComparator<ProcessorIdle, u8> for ProcessorComparator {
    fn extract_key(v: &ProcessorIdle) -> &u8 {
        &v.core_id
    }
}

#[derive(Debug)]
pub struct Processor {
    pub id: u8,
    pub cores: u8,
    pub name: String,
    pub temperature: i32,
    pub frequency: f32,
    pub load: f32,
}


impl Processor {
    fn new(id: u8, vcores: u8, name: String, temp: i32, freq: f32, load: f32) -> Processor {
        Processor {
            id: id,
            cores: vcores,
            name: name,
            temperature: temp,
            frequency: freq,
            load: load,
        }
    }
}

lazy_static! {
    static ref CACHE: Mutex<HashIndexed<ProcessorIdle, u8, ProcessorComparator>> = Mutex::new(HashIndexed::new());
}

fn get_cpu_folder() -> Vec<u8> {
    let output = Command::new("ls")
        .arg("/sys/devices/platform/coretemp.0/hwmon/")
        .output().unwrap();

    output.stdout
}

fn get_cpu_temp_info(core_id: u8) -> String {
    // if core id == 0 get temp from /sys/devices/platform/coretemp.0/hwmon/hwmon[0123]/temp2_input
    // else if core id == 1 get temp from /sys/devices/platform/coretemp.0/hwmon/hwmon[0123]/temp3_input
    // else ...
    let temp_input_num = core_id + 2;

    // Because hwmon may change dir name (hwmon2, hwmon3, etc) in hwmon directory
    // I need to read hwmon dir, filter dir by "hwmon" string and join dir to
    // path
    let join_path = String::from_utf8(get_cpu_folder()).unwrap();
    let path = format!("/sys/devices/platform/coretemp.0/hwmon/{}/temp{}_input", join_path.trim(), temp_input_num);

    utils::get_file_info(&path)
}

fn get_cpu_load(core_id: u8) -> f32 {
    let info = utils::get_file_info("/proc/stat");

    let cpu_id = format!("cpu{}", core_id);
    let (mut last_idle, mut last_total) = (0.00, 0.00);

    if !CACHE.lock().unwrap().contains(&core_id) {
        CACHE.lock().unwrap().insert(ProcessorIdle {idle: last_idle, total: last_total, core_id: core_id});
    }
    match CACHE.lock().unwrap().get(&core_id) {
        Some(info) => {
            last_idle = info.idle;
            last_total = info.total;
        }
        None => {}
    }

    for line in info.lines() {
        if line.starts_with(&cpu_id) {
            // Vectors ouput:
            // ["name", "user",   "nice",  "system", "idle",   "iowait", "irq", "softirq", "steal", "guest", "guest_nice"]
            // ["cpu2", "271368", "12444", "59293", "2342965", "212079", "0", "1795", "0", "0", "0"] == 2888744
            let mut fields: Vec<&str> = line.split(" ").collect();
            // Remove first vec, because it always string "cpu"
            //
            // Vectors output:
            // ["user",   "nice",  "system", "idle",   "iowait", "irq", "softirq", "steal", "guest", "guest_nice"]
            // ["271368", "12444", "59293", "2342965", "212079", "0", "1795", "0", "0", "0"]
            fields.remove(0);
            // Remove guest and guest_nice since they are already included in user and nice
            // See: http://unix.stackexchange.com/q/178045/20626
            //
            // Vectors output:
            // ["user",   "nice",  "system", "idle",   "iowait", "irq", "softirq", "steal"]
            // ["271368", "12444", "59293", "2342965", "212079", "0", "1795", "0"]
            fields.pop();
            fields.pop();
            // Calculate idle time as sum of idle and iowait times
            //
            // Vectors output:
            // ["user",   "nice",  "system", "irq", "softirq", "steal", "guest", "guest_nice"]
            // ["271368", "12444", "59293", "0", "1795", "0", "0", "0"]
            let idle = fields[3].parse::<f32>().unwrap() + fields[4].parse::<f32>().unwrap();
            let total = utils::vec_sum(fields);

            // Get delta idle and total time from previous numbers
            let (idle_delta, total_delta) = (idle - last_idle, total - last_total);

            CACHE.lock().unwrap().replace(ProcessorIdle {idle: idle, total: total, core_id: core_id});

            // Compute processor usage percentage
            // See: https://rosettacode.org/wiki/Linux_CPU_utilization
            let expect: f32 = 100.00f32 * (1.00f32 - idle_delta / total_delta);

            return expect;
        }
    }
    // Always return initial float number, because compiler create a error
    // if return value is none
    0.00
}

pub fn cpu_information() -> Vec<Processor> {
    let info = utils::get_file_info("/proc/cpuinfo");

    let mut processors: Vec<Processor> = Vec::new();

    let proc_vec = info.trim().split("\n\n");
    for iter in proc_vec {
        let mut id: u8 = 0;
        let mut vcores: u8 = 0;
        let mut name: String = String::new();
        let mut temp: i32 = 0;
        let mut freq: f32 = 0.00;
        let mut load: f32 = 0.00;

        let param_vec = iter.split("\n");
        for iter2 in param_vec {
            let v: Vec<&str> = iter2.split(":").collect();
            let param_name = v[0].trim();
            let param_value = v[1].trim();

            if param_name.starts_with("processor") {
                id = param_value.parse::<u8>().unwrap();
                load = get_cpu_load(id);
            } else if param_name.starts_with("cpu cores") {
                vcores = param_value.parse::<u8>().unwrap();
            } else if param_name.starts_with("model name") {
                name = String::from(param_value);
            } else if param_name.starts_with("core id") {
                let value: u8 = param_value.parse::<u8>().unwrap();

                let temperature = get_cpu_temp_info(value);
                // CPU temperature in millidegree Celsium
                // See: https://www.kernel.org/doc/Documentation/hwmon/sysfs-interface
                temp = temperature.trim().parse::<i32>().unwrap() / 1000;
            } else if param_name.starts_with("cpu MHz") {
                freq = param_value.parse::<f32>().unwrap();
            }
        }

        // Create new Processor information and push to processors vector
        processors.push(Processor::new(id, vcores, name, temp, freq, load));
    }

    processors
}
