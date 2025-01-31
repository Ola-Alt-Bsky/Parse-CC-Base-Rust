use std::{
    collections::HashSet, 
    fs::{read_to_string, create_dir, File}, 
    io::{stdin, Write, Result}, 
    path
};
use serde_json::{Value, Map, to_string_pretty};

fn parse_to_json(file_lines: &String) -> Value {
    let mut info: Map<String, Value> = Map::new();

    let mut last_season: String = String::from("Null");
    let mut last_episode: String = String::from("Null");
    let mut last_attribute: String = String::from("Null");
    let mut last_content: String = String::from("Null");
    let mut last_specific: String = String::from("Null");

    for line in file_lines.lines() {
        let starts_with_star: bool = line.starts_with("*");
        let starts_with_space: bool = line.starts_with(" ");
        let amount_leading_space: usize = line.len() - line.trim_start().len();

        if !(starts_with_star || starts_with_space) { // Season
            let trimmed_line: &str = line.strip_prefix('\u{FEFF}').unwrap_or(&line);
            info.insert(trimmed_line.to_string(), Value::Object(Map::new()));
            last_season = trimmed_line.to_string();
        }
        else if starts_with_star {  // Episode
            let trimmed_line: String = line.replace("*", "").trim().to_string();
            last_episode = trimmed_line.clone();

            if let Some(season_obj) = info
                .get_mut(&last_season)
                .and_then(|v| v.as_object_mut())
            {
                season_obj.insert(trimmed_line, Value::Object(Map::new()));
            }
        }
        else if starts_with_space && amount_leading_space == 3 { // Attribute
            let trimmed_line: String = line.replace("*", "").trim().to_string();
            last_attribute = trimmed_line.clone();

            if let Some(episode_obj) = info
                .get_mut(&last_season)
                .and_then(|v| v.as_object_mut())
                .and_then(|season_obj| season_obj.get_mut(&last_episode))
                .and_then(|ev| ev.as_object_mut())
            {   
                match trimmed_line.eq("Songs") {
                    true => {episode_obj.insert(trimmed_line, Value::Object(Map::new()));}
                    false => {episode_obj.insert(trimmed_line, Value::Array(vec![]));}
                }
            }
        }
        else if starts_with_space && amount_leading_space == 6 { // Content
            let trimmed_line: String = line.replace("*", "").trim().to_string();
            last_content = trimmed_line.clone();
            
            if let Some(episode_obj) = info
                .get_mut(&last_season)
                .and_then(|v| v.as_object_mut())
                .and_then(|season_obj| season_obj.get_mut(&last_episode))
                .and_then(|ev| ev.as_object_mut()) 
            {
                match episode_obj.get_mut(&last_attribute) {
                    Some(Value::Object(attribute_obj)) if last_attribute.eq("Songs") => {
                        attribute_obj.insert(trimmed_line, Value::Object(Map::new()));
                    }
                    Some(Value::Array(attribute_attr)) => {
                        attribute_attr.push(Value::String(trimmed_line.clone()));
                    }
                    _ => {}
                }
            }
        }
        else if starts_with_space && amount_leading_space == 9 { // Specific
            let trimmed_line: String = line.replace("*", "").trim().to_string();
            last_specific = trimmed_line.clone();

            if let Some(attribute_obj) = info
                .get_mut(&last_season)
                .and_then(|v| v.as_object_mut())
                .and_then(|season_obj| season_obj.get_mut(&last_episode))
                .and_then(|ev| ev.as_object_mut())
                .and_then(|episode_obj| episode_obj.get_mut(&last_attribute))
                .and_then(|av| av.as_object_mut())
            {
                let cv = attribute_obj.get_mut(&last_content);
                if last_content.eq("Scene Specific") {
                    if let Some(Value::Object(content_obj)) = cv {
                        content_obj.insert(trimmed_line, Value::Object(Map::new()));
                    }
                }
                else {
                    attribute_obj.insert(last_content.clone(), Value::String(trimmed_line));
                }
            }
        }
        else if starts_with_space && amount_leading_space == 12 { // Specific
            let trimmed_line: String = line.replace("*", "").trim().to_string();

            if let Some(content_obj) = info
                .get_mut(&last_season)
                .and_then(|v| v.as_object_mut())
                .and_then(|season_obj| season_obj.get_mut(&last_episode))
                .and_then(|ev| ev.as_object_mut())
                .and_then(|episode_obj| episode_obj.get_mut(&last_attribute))
                .and_then(|av| av.as_object_mut())
                .and_then(|attribute_obj| attribute_obj.get_mut(&last_content))
                .and_then(|cv| cv.as_object_mut())
            {
                content_obj.insert(last_specific.clone(), Value::String(trimmed_line));
            }
        }
    }

    // Remove extra stuff
    info.remove("Chapter Template");
    info.remove("Extra Songs");

    return Value::Object(info);
}

fn get_items(info: &mut Value, item: u8) -> HashSet<String> {
    // 1 for characters, 2 for locations, 3 for songs

    let mut item_list: HashSet<String> = HashSet::new();

    info.as_object().map(|info_obj| {
        for season in info_obj.values() {
            season.as_object().map(|season_obj| {
                for episode in season_obj.values() {
                    episode.as_object().map(|episode_obj| {
                        Some(if let Some(item_obj) = episode_obj.get(match item {
                            1 => "Characters",
                            2 => "Locations",
                            3 => "Songs",
                            _ => ""
                        }) {
                            if let Some(item_array) = item_obj.as_array() {
                                for item in item_array {
                                    if let Some(item_str) = item.as_str() {item_list.insert(item_str.to_string());}
                                }
                            }
                            else if let Some (song_obj) = item_obj.as_object() {
                                if let Some(song) = song_obj
                                    .get("Intro Song")
                                    .and_then(|intro| intro.as_str()) 
                                {
                                    item_list.insert(song.to_string());
                                }

                                if let Some(song) = song_obj
                                    .get("Outro Song")
                                    .and_then(|outro| outro.as_str()) 
                                {
                                    item_list.insert(song.to_string());
                                }

                                if let Some(scene_songs) = song_obj
                                    .get("Scene Specific")
                                    .and_then(|scene| scene.as_object())
                                {
                                    for song_val in scene_songs.values() {
                                        if let Some (song) = song_val.as_str() {item_list.insert(song.to_string());}
                                    }
                                }
                            }
                        })
                    });
                }
            });
        }
    });

    return item_list;
}


fn main() -> Result<()>{
    // Read input from a .txt file
    println!("Welcome! You will need to enter in the location of your file.");
    println!("Enter in the ABSOLUTE file path of the base txt file: ");

    let mut file_path: String = String::new();
    stdin().read_line(&mut file_path).expect("Could not read the line.");
    file_path = file_path.replace('"', "").trim().to_string();
    
    // Try to retrieve the text from the file
    let file_lines: String = read_to_string(&file_path).expect("Could not read the line.");

    // Parse and convert to JSON
    let mut parsed_json = parse_to_json(&file_lines);
    
    // Retrieve a list of characters, locations, and songs
    let characters: HashSet<String> = get_items(&mut parsed_json, 1);
    let locations: HashSet<String> = get_items(&mut parsed_json, 2);
    let songs: HashSet<String> = get_items(&mut parsed_json, 3);


    // Save the parsed JSON information to a folder
    let parent_dir = path::Path::new(&file_path).parent().unwrap();
    let output_buf = parent_dir.join("Output");
    let output_dir = output_buf.as_path();
    if !output_dir.exists() {let _ = create_dir(&output_dir);}

    let mut json_file = File::create(&output_dir.join("Casual Roleplay.json")).expect("Could not create file.");
    json_file.write_all(to_string_pretty(&parsed_json).expect("Could not format json object").as_bytes()).expect("Could not save json file");
    println!("Parsed JSON has been saved to {}/{}", &output_dir.display(), "Casual Roleplay.json");

    let mut character_file = File::create(&output_dir.join("Casual Roleplay Characters.txt")).expect("Could not create file.");
    for item in &characters {
        character_file.write_all(item.as_bytes())?;
        character_file.write_all(b"\n")?;
    }
    println!("Characters have been saved to {}/{}", &output_dir.display(), "Casual Roleplay Characters.txt");

    let mut locations_file = File::create(&output_dir.join("Casual Roleplay Locations.txt")).expect("Could not create file.");
    for item in &locations {
        locations_file.write_all(item.as_bytes())?;
        locations_file.write_all(b"\n")?;
    }
    println!("Locations have been saved to {}/{}", &output_dir.display(), "Casual Roleplay Locations.txt");

    let mut songs_file = File::create(&output_dir.join("Casual Roleplay Songs.txt")).expect("Could not create file.");
    for item in &songs {
        songs_file.write_all(item.as_bytes())?;
        songs_file.write_all(b"\n")?;
    }
    println!("Songs have been saved to {}/{}", &output_dir.display(), "Casual Roleplay Songs.txt");

    Ok(())
}
