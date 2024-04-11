use std::path::PathBuf;
use rand::Rng;
use clap::{Parser, Subcommand};
use chrono::Utc;
use std::hash::{Hash, Hasher, DefaultHasher};
use serde::{Serialize, Deserialize};
use serde_json;
use predicates::str::contains;
use assert_cmd::Command as StdCommand;
use scan_fmt::scan_fmt;

#[derive(Parser)]
#[command(name = "notus", about="Notes for us", long_about = "A DND notes app with insane functionality (dungeon generation, dice rolling, markdown support, exporting)")]
struct Cli {
    #[arg( required = false)]    
    note: Option<String>,

    #[command(subcommand)]
    command: Commands, 

}

#[derive(Subcommand)]
enum Commands {
    /// Rolls a die
    Roll {
        #[arg()]
        input: String,
    },
    /// Note global comand
    Note {
        #[clap(short = 'n', long = "new")]
        new: bool,
        #[clap(short = 'a', long = "active")]
        active: bool,
        #[clap(short = 'e', long = "edit")]
        edit: bool,
        #[arg()]
        name: String,
        #[clap(short = 't', long = "tags")]
        tags: String,
    }

}

#[derive(Serialize, Deserialize,  Debug)]
struct Note {
   date: String,
   name: String,
   path: PathBuf,
   note_type: NoteType,
   tags: Vec<String>,   
   id: u32,
   content: String,
}
impl Note {
    fn new(mut name: String, mut path: PathBuf, note_type: NoteType, tags: Vec<String>, notes: &mut Vec<NoteID>) -> Self {
        let now = Utc::now().to_string();
        let content = String::new();
        // Ensure the path is within the "notes" folder and has a .json extension
        path = PathBuf::from("notes").join(path).with_extension("json");
        let mut counter = 1;
        let original_file_stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        let oldname = name.clone();
        while path.exists() {
            
            let new_path = path.with_file_name(format!("{}{}.json", original_file_stem, counter));
            path = new_path;
            name = oldname.clone()+&counter.to_string();
            counter += 1;
        }
        let id = Note::generate_note_id(&name, &path);
        let noteid=NoteID{
            name: name.clone(),
            id,
            path: path.clone(),
        };
        notes.push(noteid);

        println!("{:?}", *notes);
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
    fn save(&self){
      // Note { date: 2024-03-17T00:13:41.073965Z, name: "Hello", path: "Hello", note_type: Note, tags: [], ID: Id(258311098) }
        // make dir if not there
        if let Some(dir) = self.path.parent() {
            if !dir.exists() {
                std::fs::create_dir_all(&dir).unwrap();
            }
        }
        let serialized = serde_json::to_string_pretty(&self).unwrap();
        //try and write
        if let Err(e) = std::fs::write(&self.path, &serialized) {
            eprintln!("Failed to write to file: {}", e);
        }
        print!("Created: {}", &self.name)
    }
    fn generate_note_id(name: &String, path: &PathBuf) -> u32 {
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        let hash = hasher.finish();
        let id = hash as u32;
        id
    }

} 
#[derive(Debug, Serialize, Deserialize)]
enum NoteType {
    Note,
    Sheet,
    Character
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct NoteID {
    name: String,
    id: u32,
    path: PathBuf,
}


fn main() {
    let cli = Cli::parse();
    let rng = rand::thread_rng();
    let mut notes = Vec::new();
    if let Some(note) = cli.note {
        println!("{}", note);
    }
    else {
       match &cli.command {
        Commands::Roll { input } => {
            let (num, die) = scan_fmt!(
                input, "{}d{}", i32, i32).unwrap();
            roll(num, die)
        },
        Commands::Note { new, active, edit, name, tags } => {
            // Handle the Note command here
            // For example, create a new note if the 'new' flag is true
            if *new {
                let note = Note::new(name.clone(), PathBuf::from(name), NoteType::Note, tags.split(',').map(String::from).collect(), &mut notes );
                note.save(

                );
            }
        },
        _ =>{ println!("you need to write something man") }
    }  
    }
   
}  
   

fn roll(num: i32, die: i32) {
    let mut rng = rand::thread_rng();
    println!("Rolling {}d{}:", num, die);
    let mut total = 0;
    for _ in 0..num {
        let roll: i32 = rng.gen_range(1..=die);
        println!("Rolled: {}", roll);
        total += roll;
    }
    println!("Total: {}", total);
}

#[test]
fn test_roll_command() {
    let mut rng = rand::thread_rng();
    let mut cmd = StdCommand::cargo_bin("notus").unwrap();
    //randomize the roll
    let num = rng.gen_range(1..=6);
    let die = rng.gen_range(1..=20);
    cmd.arg("roll").arg(format!("{}d{}", num, die));
    cmd.assert()
       .success()
       .stdout(predicates::str::contains(format!("Rolling {}d{}:", num, die)));
}