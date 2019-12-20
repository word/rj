use std::process;

mod cmd;
mod errors;
mod zfs;

use reqwest;
use std::fs;
use std::path::Path;
use tar::Archive;
use xz2::read::XzDecoder;

fn main() {
    let j_ds = String::from("zroot/jails");
    let bj_ds = format!("{}/basejail", &j_ds);
    let bj_dir = String::from("/jails/basejail");
    let release = String::from("12.0-RELEASE");
    let dists = ["base", "lib32"];

    for set in vec![&j_ds, &bj_ds] {
        println!("Creating jail data set {}", set);
        zfs::create_ds(&set).unwrap_or_else(|err| {
            eprintln!("ERROR: {}", err);
            process::exit(1);
        });
    }

    let url = format!(
        "http://ftp.uk.freebsd.org/pub/FreeBSD/releases/amd64/amd64/{}/{}.txz",
        release, dists[1]
    );
    let response = reqwest::get(&url).unwrap();
    let decompressor = XzDecoder::new(response);
    let mut archive = Archive::new(decompressor);
    archive.set_preserve_permissions(true);

    for (_, file) in archive.entries().unwrap().enumerate() {
        let mut file = file.unwrap();
        let dst_path = Path::new(&bj_dir).join(file.path().unwrap());

        if file.header().link_name().unwrap().is_some() {
            if dst_path.is_file() {
                println!("removing link {:?}", &dst_path);
                fs::remove_file(&dst_path).unwrap();
            }
        }

        file.unpack_in(&bj_dir).unwrap();
    }
}
