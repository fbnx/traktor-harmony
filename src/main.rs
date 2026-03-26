use clap::{Parser, Subcommand};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use xml::reader::{EventReader, XmlEvent};

// ============ Configuration Constants ============

const KEYS: &[(&str, &str)] = &[
    ("0", "1d"), ("1", "8d"), ("2", "3d"), ("3", "10d"), ("4", "5d"), ("5", "12d"),
    ("6", "7d"), ("7", "2d"), ("8", "9d"), ("9", "4d"), ("10", "11d"), ("11", "6d"),
    ("12", "10m"), ("13", "5m"), ("14", "12m"), ("15", "7m"), ("16", "2m"), ("17", "9m"),
    ("18", "4m"), ("19", "11m"), ("20", "6m"), ("21", "1m"), ("22", "8m"), ("23", "3m"),
];

const PERFECT_MIX: f64 = 1.0;
const ADJACENT_MIX: f64 = 0.9;
const SCALE_CHANGE: f64 = 0.9;
const ENERGY_BOOST: f64 = 0.8;
const DIAGONAL_MIX: f64 = 0.8;
const JAWS_MIX: f64 = 0.5;
const MOOD_SHIFTER: f64 = 0.5;

// ============ CLI Arguments ============

#[derive(Parser)]
#[clap(name = "traktor-harmony")]
#[clap(about = "Automatically generate harmonic playlists from Traktor collections")]
struct Args {
    /// Path to collection/playlist NML file
    path: PathBuf,

    #[clap(subcommand)]
    command: Option<Commands>,

    /// Analyze playlist
    #[clap(long, global = true)]
    analyze: bool,

    /// Harmonize playlist
    #[clap(long, global = true)]
    harmonize: bool,

    /// Generate new playlist
    #[clap(long, global = true)]
    generate: bool,

    /// Print matching next track
    #[clap(long, global = true)]
    next: Option<String>,

    /// Output file path
    #[clap(long, global = true)]
    output: Option<PathBuf>,

    /// Playlist length
    #[clap(long, global = true)]
    length: Option<usize>,

    /// Start track filename
    #[clap(long, global = true)]
    start: Option<String>,

    /// Camelot wheel key (e.g., "1m", "5d")
    #[clap(long, global = true)]
    key: Option<String>,

    /// BPM value
    #[clap(long, global = true)]
    bpm: Option<u32>,
}

#[derive(Subcommand)]
enum Commands {
    Generate {
        #[clap(long)]
        output: PathBuf,
        #[clap(long)]
        length: usize,
        #[clap(long)]
        start: Option<String>,
        #[clap(long)]
        key: Option<String>,
        #[clap(long)]
        bpm: Option<u32>,
    },
}

// ============ TraktorCollection ============

struct TraktorCollection {
    tracks: HashMap<String, (String, u32)>,
    paths: HashMap<String, String>,
}

impl TraktorCollection {
    fn new() -> Self {
        Self {
            tracks: HashMap::new(),
            paths: HashMap::new(),
        }
    }

    fn build_camelot_wheel() -> HashMap<String, HashMap<String, f64>> {
        let mut wheel = HashMap::new();

        for i in 1..=12 {
            let major = format!("{}m", i);
            let minor = format!("{}d", i);
            let prev_major = if i > 1 { format!("{}m", i - 1) } else { "12m".to_string() };
            let next_major = if i < 12 { format!("{}m", i + 1) } else { "1m".to_string() };
            let prev_minor = if i > 1 { format!("{}d", i - 1) } else { "12d".to_string() };
            let next_minor = if i < 12 { format!("{}d", i + 1) } else { "1d".to_string() };
            let two_up_major = format!("{}m", if i < 11 { i + 2 } else { i - 10 });
            let two_up_minor = format!("{}d", if i < 11 { i + 2 } else { i - 10 });
            let jaws_major = format!("{}m", if i < 6 { i + 7 } else { i - 5 });
            let jaws_minor = format!("{}d", if i < 6 { i + 7 } else { i - 5 });
            let mood_minor = format!("{}d", if i > 3 { i - 3 } else { i + 9 });
            let mood_major = format!("{}m", if i > 3 { i - 3 } else { i + 9 });

            let mut major_map = HashMap::new();
            major_map.insert(major.clone(), PERFECT_MIX);
            major_map.insert(prev_major, ADJACENT_MIX);
            major_map.insert(next_major, ADJACENT_MIX);
            major_map.insert(minor.clone(), SCALE_CHANGE);
            major_map.insert(two_up_major, ENERGY_BOOST);
            major_map.insert(prev_minor, DIAGONAL_MIX);
            major_map.insert(jaws_major, JAWS_MIX);
            major_map.insert(mood_minor, MOOD_SHIFTER);
            wheel.insert(major, major_map);

            let mut minor_map = HashMap::new();
            minor_map.insert(minor.clone(), PERFECT_MIX);
            minor_map.insert(prev_minor, ADJACENT_MIX);
            minor_map.insert(next_minor, ADJACENT_MIX);
            minor_map.insert(major.clone(), SCALE_CHANGE);
            minor_map.insert(two_up_minor, ENERGY_BOOST);
            minor_map.insert(next_major, DIAGONAL_MIX);
            minor_map.insert(jaws_minor, JAWS_MIX);
            minor_map.insert(mood_major, MOOD_SHIFTER);
            wheel.insert(minor, minor_map);
        }

        wheel
    }

    fn read_collection(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let parser = EventReader::from_str(content);
        let mut in_entry = false;
        let mut in_location = false;
        let mut in_tempo = false;
        let mut in_key = false;
        let mut current_file: Option<String> = None;
        let mut current_volume: Option<String> = None;
        let mut current_dir: Option<String> = None;
        let mut current_bpm: Option<f64> = None;
        let mut current_key_val: Option<u32> = None;

        for e in parser {
            match e {
                Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                    match name.local_name.as_str() {
                        "ENTRY" => in_entry = true,
                        "LOCATION" if in_entry => {
                            in_location = true;
                            for attr in attributes {
                                match attr.name.local_name.as_str() {
                                    "FILE" => current_file = Some(attr.value),
                                    "VOLUME" => current_volume = Some(attr.value),
                                    "DIR" => current_dir = Some(attr.value),
                                    _ => {}
                                }
                            }
                        }
                        "TEMPO" if in_entry => {
                            in_tempo = true;
                            for attr in attributes {
                                if attr.name.local_name == "BPM" {
                                    current_bpm = attr.value.parse().ok();
                                }
                            }
                        }
                        "MUSICAL_KEY" if in_entry => {
                            in_key = true;
                            for attr in attributes {
                                if attr.name.local_name == "VALUE" {
                                    current_key_val = attr.value.parse().ok();
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(XmlEvent::EndElement { name }) => {
                    match name.local_name.as_str() {
                        "ENTRY" => {
                            if let (Some(file), Some(key_val), Some(bpm)) =
                                (current_file.take(), current_key_val.take(), current_bpm.take())
                            {
                                if let Some((_, key_str)) = KEYS.iter().find(|(k, _)| k == &key_val.to_string()) {
                                    let bpm_rounded = bpm.round() as u32;
                                    self.tracks.insert(file.clone(), (key_str.to_string(), bpm_rounded));

                                    if let (Some(vol), Some(dir)) = (current_volume.take(), current_dir.take()) {
                                        let full_path = format!("/Volumes/{}{}{}", vol, dir.replace(":", ""), file);
                                        self.paths.insert(file, full_path);
                                    }
                                }
                            }
                            in_entry = false;
                        }
                        "LOCATION" => in_location = false,
                        "TEMPO" => in_tempo = false,
                        "MUSICAL_KEY" => in_key = false,
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn calculate_transition_score(
        &self,
        wheel: &HashMap<String, HashMap<String, f64>>,
        first: &str,
        second: &str,
    ) -> f64 {
        if let (Some((key1, bpm1)), Some((key2, bpm2))) = (self.tracks.get(first), self.tracks.get(second)) {
            if key1 == key2 && bpm1 == bpm2 {
                return 1.0;
            }

            let key_value = wheel
                .get(key1)
                .and_then(|m| m.get(key2))
                .copied()
                .unwrap_or(0.0);

            let bpm_diff = (*bpm1 as i32 - *bpm2 as i32).abs() as f64;
            let bpm_value = 0.01 * bpm_diff * bpm_diff;

            key_value - bpm_value
        } else {
            -999.0
        }
    }

    fn generate_playlist(
        &self,
        wheel: &HashMap<String, HashMap<String, f64>>,
        start: &str,
        length: usize,
    ) -> Vec<String> {
        let mut playlist = vec![start.to_string()];
        let mut rng = rand::thread_rng();

        for _ in 0..length - 1 {
            let mut best_matches = Vec::new();
            let mut best_score = -999.0;

            for filename in self.tracks.keys() {
                if playlist.contains(filename) {
                    continue;
                }

                let score = self.calculate_transition_score(wheel, playlist.last().unwrap(), filename);
                if score > best_score {
                    best_score = score;
                    best_matches = vec![filename.clone()];
                } else if (score - best_score).abs() < 1e-9 {
                    best_matches.push(filename.clone());
                }
            }

            if let Some(next) = best_matches.choose(&mut rng) {
                println!("{} -> {:?}: {}", playlist.last().unwrap(), best_matches, best_score);
                playlist.push(next.clone());
            }
        }

        playlist
    }

    fn write_m3u(&self, playlist: &[String], output: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let content = playlist
            .iter()
            .filter_map(|filename| self.paths.get(filename))
            .map(|path| format!("{}\r", path))
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(output, content)?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let content = fs::read_to_string(&args.path)?;
    let mut collection = TraktorCollection::new();
    collection.read_collection(&content)?;

    println!("Loaded {} tracks from collection", collection.tracks.len());

    let wheel = TraktorCollection::build_camelot_wheel();

    if args.generate {
        let start_track = if let Some(s) = args.start {
            s
        } else if let (Some(k), Some(b)) = (args.key.clone(), args.bpm) {
            collection
                .tracks
                .iter()
                .find(|(_, (track_key, track_bpm))| track_key == &k && track_bpm == &b)
                .map(|(filename, _)| filename.clone())
                .ok_or("No track found with given key and BPM")?
        } else {
            return Err("Must provide either start track or key and bpm".into());
        };

        let length = args.length.ok_or("Length is required for generate")?;
        let output = args.output.ok_or("Output path is required for generate")?;

        let playlist = collection.generate_playlist(&wheel, &start_track, length);
        collection.write_m3u(&playlist, &output)?;
        println!("\nPlaylist written to {:?}", output);
    } else if let Some(track) = args.next {
        let playlist = collection.generate_playlist(&wheel, &track, 2);
        println!("Next tracks after {}: {:?}", track, playlist);
    } else {
        eprintln!("Please specify a command (--generate or --next)");
    }

    Ok(())
}
