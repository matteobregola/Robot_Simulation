pub(crate) mod logic {
    use std::thread;
    use std::time::Duration;
    use charting_tools::ChartingTools;
    use crate::errors::errors::*;
    use crate::utils::utiliies::*;
    use crate::structs::structs::*;
    use rustbeef_nlacompass::compass::Destination;
    use robotics_lib::world::World;
    use robotics_lib::interface::*;
    use robotics_lib::world::tile::Content;
    use robotics_lib::runner::Runnable;
    use robotics_lib::utils::LibError;
    use robotics_lib::event::events::Event;
    use robotics_lib::energy::Energy;
    use robotics_lib::world::coordinates::Coordinate;
    use crate::{SWITCH_TOOL, thread_rng};
    use spyglass::spyglass::Spyglass;
    use robotics_lib::runner::backpack::BackPack;
    use rand::Rng;
    use charting_tools::charted_paths::ChartedPaths;
    use charting_tools::charted_coordinate::ChartedCoordinate;
    use robotics_lib::world::environmental_conditions::WeatherType;
    use crate::ROCKET_DATA;
    use std::sync::MutexGuard;

    impl<'a> Runnable for RobertNeville<'a> {
        fn process_tick(&mut self, world: &mut World) {

            if self.tick_number == 0{
                // to show the initial situation
                robot_view(self,world);
                thread::sleep(Duration::from_secs_f32(1.0));
                self.gui.update_world(robot_map(world).unwrap());
                let _ = self.gui.tick();
                let dim =robot_map(world).unwrap().len();
                let mut rocket_data = ROCKET_DATA.lock().unwrap();
                rocket_data.heatmap = vec![vec![0; dim]; dim];
                rocket_data.map_dim = dim;
            }

            if self.tick_number <= 30 {
                thread::sleep(Duration::from_secs_f32(1.0));
            }


            self.gui.update_world(robot_map(world).unwrap());  //#GUI
            self.tick_number += 1;

            let energy=   self.robot.energy.get_energy_level();
            let hunger = self.status.hunger;
            let thirst =  self.status.thirst;
            let robot_x= self.robot.coordinate.get_row();
            let robot_y = self.robot.coordinate.get_col();

            let mut rocket_data:MutexGuard<RocketData> = ROCKET_DATA.lock().unwrap();
            rocket_data.energy_vector.push(energy);
            rocket_data.hunger_vector.push(hunger);
            rocket_data.thirst_vector.push(thirst);
            rocket_data.heatmap[robot_x][robot_y]+=1;
            rocket_data.n_zombies=self.knowledge.zombies.len();
            rocket_data.n_shelters=self.knowledge.shelters.len();

            print!("\x1b[33m");
            println!("--TICK INFO: energy: {}, pos[{},{}], hunger: {}, thirst: {}, zombies: {:?}",
                     energy,
                     robot_x,
                     robot_y,
                     hunger,
                     thirst,
                     self.knowledge.zombies);
            print!("\x1b[0m");

            let mut next_action_decided = false;

            let mut future_action = Action::REST;

            let next_action = self.status.next_action.clone();
            match &next_action {
                Action::REST => {
                    rocket_data.action_values[4]+=1;
                    print!("\x1b[32m");
                    println!("Action: Resting...");
                    print!("\x1b[0m");
                    self.status.previous_action = Action::REST;
                }
                Action::WalkDiscover(dir, steps) => {
                    //WalkDiscover --> Robert has high energy will discover new locations
                    rocket_data.action_values[1]+=1;
                    print!("\x1b[32m");
                    println!("Action: Walk to Discover...");
                    print!("\x1b[0m");
                    let r = thread_rng().gen_range(0..2);
                    let direction;
                    match r {
                        1 => {
                            direction = dir.0.clone();
                        }
                        _ => {
                            direction = dir.1.clone();
                        }
                    }

                    let walk_result = go(self, world, direction);

                    match walk_result {
                        Ok(_) => {
                            // print!("\x1b[32m");
                            // println!("{}", "moved");
                            // print!("\x1b[0m");
                            self.status.previous_action = Action::WalkDiscover(dir.clone(), 0);
                            update_knowledge(self, world, true); //the go function update the knowledge
                            // about the world but i also need to update the knowledge of robert

                            let zombie_found = check_for_zombies(&self, world);
                            if zombie_found.0 {
                                print!("\x1b[31m");
                                println!("Robert has a zombie near him! Escape!");
                                print!("\x1b[0m");
                                // zombie sound
                                match &mut self.audio_tool {
                                    None => {}
                                    Some(audio) => {
                                        let _ =audio.play_audio(&self.sound[0]);
                                    }
                                }
                                self.status.previous_action = Action::ESCAPE(None, zombie_found.2, zombie_found.1.unwrap());
                            }
                        }
                        Err(x) => {
                            self.status.previous_action = Action::WalkDiscover(dir.clone(), steps+1); // still in the same position
                            match x {
                                LibError::NotEnoughEnergy => {
                                    future_action = Action::REST;
                                    next_action_decided = true;
                                }
                                LibError::OutOfBounds | LibError::CannotWalk => {}
                                _ => {}
                            }
                        }
                    }

                    if self.get_energy().get_energy_level() > 400 {
                        // sends also the dog
                        let robert_row = self.robot.coordinate.get_row();
                        let robert_col = self.robot.coordinate.get_col();
                        // TOOL 4:
                        let mut spyglass = Spyglass::new(
                            robert_row, // center_row
                            robert_col, // center_col
                            10, // distance
                            robot_map(world).unwrap().len(), // world_dim
                            Some(80), // energy_budget
                            false, // enable_cross
                            0.5, // view_threshold
                            |_| false,
                            // stops_when
                        );
                        let _ = spyglass.new_discover(self, world);
                        print!("\x1b[32m");
                        println!("Sam is searching with robert");
                        print!("\x1b[0m");
                        update_knowledge(self, world, false);
                    }
                }

                Action::DISCOVER(disc_info) => {

                    rocket_data.action_values[3]+=1;
                    print!("\x1b[32m");
                    println!("Action: Sam Discovering...");
                    print!("\x1b[0m");
                    let _ = disc_info.content.clone();

                    // Robert sends the dog to discover the world (he doesn't move)
                    // Spyglass discovers based on the Discover value

                    let robert_row = self.robot.coordinate.get_row();
                    let robert_col = self.robot.coordinate.get_col();

                    // TOOL 4:
                    let mut spyglass = Spyglass::new(
                        robert_row, // center_row
                        robert_col, // center_col
                        10, // distance
                        robot_map(world).unwrap().len(), // world_dim
                        Some(disc_info.energy), // energy_budget
                        false, // enable_cross
                        0.5, // view_threshold
                        |_| false,
                        // stops_when
                    );

                    match &disc_info.content {
                        None => {}
                        Some(x) => {
                            match x {
                                Content::Water(_) => {
                                    spyglass.set_stop_when(|tile|
                                                               if let Content::Water(_) = tile.content
                                                               { true } else { false },
                                    );
                                }
                                Content::Market(_) => {
                                    spyglass.set_stop_when(|tile|
                                                               if let Content::Market(_) = tile.content
                                                               { true } else { false },
                                    );
                                }
                                Content::Building => {
                                    spyglass.set_stop_when(|tile|
                                                               if let Content::Building = tile.content
                                                               { true } else { false },
                                    );
                                }
                                _ => {}
                            };
                        }
                    }
                    self.status.previous_action = Action::DISCOVER(disc_info.clone());
                    let _ = spyglass.new_discover(self, world);
                    update_knowledge(self, world, false);
                }
                Action::COLLECT(dir, content) => {
                    rocket_data.action_values[5]+=1;
                    print!("\x1b[34m");
                    println!("Action: Collecting...");
                    print!("\x1b[0m    ");
                    let content = content.clone();
                    self.status.previous_action = Action::COLLECT(dir.clone(), content.clone());

                    match &content {
                        Content::Water(_) => {
                            let result = destroy(self, world, dir.clone());
                            match result {
                                Ok(x) => {
                                    print!("\x1b[34m");
                                    println!("     Robert has some water");
                                    print!("\x1b[0");
                                    self.status.thirst += 100 + 10 * x;
                                }
                                Err(l) => {
                                    match l {
                                        LibError::NotEnoughEnergy => {
                                            future_action = Action::REST;
                                            next_action_decided = true;
                                        }
                                        _ => {
                                            println!("Collect error:{:?}", l);
                                            terminate_with_error(Error::CollectError);
                                        }
                                    }
                                }
                            }
                        }
                        Content::Market(_) => {
                            print!("\x1b[32m");
                            println!("     Robert has collected some food from the market");
                            print!("\x1b[0m");
                            self.status.hunger += 150;
                        }
                        _ => {}
                    }
                }
                Action::ESCAPE(dir, zombie, avoid) => {
                    rocket_data.action_values[2]+=1;
                    print!("\x1b[32m");
                    println!("Action: Escaping..");
                    print!("\x1b[0m");
                    self.status.previous_action = Action::ESCAPE(dir.clone(), zombie.clone(), avoid.clone());
                    let walk_result = go(self, world, dir.clone().unwrap());
                    match walk_result {
                        Ok(_) => {
                            update_knowledge(self, world, true); //the go function update the knowledge
                            // about the world but i also need to update the knowledge of robert

                            let zombie_found = check_for_zombies(&self, world);
                            if zombie_found.0 {
                                print!("\x1b[31m");
                                println!("Robert has a zombie near him! Escape!");
                                print!("\x1b[0m");
                                // zombie sound
                                match &mut self.audio_tool {
                                    None => {}
                                    Some(audio) => {
                                       let _ = audio.play_audio(&self.sound[0]);
                                    }
                                }
                                self.status.previous_action = Action::ESCAPE(None, zombie_found.2, zombie_found.1.unwrap());
                            }
                        }
                        Err(x) => {
                            match x {
                                LibError::NotEnoughEnergy => {
                                    future_action = Action::REST;
                                    next_action_decided = true;
                                }
                                LibError::OutOfBounds | LibError::CannotWalk => {}
                                _ => {}
                            }
                        }
                    }
                }
                Action::WalkToTarget(direction, target, content) => {
                    rocket_data.action_values[0]+=1;
                    print!("\x1b[32m");
                    println!("Action: Walking to Target...");
                    self.status.previous_action = Action::WalkToTarget(direction.clone(), target.clone(), content.clone());
                    let walk_result = go(self, world, direction.clone());


                    match walk_result {
                        Ok(_) => {
                            update_knowledge(self, world, true); //the go function update the knowledge
                            // about the world but i also need to update the knowledge of robert

                            let zombie_found = check_for_zombies(&self, world);
                            if zombie_found.0 {
                                print!("\x1b[31m");
                                println!("Robert has a zombie near him! Escape!");
                                print!("\x1b[0m");
                                // zombie sound
                                match &mut self.audio_tool {
                                    None => {}
                                    Some(audio) => {
                                        let _ = audio.play_audio(&self.sound[0]);
                                    }
                                }
                                self.status.previous_action = Action::ESCAPE(None, zombie_found.2, zombie_found.1.unwrap());
                                // future_action= Action::ESCAPE(None,zombie_found.2,zombie_found.1.unwrap());
                                // next_action_decided = true ;
                            }
                        }
                        Err(x) => {
                            match x {
                                LibError::NotEnoughEnergy => {
                                    future_action = Action::REST;
                                    next_action_decided = true;
                                }
                                LibError::OutOfBounds | LibError::CannotWalk => {}
                                _ => {}
                            }
                        }
                    }
                }
            }

            // ---------------------------------------

            // update Robert status
            if !next_action_decided {
                self.status.next_action = eval_next_ac(&world, self);
            } else {
                self.status.next_action = future_action;
            }


            let mut thirst = self.status.thirst as i32;
            let mut hunger = self.status.hunger as i32;
            thirst -= 2;
            hunger -= 3;

            if thirst <= 0 {
                print!("\x1b[0m");
                println!(" -----------Robert is dead of thirst!---------\n");
                *self.run.borrow_mut() = false;
            } else {
                if hunger <= 0 {
                    print!("\x1b[0m");
                    println!(" -----------Robert is dead of hunger!---------\n");
                    *self.run.borrow_mut() = false;
                } else {
                    self.status.thirst = thirst as usize;
                    self.status.hunger = hunger as usize;
                }
            }
        }
        fn handle_event(&mut self, event: Event) {
            // #GUI

            match event {
                Event::Ready => {
                    self.gui.add_robot(
                        self.get_coordinate().get_col(),
                        self.get_coordinate().get_row(),
                    );

                    self.last_coords = Some((
                        self.get_coordinate().get_row(),
                        self.get_coordinate().get_col(),
                    ));
                    self.gui.update_robot(self.last_coords, self.last_coords)
                }
                Event::Terminated => {
                    println!("Terminated!!!");
                    *self.run.borrow_mut() = false;
                }
                Event::TimeChanged(x) => {
                    self.weather_tool.process_event(&Event::TimeChanged(x.clone()));
                    self.gui.update_time_of_day(x.get_time_of_day());
                    self.gui.update_weather(x.get_weather_condition());
                }
                Event::DayChanged(e) => {
                    print!("\x1b[33m");
                    println!("\n--New Day: {:?}\n\n", e);
                    print!("\x1b[0m")
                }
                Event::Moved(_, coords) => {
                    self.gui.update_robot(Some(coords), self.last_coords);
                    match self.gui.tick() {
                        Ok(_) => {}
                        Err(_) => self.handle_event(Event::Terminated),
                    }

                    self.last_coords = Some((
                        self.get_coordinate().get_row(),
                        self.get_coordinate().get_col(),
                    ));
                }
                _ => {}
            };
        }
        fn get_energy(&self) -> &Energy {
            &self.robot.energy
        }
        fn get_energy_mut(&mut self) -> &mut Energy {
            &mut self.robot.energy
        }
        fn get_coordinate(&self) -> &Coordinate {
            &self.robot.coordinate
        }
        fn get_coordinate_mut(&mut self) -> &mut Coordinate {
            &mut self.robot.coordinate
        }
        fn get_backpack(&self) -> &BackPack {
            &self.robot.backpack
        }
        fn get_backpack_mut(&mut self) -> &mut BackPack {
            &mut self.robot.backpack
        }
    }

    pub fn eval_next_ac(world: &World, robert: &mut RobertNeville) -> Action {

        // handle escape first --> priority over everything
        let previous_action = robert.status.previous_action.clone();
        match &previous_action {
            Action::ESCAPE(dir, origin, avoid) => {
                let robert_x = robert.robot.coordinate.get_row();
                let robert_y = robert.robot.coordinate.get_col();
                let map = robot_map(world).unwrap();
                match &map[robert_x][robert_y] {
                    Some(t) => {
                        if t.content == Content::Building {
                            // he escaped, needs to rest
                            print!("\x1b[34m");
                            println!("Robert reached a shelter, is safe!");
                            print!("\x1b[0m");
                            return Action::REST;
                        } else {
                            //  find a tile that contains a Building

                            let target = robert.mapper_tool.find_closest(
                                &world, robert, Content::Building); //TOOL1;
                            match target {
                                Ok(t) => {
                                    // BUILDING FOUND
                                    let direction;
                                    match dir {
                                        None => {
                                            // eval the path
                                            direction=get_dir(world, robert, &(t.get_height(), t.get_width()), Some(&avoid.clone()), true, true).unwrap()
                                        }
                                        Some(_) => {
                                            // already specified the path
                                            direction=get_dir(world, robert, &(t.get_height(), t.get_width()), Some(&avoid.clone()), true, false).unwrap();
                                        }
                                    }
                                    return Action::ESCAPE(Some(direction), origin.clone(), avoid.clone());
                                }
                                Err(_) => {
                                    // Building not found --> walk away in some direction
                                    let possibilities = [Direction::Up, Direction::Left, Direction::Right, Direction::Down];
                                    for i in possibilities {
                                        if i != avoid.0 {
                                            return Action::ESCAPE(Some(i), origin.clone(), avoid.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    None => {
                        //not possible, the robot is on the tile
                    }
                }
            }
            _ => {
                // it will be intercepted by code below
            }
        }

        let mut thirst = false;
        let mut hunger = false;
        if robert.status.thirst <= 70 {
            thirst = true;
        }
        if robert.status.hunger <= 70 {
            hunger = true;
        }

        if thirst || hunger {
            // print!("\x1b[31m");
            // println!("I need to drink or eat");
            // print!("\x1b[0m");

            match &previous_action {
                Action::WalkToTarget(_, target, content) => {
                    let direction = get_dir(world, robert, &target, None, false, false).unwrap();
                    if check_target(&robert, &target, &direction) {
                        // Target in front of him, collect it
                        print!("\x1b[34m");
                        println!("target found");
                        print!("\x1b[0m");
                        return Action::COLLECT(direction.clone(), content.clone().unwrap_or(Content::Water(1)));
                    } else {
                        return Action::WalkToTarget(direction.clone(), target.clone(), Some(content.clone().unwrap_or(Content::Water(1))));
                    }
                }

                _ => {
                    /*
                        Robert needs to drink or eat and is the "first time" , two possibilities:
                        1) If he knows where to find it, go to it
                        2) if it doesn't, first send the dog to find it
                    */

                    // decide whether he needs more to eat or drink
                    let mut content = Content::Water(0);
                    let mut seraching_market = false;
                    if (robert.status.thirst as f32 * 1.5) > robert.status.hunger as f32 {
                        print!("\x1b[31m");
                        println!("Robert Needs to eat");
                        print!("\x1b[0m");
                        content = Content::Market(0);
                        seraching_market = true;
                    } else {
                        print!("\x1b[31m");
                        println!("Robert Needs to drink");
                        print!("\x1b[0m")
                    }
                    // TOOL 1: The resource mapper
                    let result = robert.mapper_tool.find_closest(&world, robert, content.clone());

                    match result {
                        Ok(objective) => {
                            //1) search path for going there
                            if seraching_market {
                                print!("\x1b[35m");
                                println!("I have found market at [{},{}]", objective.get_height(), objective.get_width());
                                print!("\x1b[0m");
                            } else {
                                print!("\x1b[35m");
                                println!("I have found water at [{},{}]", objective.get_height(), objective.get_width());
                                print!("\x1b[0m");
                            }

                            let target = (objective.get_height(), objective.get_width());
                            let direction = get_dir(world, robert, &target, None,false, true).unwrap();

                            if check_target(&robert, &target, &direction) {
                                // has the target in the specified Direction
                                print!("\x1b[34m");
                                println!("target found");
                                print!("\x1b[0m");
                                return Action::COLLECT(direction.clone(), content.clone());
                            } else {
                                // not reached yet
                                return Action::WalkToTarget(direction.clone(), target.clone(), Some(content.clone()));
                            }

                        }
                        Err(_) => {
                            //2) Set Discovery before moving
                            return Action::DISCOVER(DiscoverInfo {
                                content: Some(Content::Water(0)),
                                energy: robert.get_energy().get_energy_level(),
                            });
                        }
                    }
                }
            }
        }

        let energy= robert.robot.energy.get_energy_level();
        // no need to drink, no need to eat
        if energy > 100 {
            let mut high_energy = false;
            // the decision about what to do if robot has no need to drink or eat are based on the energy
            // and weather.

            if energy > 450 {
                high_energy = true;
            }

            if !high_energy {
                let prediction = robert.weather_tool.predict(1); // TOOL 4
                match prediction {
                    Ok(p) => {
                        match p {
                            WeatherType::Sunny => {
                                // it will do something in the next match
                                print!("\x1b[0m");
                                println!("There will be a good weather in the next future, i prefer walk");
                                print!("\x1b[0m");
                            }
                            _ => {
                                // bad weather in the next days, it's better to rest
                                print!("\x1b[0m");
                                println!("There will be a bad weather in the future, i prefer to send the dog");
                                print!("\x1b[0m");
                                return Action::DISCOVER(DiscoverInfo { content: None, energy:energy/4});
                            }
                        }
                    }
                    Err(_) => {
                        println!("Weather tool error");
                    }
                }
            }


            match robert.status.previous_action {
                Action::REST | Action::COLLECT(_, _) | Action::DISCOVER(_) => {

                    // Situation: robert has high energy, and it's the "first"
                    // high energy operation. He will try to reach a new zone

                    let map = map_converter_normal(robot_map(world).unwrap(), &robert.knowledge.zombies);

                    let directions;
                    let target = find_most_unvisited_zone(&map);
                    // Search for an unvisited zone
                    // get the directions to go to it
                    directions = eval_direction((robert.robot.coordinate.get_row(),
                                                     robert.robot.coordinate.get_col()), target);

                    let mut direction1=directions.0.0;
                    let mut direction2=directions.0.1;


                    while check_cannot_go(robert, world,&direction1).0 || check_cannot_go(robert, world,&direction2).0{
                        direction1=generate_random_direction();
                        direction2=generate_random_direction();
                    }


                    return Action::WalkDiscover((direction1, direction2),0);
                }
                Action::WalkDiscover(_,steps) =>{

                    if steps < 3{
                        let map = map_converter_normal(robot_map(world).unwrap(), &robert.knowledge.zombies);

                        let directions;
                        let target = find_most_unvisited_zone(&map);

                        directions = eval_direction((robert.robot.coordinate.get_row(),
                                                     robert.robot.coordinate.get_col()), target);
                        return Action::WalkDiscover((directions.0.0.clone(), directions.0.1.clone()), steps);
                    }
                    else{
                        // is stacked somewhere, eval a random direction
                        let mut direction1=generate_random_direction();
                        let mut direction2=generate_random_direction();
                        while check_cannot_go(robert, world,&direction1).0 || check_cannot_go(robert, world,&direction2).0{
                            direction1=generate_random_direction();
                            direction2=generate_random_direction();
                        }
                        return Action::WalkDiscover((direction1, direction2), steps);
                    }

                }
                _ => {
                    terminate_with_error(Error::LogicError);
                }
            }
        }

        // LOW energy: rest
        Action::REST
    }


    pub(crate) fn get_dir(world: &World, robert: &mut RobertNeville, target: &(usize, usize), avoid: Option<&(Direction, Option<Direction>)>, escape:bool, initialize:bool) -> Option<Direction> {
        // i need to escape but also to don't walk over other zombies

        let escape_map;

        if escape{
            escape_map=map_converter_escape(robot_map(world).unwrap(),
                                              &robert.knowledge.zombies);
        }
        else{
            escape_map=map_converter_normal(robot_map(world).unwrap(),
                                           &robert.knowledge.zombies);
        }

        // tool choice
        let switch_tool = SWITCH_TOOL.lock().unwrap();
        if !*switch_tool {
            if initialize || robert.knowledge.path_index == 0 {
                //println!("------EVALUATING PATH");
                let mut charted_path = ChartingTools::tool::<ChartedPaths>().unwrap();
                charted_path.init(&escape_map, world);
                let destination = ChartedCoordinate::new(target.0, target.1);
                let my_coordinate = ChartedCoordinate(robert.robot.coordinate.get_row(), robert.robot.coordinate.get_col());
                let path= charted_path.shortest_path(my_coordinate, destination);
                if path.is_none(){
                    robert.knowledge.path = None;
                    robert.knowledge.path_index = 1;
                }
                else{
                    robert.knowledge.path = Some(path.unwrap().1);
                    print!("\x1b[35m");
                    println!("path: {:?}", robert.knowledge.path);
                    print!("\x1b[om");
                    robert.knowledge.path_index = 1;
                }
            }
            let my_coordinate = ChartedCoordinate(robert.robot.coordinate.get_row(), robert.robot.coordinate.get_col());

            match &robert.knowledge.path {
                Some(path) => {
                    if path.len() == 1 {
                        // robert is on the target, move in a random but safe way

                        let mut direction = generate_random_direction();
                        let mut sourrounded = 0;
                        while check_cannot_go(robert, &world, &direction).0 && sourrounded < 5 {
                            direction = generate_random_direction();
                            sourrounded += 1;
                        }
                        robert.knowledge.path_index = 0; // flag to re_evaluate the path

                        return Some(direction);
                    } else {
                        let next = path.get(robert.knowledge.path_index).unwrap();
                        let direction = ChartedPaths::coordinates_to_direction(my_coordinate, next.clone());
                        match direction {
                            Ok(dir) => {
                                robert.knowledge.path_index+=1;
                                if robert.knowledge.path_index >= path.len() {
                                    robert.knowledge.path_index = path.len()-1;
                                }
                                return Some(dir);
                            }
                            Err(_) => {
                                return Some(Direction::Up);
                            }
                        }

                    }
                }
                None => {
                    // Path to building not found, robert will escape in a random direction but not the zombie one
                    robert.knowledge.path_index = 0 ; // to re-evaluate the path;
                    let possibilities = [Direction::Up, Direction::Left, Direction::Right, Direction::Down];
                    match avoid {
                        Some(a) => {
                            for i in possibilities {
                                if i != a.0 {
                                    return Some(i);
                                }
                            }

                            return Some(Direction::Up);
                        }
                        None => {
                            return Some(Direction::Up);
                        }
                    }
                }
            }
        } else {
            robert.nla_compass_tool.set_destination(Destination::Coordinate((target.0, target.1)));
            let nla_result = robert.nla_compass_tool.get_move(
                &escape_map,
                (robert.robot.coordinate.get_row(), robert.robot.coordinate.get_col()));
            match nla_result {
                Ok(direction) => {
                    // path to target found
                    return Some(direction);
                }
                Err(_) => {
                    // path to target not found, just walk away
                    // Path to building not found, robert will escape in a random direction but not the robot one
                    let possibilities = [Direction::Up, Direction::Left, Direction::Right, Direction::Down];
                    match avoid {
                        Some(a) => {
                            for i in possibilities {
                                if i != a.0 {
                                    return Some(i);
                                }
                            }
                            return Some(Direction::Up);
                        }
                        None => {
                            return Some(Direction::Up);
                        }
                    }
                }
            }
        }
    }
}