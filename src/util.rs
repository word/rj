use anyhow::Result;
use indicatif::HumanBytes;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;
use tar::Archive;
use xz2::read::XzDecoder;

// fetch an xz archive using http and extract it to a destination directory
pub fn fetch_extract(url: &str, dest: &Path) -> Result<()> {
    let response = reqwest::get(url)?;
    let decompressor = XzDecoder::new(response);
    let mut archive = Archive::new(decompressor);
    archive.set_preserve_permissions(true);

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner().template("{spinner:.blue} extracted {pos} files, {msg}"),
    );

    // Remove any matching hard/soft links at the destination before extracting
    // files.  This is necessary because Tar won't overwrite links and panics
    // instead.  This effectively will overwrite any existing files and links at
    // the destination.
    for entry in archive.entries()? {
        let mut entry = entry?;
        let file_dest = dest.join(entry.path()?);

        // Check if the archived file is a link (hard and soft)
        if entry.header().link_name().unwrap().is_some() {
            // Check if the link exist at the destination already.  Using symlink_metadata
            // here becasue is_file returns false for symlinks.  This will match
            // any file, symlink or hardlink.
            if file_dest.symlink_metadata().is_ok() {
                // remove the link so it can be extracted later
                // println!("removing link {:?}", &file_dest);
                fs::remove_file(&file_dest)?;
            }
        }
        entry.unpack_in(dest)?;

        // Update the spinner
        pb.inc(1);
        let mut entry_size = 0;
        if file_dest.is_file() {
            entry_size = fs::metadata(file_dest)?.len();
        }
        pb.set_message(&format!(
            "{}",
            HumanBytes(entry.raw_file_position() + entry_size)
        ))
    }
    pb.finish_at_current_pos();
    Ok(())
}
