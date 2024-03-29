use serde::{Serialize, Deserialize};
use std::str;
use std::io::prelude::*;
use crate::color::Color::{self, Blue, Brown, Green, Orange, Purple, Red, Yellow};
use crate::connection::Connection;
use crate::tile::painter::Painter;
use crate::tile::splitter::Splitter;
use crate::tile::trainsink::Trainsink;
use crate::tile::trainsource::Trainsource;
use crate::tile::rock::Rock;
use crate::tile::Tile;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PositionedTile {
    pub tile: Tile,
    pub x: u8,
    pub y: u8,
}
pub type LevelInfo = Vec<PositionedTile>;
pub type LevelProgress = (LevelInfo, bool); // the bool represents whether the play has won

#[derive(Serialize, Deserialize, Debug)]
pub struct Level {
    pub level_info: LevelInfo,
    pub current_progress: LevelProgress,
    pub name: String,
    pub num_stars: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct City {
    name: String, 
    levels:Vec<Level>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LevelManager(Vec<City>);

fn convert_string_to_color(s: &str) -> Option<Color> {
    // return none if s is invalid input
    match s {
        "red" => Some(Red),
        "blue" => Some(Blue),
        "yellow" => Some(Yellow),
        "purple" => Some(Purple),
        "green" => Some(Green),
        "orange" => Some(Orange),
        "brown" => Some(Brown),
        _ => None,
    }
}

const SAVE_FILENAME: &str = ".trainyard_saved_progress.json";

fn convert_string_to_dir(s: &str) -> Option<u8> {
    // return none if s was an invalid direction.
    match s {
        "up" => Some(0),
        "right" => Some(1),
        "down" => Some(2),
        "left" => Some(3),
        _ => None,
    }
}

impl LevelManager {
    pub fn new() -> LevelManager {
        let f = std::fs::File::open(SAVE_FILENAME);
        if let Ok(mut file) = f {
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("something went wrong reading the file");
            let new_level_manager: LevelManager = serde_json::from_str(&contents).unwrap();
            return new_level_manager;
        }
        println!("Unable to find previous save file");


        let mut lm = LevelManager(vec![]);
        let info_str = str::from_utf8(include_bytes!("../assets/levels.txt")).unwrap();
        let mut arr = info_str
            .split("\n")
            .enumerate()
            .map(|(index, line)| (index+1, line))
            .filter(|(_, line)| !(line.starts_with("//") || line.is_empty()));
        
        
        loop {
            match arr.next() {
                None => {
                    return lm;
                }
                Some((line_num, line)) => {
                    if !line.starts_with("CITY:") {
                        panic!("expected line {} to start with `CITY:`", line_num)
                    }
                    let city_name = &line[5..];
                    let mut city: City = City{name: city_name.to_owned(), levels: vec![]};
                    loop {
                        // load all levels within a city
                        match arr.next() {
                            None => {
                                panic!("unexpected EOF");
                            }
                            Some((line_num, line)) => {
                                if line == "----" {
                                    break;
                                }

                                let fields: Vec<&str> = line.split(":").collect();
                                if fields.len() != 2 {
                                    panic!("expected two arguments separated by a `:` at line {}", line_num)
                                }
                                let level_name = fields[0];
                                let num_stars: u32 = fields[1].parse()
                                    .expect(&format!("expected second argument to be an integer at line {line_num}"));

                                let level_info = LevelManager::extract_level_from_lines(&mut arr);

                                city.levels.push(Level {
                                    level_info,
                                    current_progress: (vec![], false),
                                    name: level_name.to_owned(),
                                    num_stars,
                                });

                            }
                        }
                    }
                    lm.0.push(city);
                }
            }
        }
    }

    fn extract_level_from_lines<'a, I>(arr: &mut I) -> LevelInfo 
        where I: Iterator<Item = (usize, &'a str)>, 
    {
        let mut level_info: LevelInfo = vec![];

        loop {
            match arr.next() {
                None => panic!("unexpected EOF"),
                Some((line_num, line)) => {
                    if line == "---" {
                        break;
                    } 

                    let x: u8 = line[2..3].parse()
                        .expect(&format!("expected a single-digit integer at the 3rd character of line {line_num}"));
                    let y: u8 = line[4..5].parse()
                        .expect(&format!("expected a single-digit integer at the 5th character of line {line_num}"));

                    if line.starts_with("+ ") {
                        // handle a new trainsource
                        let fields: Vec<&str> = line.split(" ").collect();
                        assert!(fields.len() == 4, "wrong number of spaces at line {line_num} (expected 3 spaces)");
                        let colors: Vec<Color> = fields[2].split(",")
                            .map(convert_string_to_color)
                            .map(|o| o.expect(&format!("invalid color at line {line_num}")))
                            .collect();
                        let dir = convert_string_to_dir(fields[3])
                            .expect(&format!("invalid direction in line {line_num}"));
                        level_info.push(PositionedTile {
                            tile: Tile::Trainsource(Trainsource::new(colors, dir)),
                            x,
                            y,
                        });
                    } else if line.starts_with("o ") {
                        // handle a new trainsink
                        let fields: Vec<&str> = line.split(" ").collect();
                        assert!(fields.len() == 4, "wrong number of spaces at line {line_num} (expected 3 spaces)");
                        let colors: Vec<Color> = fields[2].split(",")
                            .map(convert_string_to_color)
                            .map(|o| o.expect(&format!("invalid color at line {line_num}")))
                            .collect();
                        let dirs = fields[3]
                            .split(",")
                            .map(convert_string_to_dir)
                            .map(|o| o.expect(&format!("invalid direction at line {line_num}")));
                        let mut border_state = [false, false, false, false];
                        for dir in dirs {
                            border_state[dir as usize] = true;
                        }
                        level_info.push(PositionedTile {
                            tile: Tile::Trainsink(Trainsink::new(colors, border_state)),
                            x,
                            y,
                        });
                    } else if line.starts_with("* ") {
                        let mut positions = line[2..].split(" ");
                        while let Some(position) = positions.next() {
                            let x: u8 = position[0..1].parse()
                                .expect(&format!("invalid single-digit number on line {line_num}"));
                            let y: u8 = position[2..3].parse()
                                .expect(&format!("invalid single-digit number on line {line_num}"));
                            level_info.push(PositionedTile {
                                tile: Tile::Rock(Rock::new()),
                                x,
                                y,
                            });
                        }
                    } else if line.starts_with("p ") {
                        // handle a new painter
                        let fields: Vec<&str> = line.split(" ").collect();
                        assert!(fields.len() == 4, "wrong number of spaces at line {line_num} (expected 3 spaces)");

                        let color = convert_string_to_color(fields[2]).expect(&format!("invalid color at line {line_num}"));
                        let dirs: Vec<u8> = fields[3].split(",")
                            .map(convert_string_to_dir)
                            .map(|o| o.expect(&format!("invalid direction at line {line_num}")))
                            .collect();
                        
                        assert!(dirs.len() == 2, "expected exactly two directions for painter at line {line_num}");

                        level_info.push(PositionedTile {
                            tile: Tile::Painter(Painter::new(
                                Connection {
                                    dir1: dirs[0] as u8,
                                    dir2: dirs[1] as u8,
                                },
                                color,
                            )),
                            x,
                            y,
                        });
                    } else if line.starts_with("s ") {
                        // handle a new splitter
                        let fields: Vec<&str> = line.split(" ").collect();
                        assert!(fields.len() == 3, "wrong number of spaces at line {line_num} (expected 2 spaces)");
                        
                        let dir = convert_string_to_dir(fields[2]).expect(&format!("invalid direction at line {line_num}"));
                        level_info.push(PositionedTile {
                            tile: Tile::Splitter(Splitter::new(dir)),
                            x,
                            y,
                        });
                    } else {
                        panic!("line {line_num} begins with an invalid character: {line}");
                    }
                }
            }
        }

        level_info
    }


    pub fn get_city_names(&self) -> Vec<String> {
        self.0.iter().map(|city| city.name.clone()).collect()
    }
    pub fn get_names_in_city(&self, city_name: &str) -> Vec<String> {
        let city = self.0.iter().find(|city| city.name == city_name).unwrap();
        city.levels.iter().map(|level| level.name.clone()).collect()
    }
    pub fn get_level(&self, level_name: &str) -> &Level {
        for City{name: _, levels} in &self.0 {
            for level in levels {
                if level.name == level_name {
                    return &level;
                }
            }
        }
        panic!("trying to get level `{level_name}`, name not found");
    }
    pub fn set_level_current_progress(&mut self, level_name: &str, progress: &LevelProgress) {
        for City{name: _, levels} in &mut self.0 {
            for level in levels {
                if level.name == level_name {
                    level.current_progress = progress.clone();
                    return;
                }
            }
        }
        panic!("trying to set current progress on `{level_name}`, name not found");
    }

    pub fn save_progress_to_file(&self) {
        println!("saving progress to file...");
        let mut file = std::fs::File::create(SAVE_FILENAME).expect("create file failed");
        let contents_to_write = serde_json::to_string(self).unwrap();
        file.write_all(contents_to_write.as_bytes()).expect("write failed");
    }

}
