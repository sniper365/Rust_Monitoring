use std::process::Command;

use regex::Regex;

#[derive(Debug)]
pub struct HDD {
    pub name: String,
    pub size: String,
    pub fstype: String,
    pub mount_point: String
}

impl HDD {
    fn new(name: &str, size: &str, fstype: &str, mount: &str) -> HDD {
        HDD {
            name: name.to_string(),
            size: size.to_string(),
            fstype: fstype.to_string(),
            mount_point: mount.to_string()
        }
    }
}

lazy_static! {
    static ref RE: Regex = Regex::new(r"[└─├─]+").unwrap();
}

fn get_info() -> Vec<u8> {
    let output = Command::new("lsblk")
        .arg("-o")
        .arg("NAME,SIZE,FSTYPE,TYPE,MOUNTPOINT")
        .output()
        .expect("HDD command not found");

    if !output.status.success() {
        println!("HDD command lsblk return error");
    }

    output.stdout
}

pub fn hdd_information() -> Vec<HDD> {
    let info = get_info();
    let info = String::from_utf8_lossy(&info);

    let mut hdd: Vec<HDD> = Vec::new();

    for (i, line) in info.lines().enumerate() {
        // Don't need table header
        if i == 0 {
            continue;
        }

        let mut line_vec: Vec<&str> = line.split(" ").collect();
        line_vec.retain(|&x| x != "");

        // if line_vec less than 4, need to add fstype equal empty string
        // and add mount_point equal empty string
        if line_vec.len() < 4 {
            line_vec.insert(2, "");
            line_vec.insert(4, "");
        }

        let disk_type = line_vec[3].trim();
        // if type of disk == part or disk or swap
        if !disk_type.starts_with("rom") {
            let search = line_vec[0].trim();
            let name = RE.replace_all(search, "");
            let size = line_vec[1].trim();
            let fstype = line_vec[2].trim();
            let mount = line_vec[4].trim();

            hdd.push(HDD::new(&name, size, fstype, mount));
        }
    }

    hdd
}
