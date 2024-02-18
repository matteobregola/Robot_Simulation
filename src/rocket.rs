use std::collections::HashMap;
use rocket_dyn_templates::{context, Template};
use rocket::{launch, get, routes, State};
use rocket::fs::FileServer;
use crate::structs::structs::RocketData;

#[get("/")]
fn index(state: &State<RocketData>) -> Template {
    // Number of times that an action has been executed
    let context: HashMap<&str, String> = [("action_values", format!("{:?}",state.action_values))]
        .iter().cloned().collect();
    Template::render("index", &context)
}

#[get("/life_stats")]
fn life_stats(state: &State<RocketData>) -> Template {
    // values: Energy, Thirst, Hunger. 3 Vectors
    let context: HashMap<&str, String> = [
        ("energy_vector", format!("{:?}",state.energy_vector)),
        ("thirst_vector", format!("{:?}", state.thirst_vector)),
        ("hunger_vector", format!("{:?}",state.hunger_vector)),
    ]
        .iter().cloned().collect();
    Template::render("life_stats", &context)
}

#[get("/discovery_stats")]
fn discovery_stats(state: &State<RocketData>) -> Template {
    // Heatmap of the visited positions. nxn Matrix
    let context: HashMap<&str, String> = [("position_matrix", format!("{:?}",state.heatmap)),("map_dim",state.map_dim.to_string())]
        .iter().cloned().collect();
    Template::render("discovery_stats", &context)
}

#[get("/zombies_stats")]
fn zombies_stats(state: &State<RocketData>) -> Template {
    // Numero di zombie e building incontrati. Interi
    let context: HashMap<&str, String> = [("n_zombies", state.n_zombies.to_string()), ("n_shelters",state.n_shelters.to_string())]
        .iter().cloned().collect();
    Template::render("zombies_stats", &context)
}


#[launch]
pub fn rocket() ->_ {
    rocket::build()
        .mount("/static", FileServer::from("static"))
        .mount("/", routes![index,life_stats,discovery_stats,zombies_stats])
        .attach(Template::fairing())
}
