use std::path::PathBuf;

fn home() -> PathBuf {
    std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
}

fn save_path() -> PathBuf {
    home().join(".melody-quack").join("save.txt")
}

pub fn read() -> Option<String> {
    std::fs::read_to_string(save_path()).ok()
}

pub fn write(text: &str) {
    let path = save_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let temp = path.with_extension("tmp");
    if std::fs::write(&temp, text).is_ok() {
        let _ = std::fs::rename(&temp, &path);
    }
}

pub fn clear() {
    let _ = std::fs::remove_file(save_path());
}

pub fn restart() {
    if let Ok(exe) = std::env::current_exe() {
        if std::process::Command::new(exe).args(std::env::args().skip(1)).spawn().is_ok() {
            std::process::exit(0);
        }
    }
}

pub fn save_jam(bytes: &[u8]) -> Option<String> {
    let music = home().join("Music");
    let (dir, shown) = if music.is_dir() {
        (music.join("melody-quack"), "Music/melody-quack")
    } else {
        (home().join("melody-quack-jams"), "melody-quack-jams")
    };
    std::fs::create_dir_all(&dir).ok()?;
    let mut n = 1;
    while dir.join(format!("jam-{n:02}.wav")).exists() {
        n += 1;
    }
    let name = format!("jam-{n:02}.wav");
    std::fs::write(dir.join(&name), bytes).ok()?;
    Some(format!("{shown}/{name}"))
}
