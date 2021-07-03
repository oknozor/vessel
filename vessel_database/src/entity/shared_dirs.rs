use crate::settings::CONFIG;
use soulseek_protocol::peers::p2p::shared_directories::{Directory, File, SharedDirectories};
use std::io;
use std::path::Path;

pub fn get_shared_directories() -> io::Result<SharedDirectories> {
    let paths = &CONFIG.shared_directories;
    let mut shared_directories = SharedDirectories { dirs: vec![] };

    for path in paths {
        visit_dir(path, &mut shared_directories.dirs)?;
    }

    Ok(shared_directories)
}

fn visit_dir(path: &Path, dirs: &mut Vec<Directory>) -> io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let mut dir = Directory {
                name: path.to_str().unwrap().to_string(),
                files: vec![],
            };

            if entry.path().is_dir() {
                visit_dir(&entry.path(), dirs)?;
            } else {
                let metadata = entry.metadata()?;
                dir.files.push(File {
                    name: entry.file_name().to_str().unwrap().to_string(),
                    size: metadata.len(),
                    extension: entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .split('.')
                        .last()
                        .unwrap()
                        .to_string(),
                    // TODO
                    attributes: vec![],
                })
            }

            dirs.push(dir);
        }
    } else {
        error!("{:?} is not a directory", path)
    }

    Ok(())
}
