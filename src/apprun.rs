use std::{env::set_current_dir, ffi::OsString};

use anyhow::{bail, Context};
use walkdir::WalkDir;

fn main() -> anyhow::Result<()> {
    let here_dir = std::env::current_exe()?;
    let parent = here_dir
        .parent()
        .with_context(|| format!("{} has no parent directory", &here_dir.display()))?;
    std::env::set_current_dir(parent)?;
    let mut loader_path = None;
    for entry in WalkDir::new(parent).into_iter() {
        let entry = entry?;
        if entry.file_type().is_file()
            && entry.file_name().to_string_lossy().starts_with("ld-linux")
        {
            loader_path = Some(entry.into_path());
        }
    }
    if loader_path.is_none() {
        bail!("Failed to find a loader (name started with `ld-linux`)");
    }
    let loader_path = loader_path.unwrap();
    std::env::set_var(
        "LD_LIBRARY_PATH",
        format!("{}/usr/lib/:{}/usr/lib/i386-linux-gnu/:{}/usr/lib/x86_64-linux-gnu/:{}/usr/lib32/:{}/usr/lib64/:{}/lib/:{}/lib/i386-linux-gnu/:{}/lib/x86_64-linux-gnu/:{}/lib32/:{}/lib64/{}", parent.display(), parent.display(), parent.display(), parent.display(), parent.display(), parent.display(), parent.display(), parent.display(), parent.display(), parent.display(), if let Ok(ldlibpath) = std::env::var("LD_LIBRARY_PATH") { ":".to_string() + &ldlibpath } else { String::new() }),
    );
    std::env::set_var(
        "XDG_DATA_DIRS",
        format!(
            "XDG_DATA_DIRS={}:{}",
            parent.join("usr/share").display(),
            std::env::var("XDG_DATA_DIRS").unwrap_or(String::new())
        ),
    );
    let argv0 = std::env::args_os().next().unwrap();
    let mut args_list = vec![
        loader_path.as_os_str().to_owned(),
        OsString::from("--argv0"),
        argv0,
        parent.join("usr/bin/bin").as_os_str().to_owned(),
    ];
    args_list.extend(std::env::args().skip(1).map(OsString::from));
    let owd = std::env::var("OWD").unwrap();
    set_current_dir(owd).unwrap();
    let err = exec::execvp(loader_path, args_list);
    eprintln!("Error: {}", err);
    Ok(())
}
