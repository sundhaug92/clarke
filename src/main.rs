use dotenvy::dotenv;
use std::io::{Write, Read, Result};
use serde_json;
use serde_json::json;


fn IsValidChecklistDir(path: &str) -> bool {
    // Does the path even exist?
    
    let path = std::path::Path::new(path);
    if !path.exists() || !path.is_dir() {
        return false; // Path does not exist or is not a directory
    }

    // Does it contain one of the expected subdirectories?
    let expected_subdirs = ["Mercury", "Gemini", "LunarModule", "CommandModule", "SpaceShuttle", "Vostok"];

    for subdir in expected_subdirs.iter() {
        let subdir_path = std::path::Path::new(path).join(subdir);
        if subdir_path.exists() && subdir_path.is_dir() {
            return true; // Found a valid subdirectory
        }
    };
    false // No valid subdirectory found
}


fn main() {
    dotenv().ok();
    // Load environment variables from .env file

    if let Err(e) = dotenvy::dotenv() {
        eprintln!("Failed to load .env file: {}", e);
        println!("Where are checklists stored? ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim();

        let input = &input.replace("\\", "\\\\");

        let mut file = std::fs::File::create(".env").expect("Failed to create .env file");
        writeln!(file, "CHECKLISTS_DIR=\"{}\"", &input).expect("Failed to write to .env file");
        file.flush().expect("Failed to flush file");
    }

    let checklists_dir = std::env::var("CHECKLISTS_DIR").expect("CHECKLISTS_DIR not set");
    if !IsValidChecklistDir(&checklists_dir) {
        eprintln!("Invalid CHECKLISTS_DIR: {}", checklists_dir);
        // Delete the .env file if it exists
        if std::path::Path::new(".env").exists() {
            std::fs::remove_file(".env").expect("Failed to delete .env file");
        }
        println!("Expected subdirectories: Mercury, Gemini, LunarModule, CommandModule, SpaceShuttle, Vostok");
        std::process::exit(1);
    }

    // Iterate through the subdirectories to find the checklists
    let subdirs = ["Mercury", "Gemini", "LunarModule", "CommandModule", "SpaceShuttle", "Vostok"];

    let mut checklists = Vec::new();
    for subdir in subdirs.iter() {
        let subdir_path = std::path::Path::new(&checklists_dir).join(subdir);
        if subdir_path.exists() && subdir_path.is_dir() {
            // Each checklist is a subdirectory in the subdir
            if let Ok(entries) = std::fs::read_dir(&subdir_path) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        // Load the checklist.json file in the subdirectory
                        let checklist_path = entry.path().join("checklist.json");
                        if checklist_path.exists() && checklist_path.is_file() {
                            let mut file = std::fs::File::open(&checklist_path)
                                .expect("Failed to open checklist.json file");
                            let mut contents = String::new();
                            file.read_to_string(&mut contents)
                                .expect("Failed to read checklist.json file");

                            // Parse the JSON content
                            match serde_json::from_str::<serde_json::Value>(&contents) {
                                Ok(json) => {
                                    checklists.push((entry.file_name().to_string_lossy().to_string(), json));
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse JSON in {}: {}", checklist_path.display(), e);
                                }
                            }
                        } else {
                            eprintln!("Checklist file not found: {}", checklist_path.display());
                        }
                    }
                }
            } else {
                eprintln!("Failed to read directory: {}", subdir_path.display());
            }
        }
    }
    if checklists.is_empty() {
        eprintln!("No checklists found in the specified directories.");
        std::process::exit(1);
    }

    let mut spacecraftSelected = -1;
    loop {
        match spacecraftSelected {
            -1 => {
                println!("Select a spacecraft:");
                println!("Q: Exit");
                println!("");
                println!("0. Command Module");
                println!("1: Lunar Module");
                println!("2: Gemini");
                println!("3: Mercury");
                println!("4: Space Shuttle");

                let mut input = String::new();
                std::io::stdin().read_line(&mut input).expect("Failed to read line");
                let input = input.trim();
                if input.eq_ignore_ascii_case("q") {
                    break;
                }
                match input.parse::<i32>() {
                    Ok(num) if num >= 0 && num < 5 as i32 => {
                        spacecraftSelected = num;
                    }
                    _ => {
                        println!("Invalid selection. Please enter a number between 0 and {} or 'Q' to exit.", 5 - 1);
                        spacecraftSelected = -1; // Reset selection
                    }
                }
            }
            _ => {
                let mut spacecraftChecklists = Vec::new();
                for (name, json) in &checklists {
                    let checklistSpacecraftId = json.get("Spacecraft").unwrap().to_string().parse::<i32>().unwrap_or(-1);
                    if checklistSpacecraftId != spacecraftSelected {
                        continue; // Skip this checklist if it doesn't match the selected spacecraft
                    }
                    let checklistGroup = json.get("Group").unwrap().to_string();
                    let checklistName = json.get("Name").unwrap().to_string();

                    spacecraftChecklists.push((name.clone(), checklistGroup.to_string(), checklistName.to_string(), json.clone()));
                }
                
                
                if spacecraftChecklists.is_empty() {
                    println!("No checklists found for the selected spacecraft.");
                    spacecraftSelected = -1; // Reset selection
                    continue;
                }
                println!("Select a checklist for the selected spacecraft:");
                println!("Q: Exit");
                for (i, (name, group, checklistName, _)) in spacecraftChecklists.iter().enumerate() {
                    println!("{}. {} - {}", i + 1, group, checklistName);
                }
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).expect("Failed to read line");
                let input = input.trim();
                if input.eq_ignore_ascii_case("q") {
                    break;
                }
                match input.parse::<usize>() {
                    Ok(num) if num > 0 && num <= spacecraftChecklists.len() => {
                        let selected_checklist = &spacecraftChecklists[num - 1];
                        let checklist_json = &selected_checklist.3;

                        for step in checklist_json.get("Steps").unwrap().as_array().unwrap() {
                            match step.get("Type") {
                                Some(step_type_val) => {
                                    let step_type = step_type_val.as_i64().unwrap_or(-1) as i32;
                                    match step_type  {
                                        0 | 7 => {
                                            println!("{}", step.get("Text").unwrap().as_str().unwrap_or("").trim());

                                            if step_type == 0 {
                                                let mut dummy_input = String::new();
                                                std::io::stdin().read_line(&mut dummy_input).expect("Failed to read line");
                                            }
                                        }
                                        1|2 => {
                                            println!("Performing action: {}", step.get("Description").unwrap().as_str().unwrap_or("").trim());
                                            let setID: i32 = step.get("SetID").unwrap().as_i64().unwrap_or(-1) as i32;
                                            let toPosID: i32 = step.get("ToPosID").unwrap().as_i64().unwrap_or(-1) as i32;
                                            // TODO send command to the spacecraft
                                        }
                                        _ => {
                                            println!("{}", step);
                                            todo!("Handle other step types here: {}", step_type);
                                        }
                                    }
                                }
                                None => {
                                    eprintln!("Step type not found in checklist step: {:?}", step);
                                    continue; // Skip this step if type is not found
                                }
                            }
                        }
                    }
                    _ => {
                        println!("Invalid selection. Please enter a number between 1 and {} or 'Q' to exit.", spacecraftChecklists.len());
                    }
                }
            }
        }
    }
}
