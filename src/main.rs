use std::process;

mod errors;
mod cmd;
mod zfs;

fn main() {
    let _release = String::from("12-RELEASE");
    let jdataset = String::from("zroot/jails");
    let bjdataset = format!("{}/basejail", &jdataset);

    for set in vec![&jdataset, &bjdataset] {
        println!("Creating jail data set {}", set);
        zfs::create_ds(&set).unwrap_or_else(|err| {
            eprintln!("ERROR: {}", err);
            process::exit(1);
        });
    };

}
