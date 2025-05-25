use dotenvy::dotenv;
use std::io::{Write, Read};
use serde_json;
use reentryudp::domain_models::*;
use std::net::UdpSocket;



fn is_valid_checklist_dir(path: &str) -> bool {
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
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();

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
    if !is_valid_checklist_dir(&checklists_dir) {
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

    let mut spacecraft_selected = -1;
    loop {
        match spacecraft_selected {
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
                        spacecraft_selected = num;
                    }
                    _ => {
                        println!("Invalid selection. Please enter a number between 0 and {} or 'Q' to exit.", 5 - 1);
                        spacecraft_selected = -1; // Reset selection
                    }
                }
            }
            _ => {
                let mut spacecraft_checklists = Vec::new();
                for (name, json) in &checklists {
                    let checklist_spacecraft_id = json.get("Spacecraft").unwrap().to_string().parse::<i32>().unwrap_or(-1);
                    if checklist_spacecraft_id != spacecraft_selected {
                        continue; // Skip this checklist if it doesn't match the selected spacecraft
                    }
                    let checklist_group = json.get("Group").unwrap().to_string();
                    let checklist_name = json.get("Name").unwrap().to_string();

                    spacecraft_checklists.push((name.clone(), checklist_group.to_string(), checklist_name.to_string(), json.clone()));
                }
                
                
                if spacecraft_checklists.is_empty() {
                    println!("No checklists found for the selected spacecraft.");
                    spacecraft_selected = -1; // Reset selection
                    continue;
                }
                println!("Select a checklist for the selected spacecraft:");
                println!("Q: Exit");
                for (i, (_, group, checklist_name, _)) in spacecraft_checklists.iter().enumerate() {
                    println!("{}. {} - {}", i + 1, group, checklist_name);
                }
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).expect("Failed to read line");
                let input = input.trim();
                if input.eq_ignore_ascii_case("q") {
                    break;
                }
                match input.parse::<usize>() {
                    Ok(num) if num > 0 && num <= spacecraft_checklists.len() => {
                        let selected_checklist = &spacecraft_checklists[num - 1];
                        let checklist_json = &selected_checklist.3;

                        for step in checklist_json.get("Steps").unwrap().as_array().unwrap() {
                            match step.get("Type") {
                                Some(step_type_val) => {
                                    let step_type = step_type_val.as_i64().unwrap_or(-1) as u32;
                                    match step_type  {
                                        0 | 7 => {
                                            println!("{}", step.get("Text").unwrap().as_str().unwrap_or("").trim());

                                            if step_type == 0 {
                                                let mut dummy_input = String::new();
                                                std::io::stdin().read_line(&mut dummy_input).expect("Failed to read line");
                                            }
                                        }
                                        1|2|3|5 => {
                                            println!("Performing action: {}", step.get("Description").unwrap().as_str().unwrap_or("").trim());
                                            let set_id: u32 = step.get("SetID").unwrap().as_i64().unwrap_or(-1) as u32;
                                            let to_pos_id: u32 = step.get("ToPosID").unwrap().as_i64().unwrap_or(-1) as u32;
                                            
                                            let messagetype:u32 = match step_type {

                                                1 => MessageType::SetSwitch as u32,
                                                2 => MessageType::SetCircuitBreaker as u32,
                                                3 => MessageType::SetSelector as u32,
                                                5 => MessageType::SetHandle as u32,
                                                _ => {
                                                    eprintln!("Unknown step type: {}", step_type);
                                                    eprintln!("Step: {:?}", step);
                                                    todo!("Handle unknown step type: {}", step_type);
                                                }
                                            };

                                            let target_craft: u32 = match spacecraft_selected {
                                                0 => 2, // Command Module
                                                1 => 3, // Lunar Module
                                                2 => 1, // Gemini
                                                3 => 0, // Mercury
                                                4 => 4, // Space Shuttle
                                                5 => 5, // Vostok

                                                _ => {
                                                    todo!("Invalid spacecraft selected: {}", spacecraft_selected);
                                                }
                                            };

                                            let data_packet = DataPacket {
                                                TargetCraft: target_craft,
                                                MessageType: messagetype,
                                                ID: set_id,
                                                ToPos: to_pos_id,
                                            };

                                            let serialized_packet = serde_json::to_string(&data_packet).unwrap();
                                            socket.send_to(serialized_packet.as_bytes(), "127.0.0.1:8051").unwrap();
                                            // println!("Sent data packet: {:?}", data_packet);
                                            // Sleep to let the system process the command
                                            std::thread::sleep(std::time::Duration::from_millis(100));
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
                        println!("Invalid selection. Please enter a number between 1 and {} or 'Q' to exit.", spacecraft_checklists.len());
                    }
                }
            }
        }
    }
}
