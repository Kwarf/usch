use std::{path::{Path, PathBuf}, io::{Write, Read}, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{binary, time::{SeekableTimeSource, TimeSource}};

pub struct Tracker {
    bpm: u32,
    tracks: Vec<Track>,
    path: Option<PathBuf>,
    pub time: SeekableTimeSource,
}

impl Tracker {
    pub fn new(bpm: u32, path: Option<&Path>, track_names: &[&'static str]) -> Tracker {
        let tracks = track_names
            .iter()
            .map(|x| Track
            {
                name: x.to_string(),
                values: Vec::new(),
            })
            .collect::<Vec<Track>>();

        Tracker { bpm, tracks, path: path.map(|x| x.to_path_buf()), time: SeekableTimeSource::now() }
    }

    pub fn current_row(&self) -> u32 {
        (self.time.elapsed().as_secs_f32() * self.rows_per_second() + 0.5) as u32
    }

    pub fn get_time_from_row(&self, row: u32) -> Duration {
        Duration::from_secs_f32(row as f32 / self.rows_per_second())
    }

    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn set_value(&mut self, track_name: &'static str, row: u32, value: f32) {
        self.tracks
            .iter_mut()
            .find(|x| x.name == track_name)
            .expect(&format!("Failed to find a track named {}", track_name))
            .set_value(row, value);
        
        self.save();
    }

    fn rows_per_second(&self) -> f32 {
        (self.bpm as f32 / 60f32) * 4f32
    }

    fn save(&self) {
        match &self.path {
            Some(path) => {
                let json = serde_json::to_string_pretty(&self.tracks).unwrap();
                std::fs::write(path, json).unwrap();
            },
            None => (),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Track {
    name: String,
    values: Vec<Key>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Key {
    row: u32,
    value: f32,
}

impl Key {
    fn read(mut reader: impl Read) -> Key {
        Key {
            row: binary::read(&mut reader),
            value: binary::read(&mut reader),
        }
    }

    fn write(&self, mut writer: impl Write) {
        binary::write(&mut writer, &self.row);
        binary::write(&mut writer, &self.value);
    }
}

impl Track {
    fn new(name: String) -> Track {
        Track { name, values: Vec::new() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_value(&self, row: u32) -> Option<f32> {
        self.values
            .iter()
            .find(|x| x.row == row)
            .map(|x| x.value)
    }

    fn set_value(&mut self, row: u32, value: f32) {
        match self.values.iter_mut().find(|x| x.row == row) {
            Some(key) => key.value = value,
            None => {
                // This could be improved, but not a priority since it should not run in releases
                self.values.push(Key {
                    row,
                    value,
                });
                self.values.sort_by_key(|x| x.row);
            }
        }
    }

    fn read(mut reader: impl Read) -> Track {
        let name_len: u16 = binary::read(&mut reader);
        let mut name_buf = Vec::with_capacity(name_len as usize);
        reader.read_exact(&mut name_buf).unwrap();

        let num_keys = binary::read(&mut reader);
        let mut values = Vec::with_capacity(num_keys as usize);
        for _ in 0..num_keys {
            values.push(Key::read(&mut reader));
        }

        Track { name: String::from_utf8(name_buf).unwrap(), values }
    }

    fn write(&self, mut writer: impl Write) {
        binary::write(&mut writer, &(self.name.len() as u16));
        binary::write_bytes(&mut writer, self.name.as_bytes());

        for value in &self.values {
            value.write(&mut writer);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn key_can_save_and_load_binary() {
        let input = Key {
            row: 0xDEADBEEF,
            value: 1337.69f32,
        };

        // Write to binary
        let data = Vec::new();
        let mut cursor = Cursor::new(data);
        input.write(&mut cursor);

        // Read back into struct
        cursor.set_position(0);
        let result = Key::read(cursor);

        assert_eq!(input.row, result.row);
        assert_eq!(input.value, result.value);
    }
}
