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
        let home = std::env::home_dir().unwrap();
        let home = home.to_string_lossy();
        let home = home.as_ref();
        let username = &CONFIG.username;
        let virtual_path = format!("@@{}", username);
        let name = path.to_str().unwrap().to_string();
        let name = name.replace(home, &virtual_path);

        let mut dir = Directory {
            name,
            files: vec![],
        };

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                visit_dir(&entry.path(), dirs)?;
            } else {
                let metadata = entry.metadata()?;
                let name = entry.file_name().to_str().unwrap().to_string();

                dir.files.push(File {
                    name,
                    size: metadata.len(),
                    extension: entry
                        .path()
                        .extension()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                    // TODO
                    attributes: vec![],
                });
            }
        }

        dirs.push(dir);
    } else {
        error!("{:?} is not a directory", path)
    }

    Ok(())
}
