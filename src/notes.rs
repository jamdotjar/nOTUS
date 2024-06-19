use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::hash::{Hash, DefaultHasher, Hasher};
use chrono::Utc;
use std::fs;
use std::fmt;
use dirs::{data_dir, home_dir};

#[derive(Serialize, Deserialize, Debug)]
pub struct Note {
   date: String,
   pub name: String,
   pub path: PathBuf,
   note_type: NoteType,
   tags: Vec<String>, 
   id: NoteID,
   pub content: String,
}

impl Note {
    pub fn new(mut name: String, mut path: PathBuf, note_type: NoteType, tags: Vec<String>, notes: &mut Vec<NoteID>) -> Self {
        let now = Utc::now().to_string();
        let content = String::new();
        // Ensure the base directory is 'notes/'
        path = get_notes_path().join(&path);
        path.set_extension("json");  // Set the extension before the loop
        let original_file_stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        let mut counter = 0;
        while path.exists() {
            counter += 1;
            let new_file_name = format!("{}{}.json", original_file_stem, counter);
            path = PathBuf::from("notes").join(&new_file_name); }
        if counter > 0 {
            name = format!("{}{}", original_file_stem, counter);
        }
        let id = Note::generate_note_id(&name, &path);
        notes.push(id.clone());
        set_active_note(notes, &id.name);
        Self {
            name,
            path,
            note_type,
            tags,
            date: now,
            id: id,
            content: content,
        }
    }

    pub fn save(&self) {
        // make dir if not there
        if let Some(dir) = self.path.parent() {
            if !dir.exists() {
                fs::create_dir_all(&dir).unwrap();
            }
        }
        let serialized = serde_json::to_string_pretty(&self).unwrap();
        // try and write
        if let Err(e) = fs::write(&self.path, &serialized) {
            eprintln!("Failed to write to file: {}", e);
        }
        println!("Created: {}", &self.name)
    }

    fn generate_note_id(name: &String, path: &PathBuf) -> NoteID {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        path.hash(&mut hasher);
        let hash = hasher.finish();
        let id = NoteID { name: name.clone(), id: hash as u32, path: path.clone(), active: true };
        id
    }
}
pub fn load_notes_list() -> Vec<NoteID>{
    let mut notes = Vec::new();
    if let Some(dir) = PathBuf::from(".conf/.notes").parent() {
        if !dir.exists() {
            println!("No notes to load");
            return notes;
        }
    }
    // Load notes from .conf/.notes
    let path = get_data_path().join("notes.nsv");
    if let Ok(bytes) = fs::read(&path) {
        notes = bincode::deserialize(&bytes).unwrap_or_else(|_| Vec::new());
    }
    notes
}

pub fn save_notes_list(notes: &Vec<NoteID>){
    let encoded: Vec<u8> = bincode::serialize(notes).unwrap();
    let path = get_data_path().join("notes.nsv");
    if let Some(dir) = path.parent() { //create directory if not present
        if !dir.exists() { 
            std::fs::create_dir_all(dir).unwrap();
        }
    }
    fs::write(path, &encoded).expect("Couldn't save note data");
}
#[derive(Debug, Serialize, Deserialize)]
pub enum NoteType {
    Note,
    Sheet,
    Character
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NoteID {
    pub name: String,
    id: u32,
    pub path: PathBuf,
    pub active: bool,
}

impl NoteID {
    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
}
impl fmt::Display for NoteID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name) // Assuming NoteID has a single field that implements Display
    }
}
pub fn set_active_note(notes: &mut Vec<NoteID>, active_note_name: &str) {
   for note in notes.iter_mut() {
        note.active = note.name == active_note_name;
    }
}




pub fn export() {
    
}
fn get_notes_path() -> PathBuf {
    let mut path = home_dir().expect("Could not find home directory");
    path.push(".notus");
    path.push("notes");
    path
}
fn get_data_path() -> PathBuf {
    let mut path = data_dir().expect("Could not find data directory");
    path.push("notus");
    path.push("notes");
    path
}