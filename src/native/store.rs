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

fn jam_dir() -> (std::path::PathBuf, &'static str) {
    let music = home().join("Music");
    if music.is_dir() {
        (music.join("melody-quack"), "Music/melody-quack")
    } else {
        (home().join("melody-quack-jams"), "melody-quack-jams")
    }
}

pub fn jam_place() -> String {
    jam_dir().1.to_string()
}

pub fn jam_name() -> String {
    let (dir, _) = jam_dir();
    let mut n = 1;
    while dir.join(format!("jam-{n:02}.wav")).exists() {
        n += 1;
    }
    format!("jam-{n:02}")
}

pub fn save_jam(bytes: &[u8]) -> Option<String> {
    save_jam_as(bytes, &jam_name())
}

pub fn save_jam_as(bytes: &[u8], name: &str) -> Option<String> {
    let (dir, shown) = jam_dir();
    std::fs::create_dir_all(&dir).ok()?;
    let base = crate::ui::file_stem(name);
    let mut file = format!("{base}.wav");
    let mut n = 2;
    while dir.join(&file).exists() {
        file = format!("{base} {n}.wav");
        n += 1;
    }
    std::fs::write(dir.join(&file), bytes).ok()?;
    Some(format!("{shown}/{file}"))
}
