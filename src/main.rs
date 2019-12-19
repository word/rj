use std::process;

mod errors;
mod cmd;
mod zfs;

use reqwest;
use xz2::read::XzDecoder;
use std::fs::File;
use tar::Archive;
use std::path::Path;

fn main() {
    let j_ds = String::from("zroot/jails");
    let bj_ds = format!("{}/basejail", &j_ds);
    let bj_dir = String::from("/jails/basejail");
    let release = String::from("12.0-RELEASE");
    let dists = ["base", "lib32" ];

    for set in vec![&j_ds, &bj_ds] {
        println!("Creating jail data set {}", set);
        zfs::create_ds(&set).unwrap_or_else(|err| {
            eprintln!("ERROR: {}", err);
            process::exit(1);
        });
    };

    let url = format!("http://ftp.uk.freebsd.org/pub/FreeBSD/releases/amd64/amd64/{}/{}.txz", release, dists[0]);
    let response = reqwest::get(&url).unwrap();
    let decompressor = XzDecoder::new(response);
    let mut archive = Archive::new(decompressor);
    archive.set_preserve_permissions(true);
    for (_, file) in archive.entries().unwrap().enumerate() {
        let mut file = file.unwrap();
        let dst_path = Path::new(&bj_dir).join(file.path().unwrap());
        println!("{:?}", dst_path);
        // if dst_path.is_file() {
        //     println!("exists");
        // }
        file.unpack_in(&bj_dir).unwrap();
    }
    // archive.unpack("/tmp/test2").unwrap();
    // archive.unpack(bj_dir).unwrap();

}
