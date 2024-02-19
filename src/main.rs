mod errors;
mod utils;
mod logic;
mod structs;
mod rocket;


use std::path::PathBuf;
use std::{env};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use another_one_bytes_the_dust_tile_resource_mapper_tool::tool::tile_mapper::TileMapper;
use ohcrab_weather::weather_tool::{WeatherPredictionTool};
use rand::{thread_rng};
use robo_gui::MainState;
use robotics_lib::runner::{Robot, Runner};
use worldgen_unwrap::public::WorldgeneratorUnwrap;
use structs::structs::*;
use std::sync::Mutex;
use lazy_static::lazy_static;
use rustbeef_nlacompass::compass::NLACompass;
use rocket::rocket;
use oxagaudiotool::OxAgAudioTool;
use oxagaudiotool::sound_config::OxAgSoundConfig;

// note:  Color output will be displayed if the IDE is configured accordingly.

// NB: The "NLA" tool occasionally exhibits erratic behavior, leading to the robot moving back and forth.
// This tool's usage is integrated into the get_dir() function. To demonstrate the expected behavior,
// it is replaced with charting_tools (our group tool) using the same logic. To experiment with
// this tool, users can execute the following command:
// cargo run switch_tool
// the code then uses a global variable to decide which one to use


// The additional global variable is necessary for storing the rocket's data  as I cannot retain it within the robot
// due to ownership being transferred to the runner (I wouldn't be able to access it after the run loop).
lazy_static! {
    pub static ref SWITCH_TOOL: Mutex<bool> = Mutex::new(false);
    pub static ref ROCKET_DATA: Mutex<RocketData> = Mutex::new(RocketData::new());
}

// tokio is used as async runtime for rocket
#[tokio::main]
async fn main(){

    // Args management for using different logic
    let args: Vec<String> = env::args().collect();
    if let Some(arg) = args.get(1){
        if arg == "switch_tool"{
            *SWITCH_TOOL.lock().unwrap()= true;
            print!("\x1b[31m");
            println!("INFO: Switched tool to NLA");
            print!("\x1b[0m");
        }
    }

    let audio_tool =
        OxAgAudioTool::new(HashMap::new(), HashMap::new(),
                           HashMap::new());

    let audio_result;

    match audio_tool {
        Ok(a) => {
            audio_result=Some(a);
        }
        Err(_) => {
            audio_result=None;
        }
    }

    let sounds= vec![OxAgSoundConfig::new("assets/zombie-growl.mp3")];

    // robot creation
    let robot = RobertNeville {
        robot: Robot::new(),
        status: RobertStatus::new(),
        knowledge: RobertKnowledge::new(),
        mapper_tool: TileMapper {},
        weather_tool: WeatherPredictionTool::new(),
        nla_compass_tool: NLACompass::new(),
        audio_tool: audio_result,
        sound: sounds,
        tick_number:0,
        gui: MainState::new(1).unwrap(), // #GUI
        last_coords: None, //  #GUI
        run: Rc::new(RefCell::new(true)),

    };

    let cont = Rc::clone(&robot.run);

    // TOOL 1
    // There are two already generated maps: "world_map_small" and "world_map"
    let mut w=WorldgeneratorUnwrap::init(false,  Some(PathBuf::from("maps/world_map_small")));

    let mut run = Runner::new(Box::new(robot), &mut w);
    match run {
        Ok(ref mut r) => 'running: loop {
            if !*cont.borrow() {
                print!("\x1b[33m");
                println!("\n\n Game stopped with Robert Still alive");
                break 'running;
            }
            let _ = r.game_tick();

        },
        Err(e) => println!("Error:{:?}", e),
    }
    let rocket_data = ROCKET_DATA.lock().unwrap();
    let rocket = rocket().manage(rocket_data.clone());
    let _ = rocket.launch().await;
}
