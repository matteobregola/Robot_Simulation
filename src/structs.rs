
pub(crate) mod structs {
    use std::cell::RefCell;
    use std::collections::HashSet;
    use std::fmt::{Display, Formatter};
    use std::rc::Rc;
    use another_one_bytes_the_dust_tile_resource_mapper_tool::tool::tile_mapper::TileMapper;
    use charting_tools::charted_coordinate::ChartedCoordinate;
    use ohcrab_weather::weather_tool::WeatherPredictionTool;
    use robo_gui::MainState;
    use robotics_lib::interface::*;
    use robotics_lib::runner::Robot;
    use robotics_lib::world::tile::Content;
    use rustbeef_nlacompass::compass::NLACompass;
    use oxagaudiotool::{OxAgAudioTool};
    use oxagaudiotool::sound_config::OxAgSoundConfig;


    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct DiscoverInfo {
        pub content: Option<Content>,
        pub energy: usize,
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub(crate) enum Action {
        REST,
        WalkDiscover((Direction, Direction), usize),
        WalkToTarget(Direction, (usize, usize), Option<Content>),
        DISCOVER(DiscoverInfo),
        COLLECT(Direction, Content),
        ESCAPE(Option<Direction>,(usize, usize), (Direction,Option<Direction>)),
    }
    /*
        Rest --> Robert is tired, needs to sleep
        WalkDiscover --> Robert has high energy will discover new locations. the usize contains how many times
                         the robot is in the same position
        Walk to target --> Robert needs to reach water/food. Direction to reach the target, target
                            coordinates and type
        Discover --> Robert needs to find a specific element in the world, sends the dog to find it
        Collect --> Robert has reach a target and now he can collect it (drink/eat)
        Escape --> Robert has encountered a robot, needs to reach a safe place. If the Direction is None
                   means that i have to evaluate where to escape, the second field is the origin of the escape
                   and the third one is where was the zombie in respect to him
     */
    pub(crate) struct RobertStatus {
        pub hunger: usize,
        pub thirst: usize,
        pub previous_action: Action,
        pub next_action: Action,
    }

    impl RobertStatus {
        pub fn new() -> RobertStatus {
            RobertStatus {
                hunger: 100,
                thirst: 100,
                previous_action: Action::REST,
                next_action: Action::DISCOVER(DiscoverInfo { content: None, energy: 500 }),
                // first action is to discover generally the world
            }
        }
    }

    impl Display for RobertStatus {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let s = format!("Robert Status: [hunger = {}, thirst={}]", self.hunger, self.thirst);
            write!(f, "{}", s)
        }
    }

    pub(crate) struct RobertKnowledge {
        pub zombies: HashSet<(usize, usize)>,
        pub shelters: HashSet<(usize, usize)>,
        // Hashset to have unique set of coordinates without checking
        pub path: Option<Vec<ChartedCoordinate>>,
        pub path_index: usize,
    }

    impl Display for RobertKnowledge {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let mut s = String::from("Robert Knowledge:\n[");
            if self.zombies.len() == 0 {
                s += "No zombie found, "
            } else {
                s = format!("{} zombies {} found: ", s, self.zombies.len());
                for p in &self.zombies {
                    s = format!("{}({},{}) ", s, p.0, p.1);
                }
                s += ",  ";
            }
            if self.shelters.len() == 0 {
                s += "No shelter found]"
            } else {
                s = format!("{} buildings {} found: ", s, self.shelters.len());
                for p in &self.shelters {
                    s = format!("{}({},{}) ", s, p.0, p.1);
                }
                s += "]";
            }


            write!(f, "{}", s)
        }
    }

    impl RobertKnowledge {
        pub fn new() -> RobertKnowledge {
            RobertKnowledge {
                zombies: HashSet::new(),
                shelters: HashSet::new(),
                path: Some(Vec::new()),
                path_index: 1,
            }
        }
    }

    pub(crate) struct RobertNeville<'a> {
        pub robot: Robot,
        pub status: RobertStatus,
        pub knowledge: RobertKnowledge,
        pub mapper_tool: TileMapper,
        pub weather_tool: WeatherPredictionTool,
        pub nla_compass_tool: NLACompass,
        pub audio_tool: Option<OxAgAudioTool>,
        pub sound: Vec<OxAgSoundConfig>,
        pub tick_number: u64, // #GUI
        pub gui: MainState<'a>, // #GUI
        pub last_coords: Option<(usize, usize)>, // #GUI
        pub run: Rc<RefCell<bool>>, // ROCKET
    }

    #[derive(Debug, Clone)]
    pub struct RocketData{
        pub action_values: Vec<usize>, //Wtd, Wd, E, D, R.
        pub energy_vector: Vec<usize>,
        pub thirst_vector: Vec<usize>,
        pub hunger_vector: Vec<usize>,
        pub heatmap: Vec<Vec<usize>>,
        pub map_dim: usize,
        pub n_zombies: usize,
        pub n_shelters: usize,
    }
    
    impl RocketData{
        pub(crate) fn new() ->RocketData{
            RocketData{
                action_values: vec![0;6],
                energy_vector: Vec::new(),
                thirst_vector: Vec::new(),
                hunger_vector: Vec::new(),
                heatmap: Vec::new(),
                map_dim: 0,
                n_zombies: 0,
                n_shelters: 0,
            }
        }
    }
}
