mod errors;
mod utils;
mod logic;
mod structs;
mod rocket;


use std::error::Error;
use std::fmt::{Display, Pointer};
use std::ops::Deref;
use std::path::PathBuf;
use std::{env, thread};
use std::rc::Rc;
use std::cell::RefCell;
use std::time::Duration;
use another_one_bytes_the_dust_tile_resource_mapper_tool::tool::tile_mapper::TileMapper;
use ohcrab_weather::weather_tool::{WeatherPredictionTool};
use rand::{Rng, thread_rng};
use robo_gui::MainState;
use robotics_lib::runner::{Robot, Runnable, Runner};
use robotics_lib::world::world_generator::Generator;
use worldgen_unwrap::public::WorldgeneratorUnwrap;
use structs::structs::*;
use std::sync::Mutex;
use lazy_static::lazy_static;
use rustbeef_nlacompass::compass::NLACompass;
use rocket::rocket;


// note: color output are shown if the IDE is set to do so

// NB: The "nla" tool has sometimes a strange behaviour causing the robot to move back and forth
// The tool usage is inserted in the get_dir() function. To show the expected behaviour it
// it substituted with charting_tools (our group tool) in the same logic. If the user
// wants to try the behaviour with it it can run the following command:
// cargo run switch_tool

// The other one is used because i need to save somewhere the data, i can't save them in the
// robot because ownership is passed to the runner
lazy_static! {
    pub static ref SWITCH_TOOL: Mutex<bool> = Mutex::new(false);
    pub static ref ROCKET_DATA: Mutex<RocketData> = Mutex::new(RocketData::new());
}


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

    // robot creation
    let mut robot = RobertNeville {
        robot: Robot::new(),
        status: RobertStatus::new(),
        knowledge: RobertKnowledge::new(),
        mapper_tool: TileMapper {},
        weather_tool: WeatherPredictionTool::new(),
        nla_compass_tool: NLACompass::new(),
        tick_number:0,
        gui: MainState::new(1).unwrap(), // #GUI
        last_coords: None, //  #GUI
        run: Rc::new(RefCell::new(true)),
    };

    let cont = Rc::clone(&robot.run);
    // Map loading
    let mut w=WorldgeneratorUnwrap::init(false,  Some(PathBuf::from("maps/world_map")));

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
    let mut rocket_data = ROCKET_DATA.lock().unwrap();
    let rocket = rocket().manage(rocket_data.clone());
    let _ = rocket.launch().await;
}
