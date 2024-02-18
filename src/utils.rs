pub(crate) mod utiliies {
    use std::collections::HashSet;
    use rand::thread_rng;
    use robotics_lib::interface::{Direction, robot_map};
    use robotics_lib::world::tile::{Content, Tile};
    use robotics_lib::world::World;
    use crate::RobertNeville;
    use rand::Rng;

    pub(crate) fn check_target(robert: &RobertNeville, target: &(usize, usize), direction: &Direction) -> bool {
        // check if in the direction is considering there is the target

        let robert_x = robert.robot.coordinate.get_row() as i32; // to avoid overflow
        let robert_y = robert.robot.coordinate.get_col() as i32;

        let t_x = target.0 as i32;
        let t_y = target.1 as i32;
        match direction {
            Direction::Up => {
                robert_x - 1 == t_x && robert_y == t_y
            }
            Direction::Down => {
                robert_x + 1 == t_x && robert_y == t_y
            }
            Direction::Left => {
                 robert_x == t_x && robert_y - 1 == t_y
            }
            Direction::Right => {
               robert_x == t_x && robert_y + 1 == t_y
            }
        }
    }

    pub(crate) fn check_for_zombies(robert: &RobertNeville, world: &World) -> (bool,Option<(Direction,Option<Direction>)>, (usize,usize)) {
        // return true if around him there is a zombie, if so returns the direction that robert needs to avoid and where the robot is
        // (however if it's in a building already it returns None)

        let rob_x=robert.robot.coordinate.get_row();
        let rob_y=robert.robot.coordinate.get_col();
        let map=robot_map(world).unwrap();
        match &map[rob_x][rob_y]{
            None => {}
            Some(t) => {
                if t.content == Content::Building{
                    return (false,None,(0,0));
                }
            }
        }


        let offsets:[(i32,i32);8] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];
        for x in &offsets{
            let val_x=rob_x as i32 + x.0;
            let val_y=rob_y as i32 + x.1;
            if val_x>=0 && val_y>=0 && val_x<map.len() as i32{
                if val_y < map[val_x as usize].len() as i32{
                    match &map[val_x as usize][val_y as usize]{
                        Some(t) => {
                            if t.content == Content::Scarecrow {
                                let position = calculate_direction((rob_x, rob_y),
                                                                   (val_x as usize, val_y as usize));
                                // println!(" ROBERT HAS A zombie NEAR TO HIM. {},{}", val_x, val_y);
                                return (true, Some(position), (val_x as usize, val_y as usize));
                            }
                        }
                        None => {}

                    }
                }
            }
        }

        return (false,None,(0,0));
    }

    pub(crate) fn calculate_direction(coord1: (usize, usize), coord2: (usize, usize)) -> (Direction, Option<Direction>) {
        // used to get the direction to encounter the zombie
        let dx = coord2.0 as i32 - coord1.0 as i32;
        let dy = coord2.1 as i32 - coord1.1 as i32;

        match (dx, dy) {
            (1, 0) => (Direction::Down, None),
            (-1, 0) => (Direction::Up, None),
            (0, 1) => (Direction::Right, None),
            (0, -1) => (Direction::Left, None),
            (1, 1) => (Direction::Down, Some(Direction::Right)),
            (-1, 1) => (Direction::Up, Some(Direction::Right)),
            (1, -1) => (Direction::Down, Some(Direction::Left)),
            (-1, -1) => (Direction::Up, Some(Direction::Left)),
            _ => (Direction::Down, Some(Direction::Right)),
        }
    }

    pub(crate) fn update_knowledge(robert: &mut RobertNeville, world: &World, around: bool) {
        // Update Robert knowledge update Zombie Zones and shelters.
        // it is optimized based on the fact that the if the new tiles have been discovered by moving
        // then i need only to check around him
        if around{
            let rob_x=robert.robot.coordinate.get_row();
            let rob_y=robert.robot.coordinate.get_col();
            let map=robot_map(world).unwrap();
            let offsets:[(i32,i32);8] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];
            for x in &offsets{
                let val_x=rob_x as i32 + x.0;
                let val_y=rob_y as i32 + x.1;
                if val_x>=0 && val_y>=0 && val_x<map.len() as i32{
                    if val_y < map[val_x as usize].len() as i32{
                        match &map[val_x as usize][val_y as usize]{
                            Some(t) => {
                                match t.content {
                                    Content::Building => { robert.knowledge.shelters.insert((val_x as usize, val_y as usize)); }
                                    Content::Scarecrow => { robert.knowledge.zombies.insert((val_x as usize, val_y as usize)); }
                                    _ => {}
                                }
                            }
                            None => {}

                        }
                    }
                }
            }
        }
        else {
            let map = robot_map(world).unwrap();
            for (index_i, i) in map.iter().enumerate() {
                for (index_j, j) in i.iter().enumerate() {
                    match j {
                        None => {}
                        Some(t) => {
                            match t.content {
                                Content::Building => { robert.knowledge.shelters.insert((index_i, index_j)); }
                                Content::Scarecrow => { robert.knowledge.zombies.insert((index_i, index_j)); }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }


    pub(crate) fn find_most_unvisited_zone(map: &Vec<Vec<Option<Tile>>>) -> (usize, usize) {

        // convert Some to 1 and None to 0
        let map_converted: Vec<Vec<usize>> = map
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|option| match option {
                        Some(_) => 0,
                        None => 1,
                    })
                    .collect()
            })
            .collect();

        let dim = map_converted.len();

        let mut sum_matrix = vec![vec![0; dim]; dim];
        let mut max_value = 0;
        let mut max_coords = (0, 0);

        for i in 0..dim {
            for j in 0..dim {
                let row_sum = map_converted[i].iter().sum::<usize>();
                let col_sum = map_converted.iter().map(|row| row[j]).sum::<usize>();
                sum_matrix[i][j] = row_sum + col_sum;

                if sum_matrix[i][j] > max_value {
                    max_value = sum_matrix[i][j];
                    max_coords = (i, j);
                }
            }
        }

        max_coords
    }

    pub(crate) fn eval_direction(now: (usize, usize), to: (usize, usize)) -> ((Direction, Direction), usize) {
        let distance = ((now.0 as i32 - now.1 as i32).abs() + (now.1 as i32 - now.1 as i32).abs()) as usize;
        if now.0 < to.0 {
            if now.1 < to.1 {
                return ((Direction::Right, Direction::Down), distance);
            } else {
                return ((Direction::Left, Direction::Down), distance);
            }
        } else {
            if now.1 < to.1 {
                return ((Direction::Right, Direction::Up), distance)
            } else {
                return ((Direction::Left, Direction::Up), distance)
            }
        }
    }

    pub(crate) fn map_converter_escape(mut old_map: Vec<Vec<Option<Tile>>>, positions:&HashSet<(usize,usize)>)-> Vec<Vec<Option<Tile>>>{
        // takes the world map and removes the tile with the zombie to get a better path
        // to be used with the path_finder in ESCAPE MODE (even if he passes by to another zombie he
        // doesn't care because he is already escaping)

        for zombie_position in positions.iter(){
            old_map[zombie_position.0][zombie_position.1]=None;
           // println!("--------------A)I have modified this: [{},{}]",zombie_position.0, zombie_position.1);
        }
        old_map
    }

    pub(crate) fn map_converter_normal(mut old_map: Vec<Vec<Option<Tile>>>, positions:&HashSet<(usize,usize)>) -> Vec<Vec<Option<Tile>>> {
        // as the previous but removes also the tiles around it so it's suggested in other mods,
        // especially when the energy is high and he is not hungry/thirsty

        let offsets:[(i32,i32);8] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];

        for zombie_position in positions.iter(){

            for &(dx, dy) in &offsets {
                let neighbour_x = zombie_position.0 as i32 + dx;
                let neighbour_y = zombie_position.1 as i32 + dy;
                if neighbour_x >=0 && neighbour_x < old_map.len() as i32{
                    if neighbour_y>=0 && neighbour_y < old_map[neighbour_x as usize].len() as i32{

                        old_map[neighbour_x as usize][neighbour_y as usize]=None;
                       // println!("--------------B) I have modified this: [{},{}]",neighbour_x, neighbour_y);
                    }
                }
            }
        }
        old_map
    }

    pub(crate) fn check_cannot_go(robert:& RobertNeville, _world: &World, dir:&Direction)-> (bool, Option<(usize,usize)>) {
        // check if it can go in that direction or if there is a zombie. return true if cannot go
        // and where is it
        let rob_x=robert.robot.coordinate.get_row();
        let rob_y=robert.robot.coordinate.get_col();
        match dir {
            Direction::Up => {
                if rob_x as i32 -1 >=0{
                    if robert.knowledge.zombies.contains(&(rob_x-1, rob_y)){
                        (true, Some((rob_x-1,rob_y)))
                    }
                    else{
                        (false, None)
                    }
                }
                else{
                    (false, None)
                }
            }
            Direction::Down => {
                if robert.knowledge.zombies.contains(&(rob_x+1, rob_y)){
                    (true, Some((rob_x+1,rob_y)))
                }
                else{
                    (false, None)
                }

            }
            Direction::Left => {
                if robert.knowledge.zombies.contains(&(rob_x, rob_y+1)){
                    (true, Some((rob_x,rob_y+1)))
                }
                else{
                    (false, None)
                }
            }
            Direction::Right => {
                if rob_y as i32 -1 >=0{
                    if robert.knowledge.zombies.contains(&(rob_x, rob_y-1)){
                        (true, Some((rob_x,rob_y-1)))
                    }else{
                        (false, None)
                    }
                }
                else{
                    (false,None)
                }
            }
        }
    }

    pub(crate) fn generate_random_direction()-> Direction{
        let r = thread_rng().gen_range(0..4);
        match r {
            0 => {
                return Direction::Up;
            }
            1 => {
                return Direction::Down;
            }
            2 => {
                return Direction::Right;
            }
            3 => {
                return Direction::Left;
            }
            _ => {
                return Direction::Left
            }
        }
    }

    // Debug Purpose
    #[allow(dead_code)]
    pub(crate) fn get_quantity(c: &Content) -> usize {
        match c {
            | Content::Rock(x)
            | Content::Tree(x)
            | Content::Garbage(x)
            | Content::Coin(x)
            | Content::Water(x)
            | Content::Market(x)
            | Content::JollyBlock(x)
            | Content::Bush(x)
            | Content::Fish(x) => *x,
            | Content::Bin(_) | Content::Crate(_) | Content::Bank(_) => 0,
            | Content::Fire | Content::Building | Content::Scarecrow | Content::None => 0,
        }
    }

    // Debug Purpose
    #[allow(dead_code)]
    pub(crate) fn print_map(map: Option<Vec<Vec<Option<Tile>>>>, size: (usize, usize)) {
        println!("---------ROBOT_MAP------------");
        for i in 0..10 {
            print!("          {i}   ")
        }
        println!();
        let mut y: i32 = 0;
        for i in map.unwrap().iter() {
            if y as usize > size.0 {
                break;
            }
            let mut x = 0;
            for j in i.iter() {
                if x as usize > size.1 {
                    break;
                }
                match j {
                    Some(t) => {
                        print!(
                            " {:?}({}({})) ",
                            t.tile_type,
                            t.content,
                            get_quantity(&t.content),
                        );
                    }
                    None => { print!(" ################ ") }
                }
                x += 1;
            }

            println!("");
            y += 1;
        }
        println!("\n-----------------------------------");
    }
}