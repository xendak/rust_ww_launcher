use std::{fs, io};
use std::path::Path;
use std::io::Write;
use std::process::{Command, Stdio};


use config::{Config, ConfigError, File};
use serde::Deserialize;
use rfd::FileDialog;

// Global variables 
const PAK_PATH: &str = r"Client\Content\Paks\~mod";
const BIN_PATH: &str = r"Client\Binaries\Win64";
const PROCESS_NAME: &str = r"Client-Win64-Shipping.exe";

#[derive(Debug, Deserialize, Clone)]
struct AppConfig {
    mods_folder: String,
    game_folder: String,
    modded_launcher: String,
}

impl AppConfig {
    fn new() -> Result<Self, ConfigError> {
        let mut settings = Config::default();

        // Try to read the configuration file
        match settings.merge(File::with_name("config.ini")) {
            Ok(_) => {
                let config: Result<AppConfig, ConfigError> = settings.try_into();
                match config {
                    Ok(c) => {
                        println!("Config read successfully: {:?}", c);
                        Ok(c)
                    }
                    Err(e) => {
                        eprintln!("Error deserializing config: {}, therefore we'll delete the config file.", e);
                        // Delete the config.ini file if deserialization fails
                        if Path::new("config.ini").exists() {
                            fs::remove_file("config.ini").expect("Unable to delete corrupt config.ini file");
                        }
                        Err(e)
                    }
                }
            }
            Err(_) => {
                // If reading the config file fails, prompt the user for paths
                let mods_folder = Self::select_folder("Select the Mods Folder");
                let game_folder = Self::select_folder("Select the Game Folder");
                let modded_launcher = Self::select_file("Select the Modded Launcher");

                let config = AppConfig {
                    mods_folder,
                    game_folder,
                    modded_launcher,
                };

                // Save the configuration to a file
                Self::save_config(&config);

                println!("Config created and saved: {:?}", config);

                Ok(config)
            }
        }
    }

    fn select_folder(prompt: &str) -> String {
        FileDialog::new()
            .set_title(prompt)
            .pick_folder()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| {
                println!("No folder selected, exiting.");
                std::process::exit(1);
            })
    }

    fn select_file(prompt: &str) -> String {
        FileDialog::new()
            .set_title(prompt)
            .add_filter("Executable", &["exe"])
            .pick_file()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| {
                println!("No file selected, exiting.");
                std::process::exit(1);
            })
    }

    fn save_config(config: &AppConfig) {
        let content = format!(
            "mods_folder = \"{}\"\ngame_folder = \"{}\"\nmodded_launcher = \"{}\"\n",
            config.mods_folder.replace("\\", "\\\\"), config.game_folder.replace("\\", "\\\\"), config.modded_launcher.replace("\\", "\\\\")
        );
        fs::write("config.ini", content).expect("Unable to write config file");
    }
}

fn main() {
    // Get the input argument
    println!("Rust -- WW Launcher");
    kill_process();

    

    let app_config = match AppConfig::new() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // delete mods folder forr both options instead, easier to recopy than change 2 dirs at once when moddding files
    let mods_folder_path = Path::new(&app_config.game_folder).join(PAK_PATH);
    if mods_folder_path.exists() {
        fs::remove_dir_all(&mods_folder_path).expect("Failed to delete /~mod folder");
        println!("Deleted /~mod folder");
    }


    loop {
        println!("Choose an option: NEWVER");
        println!("1. Original (type '1', 'o', or 'original')");
        println!("2. Modded (type '2', 'm', 'mod', or 'modded')");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");

        match input.trim().to_lowercase().as_str() {
            "1" | "o" | "original" => {
                println!("Executing original option...");
                let game_folder_path = Path::new(&app_config.game_folder).join(BIN_PATH);
                for file_to_delete in &["reboot.bat", "imgui.ini", "Pipsi-WW.cfg"] {
                    let file_path = game_folder_path.join(file_to_delete);
                    if file_path.exists() {
                        fs::remove_file(&file_path).expect("Failed to delete file");
                        println!("Deleted {}", file_path.display());
                    }
                }
                execute_binary_detached(&(app_config.game_folder.clone() + "\\" + BIN_PATH + "\\" + PROCESS_NAME));
                break;
            }
            "2" | "m" | "mod" | "modded" => {
                println!("Executing modded option..., changed");
                if let Err(err) = copy_files(&app_config.mods_folder, &app_config.game_folder) {
                    println!("Error copying files: {}", err);
                }
                execute_binary_detached(&app_config.modded_launcher);
                break;
            }
            _ => {
                println!("Invalid input. Please choose either '1' or '2'.");
            }
        }
    }
}

fn kill_process() {
    // Execute the taskkill command to forcefully end the process
    let output = Command::new("taskkill")
        .args(&["/F", "/IM", PROCESS_NAME])
        .output()
        .expect("Failed to execute taskkill command");

    if output.status.success() {
        println!("Process '{}' terminated successfully.", PROCESS_NAME);
    } else {
        println!("Error terminating process");
    }
}

fn execute_binary_detached(binary_path: &str) {
    if binary_path.contains("Client-Win64-Shipping") {
        println!("we are running default, therefore append -fileopenlog");
        let status = Command::new(binary_path)
        .arg("-fileopenlog")
        .stdout(Stdio::null()) // Redirect stdout to /dev/null (or NUL on Windows)
        .stderr(Stdio::null()) // Redirect stderr to /dev/null (or NUL on Windows)
        .spawn()
        .expect("Failed to execute binary");
    } else {
    let status = Command::new(binary_path)
        .stdout(Stdio::null()) // Redirect stdout to /dev/null (or NUL on Windows)
        .stderr(Stdio::null()) // Redirect stderr to /dev/null (or NUL on Windows)
        .spawn()
        .expect("Failed to execute binary");
    }
    // The spawned process runs independently in the background
    // You won't receive status information here
    println!("Binary started as a detached process.");
}

fn execute_binary(binary_path: &str) {
    let status = Command::new(binary_path)
        .status()
        .expect("Failed to execute binary");

    if status.success() {
        println!("Binary executed successfully.");
    } else {
        println!("Error executing binary.");
    }
}

fn copy_files(source: &str, destination: &str) -> Result<(), Box<dyn std::error::Error>> {
    let source_path = Path::new(source);
    let destination_path = Path::new(destination).join(PAK_PATH);

    // Create the destination folder if it doesn't exist
    if !destination_path.exists() {
        fs::create_dir_all(&destination_path)?;
    }

    // Copy files from source to destination
    for entry in fs::read_dir(source_path)? {
        let entry = entry?;
        let source_file = entry.path();
        let destination_file = destination_path.join(entry.file_name());

        fs::copy(&source_file, &destination_file)?;
        println!("Copied {} to {}", source_file.display(), destination_file.display());
    }

    Ok(())
}
