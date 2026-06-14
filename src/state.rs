use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

pub struct AppState {
    pub music: Music,
    pub bt: BtReceiver,
    pub settings: Settings,
    pub nav: Vec<Screen>,
}

pub enum Screen {
    MainMenu {
        selected: usize,
        items: Vec<MenuItem>,
    },

    Music {
        view: MusicView,
    },
    Bluetooth,
    Settings,
}

pub enum MusicView {
    List { selected: usize },
    NowPlaying,
}

#[derive(Clone)]
pub enum MenuItem {
    Music,
    Bluetooth,
    Settings,
}

pub struct Music {
    pub title: Option<String>,
    pub collection: Vec<PathBuf>,
    pub song_position: usize,
    pub is_playing: bool,
}

pub struct BtReceiver {
    pub discovered: Vec<BtDevice>,
    pub connected: Option<BtDevice>,
    pub scanning: bool,
    pub pairing_code: Option<String>,
}

pub struct BtDevice {
    pub name: String,
    pub address: String,
    pub paired: bool,
    pub signal_strength: Option<i8>,
}

pub struct Settings {
    pub volume: u8,
    pub brightness: u8,
    pub theme: Theme,
    pub auto_connect_bt: bool,
}

pub enum Theme {
    Light,
    Dark,
    System,
}

pub fn get_state() -> &'static Mutex<AppState> {
    STATE.get_or_init(|| Mutex::new(AppState::new()))
}

impl AppState {
    pub fn new() -> Self {
        Self {
            music: Music {
                title: None,
                collection: get_music("music").unwrap(),
                song_position: 0,
                is_playing: false,
            },
            bt: BtReceiver {
                discovered: Vec::new(),
                connected: None,
                scanning: false,
                pairing_code: None,
            },
            settings: Settings {
                volume: 50,
                brightness: 80,
                theme: Theme::System,
                auto_connect_bt: false,
            },
            nav: vec![Screen::MainMenu {
                selected: 0,
                items: vec![MenuItem::Music, MenuItem::Bluetooth, MenuItem::Settings],
            }],
        }
    }
}

fn get_music(path: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let dir = Path::new(path);
    let mut result = Vec::new();

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                let sub_files = get_music(&entry_path.to_string_lossy().into_owned())?;
                result.extend(sub_files);
            } else {
                result.push(entry_path);
            }
        }
    }

    Ok(result)
}
