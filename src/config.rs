use std::{
    env::home_dir,
    fs::{File, create_dir_all},
    io::{BufReader, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::RuxError;

/// Light/dark appearance, or follow the desktop (where iced supports it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemePreference {
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];
}

impl fmt::Display for ThemePreference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::System => f.write_str("System"),
            Self::Light => f.write_str("Light"),
            Self::Dark => f.write_str("Dark"),
        }
    }
}

/// Last UI and playback state, restored on the next launch.
///
/// Fields missing from a saved session take their default, so a config
/// written before a field existed still loads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionState {
    pub last_tab_file: Option<PathBuf>,
    pub track_index: usize,
    pub tempo_percentage: u32,
    pub master_volume: f32,
    pub measure_index: usize,
    pub beat_index: usize,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            last_tab_file: None,
            track_index: 0,
            tempo_percentage: 100,
            master_volume: 1.0,
            measure_index: 0,
            beat_index: 0,
            selection_start: None,
            selection_end: None,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    tabs_folder: Option<PathBuf>,
    #[serde(default)]
    session: SessionState,
    #[serde(default)]
    theme: ThemePreference,
    sound_font_file: Option<PathBuf>,
}

impl Config {
    pub fn session(&self) -> &SessionState {
        &self.session
    }

    pub fn set_session(&mut self, session: SessionState) -> Result<(), RuxError> {
        self.session = session;
        self.save_config()
    }
}

impl Config {
    // folder placed in $HOME directory
    const FOLDER: &'static str = ".config/ruxguitar";

    pub fn get_tabs_folder(&self) -> Option<PathBuf> {
        self.tabs_folder.clone()
    }

    pub fn set_tabs_folder(&mut self, new_tabs_folder: Option<PathBuf>) -> Result<(), RuxError> {
        if self.tabs_folder == new_tabs_folder {
            Ok(())
        } else {
            self.tabs_folder = new_tabs_folder;
            self.save_config()
        }
    }

    pub fn theme_preference(&self) -> ThemePreference {
        self.theme
    }

    pub fn set_theme_preference(&mut self, theme: ThemePreference) -> Result<(), RuxError> {
        if self.theme == theme {
            Ok(())
        } else {
            self.theme = theme;
            self.save_config()
        }
    }

    pub fn sound_font_file(&self) -> Option<&PathBuf> {
        self.sound_font_file.as_ref()
    }

    pub fn set_sound_font_file(&mut self, path: Option<PathBuf>) -> Result<(), RuxError> {
        if self.sound_font_file == path {
            Ok(())
        } else {
            self.sound_font_file = path;
            self.save_config()
        }
    }

    fn get_base_path() -> Result<PathBuf, RuxError> {
        let home = home_dir()
            .ok_or_else(|| RuxError::ConfigError("Could not find home directory".to_string()))?;
        let path = home.join(Self::FOLDER);
        Ok(path)
    }

    fn get_path() -> Result<PathBuf, RuxError> {
        let base = Self::get_base_path()?;
        Ok(base.join("config.json"))
    }

    /// Creates config if it does not exist
    pub fn read_config() -> Result<Self, RuxError> {
        let base_path = Self::get_base_path()?;
        if !base_path.exists() {
            create_dir_all(base_path)?;
        }
        let config_path = Self::get_path()?;
        if !config_path.exists() {
            // create empty config
            Config::default().save_config()?;
        }
        let file = File::open(&config_path)?;
        let reader = BufReader::new(file);
        match serde_json::from_reader(reader) {
            Ok(config) => Ok(config),
            Err(err) => {
                log::warn!(
                    "Could not read local configuration {}: {err}, resetting to default",
                    config_path.display()
                );
                let default_config = Config::default();
                default_config.save_config()?;
                Ok(default_config)
            }
        }
    }

    /// Assumes the config folder exists
    pub fn save_config(&self) -> Result<(), RuxError> {
        let config_path = Self::get_path()?;
        let json = serde_json::to_string_pretty(self).map_err(|err| {
            RuxError::ConfigError(format!("Could not save local configuration {err:}"))
        })?;
        let mut file = File::create(config_path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_defaults_when_missing_from_json() {
        let json = r#"{"tabs_folder":null}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.session.tempo_percentage, 100);
        assert!(config.session.last_tab_file.is_none());
    }

    #[test]
    fn preferences_default_when_missing_from_json() {
        let json = r#"{"tabs_folder":null}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.theme_preference(), ThemePreference::System);
        assert!(config.sound_font_file().is_none());
    }

    #[test]
    fn session_fields_default_when_missing_from_json() {
        // a session saved before some of its fields existed
        let json = r#"{"tabs_folder":"/tabs","session":{"last_tab_file":"/tabs/song.gp5","tempo_percentage":75}}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.tabs_folder, Some(PathBuf::from("/tabs")));
        assert_eq!(
            config.session.last_tab_file,
            Some(PathBuf::from("/tabs/song.gp5"))
        );
        assert_eq!(config.session.tempo_percentage, 75);
        assert!((config.session.master_volume - 1.0).abs() < f32::EPSILON);
        assert_eq!(config.session.measure_index, 0);
        assert!(config.session.selection_start.is_none());
    }
}
