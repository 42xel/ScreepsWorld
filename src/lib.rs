#![feature(local_key_cell_methods)]

use std::cell::RefCell;
//use std::collections::{hash_map::Entry, HashMap};

mod prelude;
use prelude::*;

use screeps::{
    constants::{Part,},
//    enums::StructureObject,
    game,// find,
//    local::ObjectId,
//    objects::{Source, StructureController, StructureSpawn},
    prelude::*,
};
use wasm_bindgen::prelude::*;

use crate::my_wasm::UnwrapJsExt;

mod logging;
// add wasm_bindgen to any function you would like to expose for call from js
#[wasm_bindgen]
pub fn setup() {
    logging::setup_logging(logging::Info);
}

pub mod my_wasm;

pub mod creeps;
pub mod utils;

// this is one way to persist data between ticks within Rust's memory, as opposed to
// keeping state in memory on game objects - but will be lost on global resets!
thread_local! {
    static INIT: RefCell<bool> = RefCell::new(false);
}

fn census() {
   creeps::count::DRONE.with_borrow_mut(|cd|
    creeps::count::UNKNOWN.with_borrow_mut(|cu|
        {
             *cd = 0;
             for creep in game::creeps().values() {
                if let Some(c) = creep.name().chars().next(){
                    match c {
                        'd' => *cd += 1,
                        _   => *cu += 1,
                    }
                }
                else { error!("empty creep name !") }
            }
        }
    ));
}

pub fn init (b: &mut bool) {
    debug!("starting init");
    //TODO iter room

    //census();
    info!("initialization");

    *b = true;
}

// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    INIT.with_borrow_mut(|b| if !*b
        {init(b)});

    //CREEP_TARGETS.with_borrow_mut(|creep_targets| {
        debug!("running creeps");
        for creep in game::creeps().values() {
            creeps::run_creep(&creep);
        }
//    });

    debug!("running spawns");
    for spawn in game::spawns().values() {
        if let Some(_) = spawn.spawning() {continue;}
        debug!("running spawn {}", String::from(spawn.name()));

        census();

        if creeps::count::DRONE.with_borrow_mut(|cd| {
            if creeps::count::MAX_DRONE.with(|m| *m <= *cd) {return false;};

            
            let body1 = [Part::Work, Part::Carry, Part::Move];
//            let body2 = [Part::Carry, Part::Work, Part::Move, Part::Carry, Part::Work, Part::Move,];
            let body2 = [Part::Carry, Part::Work, Part::Move, Part::Carry, Part::Work, Part::Move, Part::Carry, Part::Work, Part::Move,];
            let name;

            let energy_available = spawn.room().unwrap_js().energy_available();

            let body = if spawn.room().unwrap_js().energy_capacity_available() < body2.iter().map(|p| p.cost()).sum()
            || *cd < 3 &&  energy_available < 400 {
                name = format!("d1-{}-{}", spawn.name(), game::time());
                Vec::from(body1)
            } else {
                name = format!("d2-{}-{}", spawn.name(), game::time());
                Vec::from(body2)
            };

            if  energy_available < body.iter().map(|p| p.cost()).sum() { return false; }

            // note that this bot has a fatal flaw; spawning a creep
            // creates Memory.creeps[creep_name] which will build up forever;
            // these memory entries should be prevented (todo doc link on how) or cleaned up
            if let Err(e) = spawn.spawn_creep(&body, &name) {
                warn!("couldn't spawn: {:?}", e); false
            } else { *cd += 1; true }
        }) {continue;}

//        [Part::Carry, Part::Work, Part::Carry, Part::Move, Part::Work, Part::Move,]
    }
//    info!("done! cpu: {}", game::cpu::get_used())
}

