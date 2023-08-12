use std::any::Any;

use log::*;

use screeps::{
    constants::{Part, ResourceType}, 
    objects::Creep, SharedCreepProperties, find, HasPosition, Room, StructureController, game::{get_object_by_id_typed, get_object_by_id_erased}, MaybeHasId, StructureType, StructureObject
};
use wasm_bindgen::JsCast;

use crate::my_wasm::*;

pub mod count {
    use super::super::*;

    thread_local! {
        //TODO make it Room local
        //TODO make a struct/trait for several roles with several bodies (>3)
        pub static DRONE: RefCell<u32> = Default::default();
        pub static UNKNOWN: RefCell<u32> = Default::default();

        pub static MAX_DRONE: u32 = 10;
    }
}
pub(crate) fn run_creep(creep: &Creep) {
    if creep.spawning() {return;}
    let Some(room) = &creep.room() else { warn!("couldn't resolve creep room"); return;};

    let name = creep.name();
    debug!("running creep {}", name);

    //let target = creep_targets.entry(name);
    let role = creep.name().chars().next().expect_js("creep doesn't have a name !".into());

    match role {
        'd'|'H' => run_drone(room, creep),
        _ => (),
    }
}

fn run_drone(room: &Room, creep: &Creep) {
    let pos = creep.pos();

    let mut used_capacity = creep.store().get_used_capacity(Some(ResourceType::Energy)) as i32;
    let mut free_capacity = creep.store().get_free_capacity(Some(ResourceType::Energy));
    let vspawns;
    let vstructures;

    let (spawn, controller) = if used_capacity > 0 {
        vspawns = room.find(find::MY_SPAWNS, None);
        vstructures = room.find(find::MY_STRUCTURES, None);
        (if let Some(spawn) = vspawns.get(0)
        {
            if creep.transfer(spawn, ResourceType::Energy, Some(used_capacity as u32)).is_ok() {
                //anticipate cargo emptying so as to move the same turn
                free_capacity += used_capacity;
                used_capacity = 0;
                None
            } else {
                Some(spawn)
            }
        } else {
            warn!("Couldn't find spawn"); None
        },
        if let Some(controller) = vstructures.iter().find_map(|s| if let StructureObject::StructureController(c) = s {Some(c)} else {None})
        {
            if creep.upgrade_controller(&controller).is_ok() {
                let w = creep.get_active_bodyparts(Part::Work) as i32;
                //anticipate cargo emptying so as to move the same turn
                free_capacity -= w;
                used_capacity += w;
            }
            Some(controller)
        } else {
            warn!("Couldn't find controller"); None
        })
    } else {(None, None)};
    let source = if free_capacity > 0 {
        if let Some(source) = pos.find_closest_by_path(find::SOURCES_ACTIVE, None) {
            if creep.harvest(&source).is_ok() {
                let w = creep.get_active_bodyparts(Part::Work) as i32 * 2;
                //anticipate cargo filling so as to move the same turn
                free_capacity -= w;
                used_capacity += w;
                None
            } else {
                Some(source)
            }
        } else { warn!("couldn't find any active source."); None}
    } else {None};

    let target = match (spawn, controller) {
        (Some(s), _) if s.store().get_free_capacity(Some(ResourceType::Energy)) >= 100 => Some(s.pos()),
        (_, Some(c)) => Some(c.pos()),
        _ => None,
    };

    if used_capacity <= 0 {
        if let Some(s) = source {
            let _ = creep.move_to(s);
        }
    }
    else if free_capacity <= 0 {
        if let Some(t) = target {
            let _ = creep.move_to(t);
        }
    }
    //if neither full nor empty, make a decision based on range.
    else if let (Some(t), Some(source)) = (target, source) {
        if (used_capacity * (pos.get_range_to(source.pos()) - 1) as i32) <= free_capacity * (pos.get_range_to(t) - 1) as i32 {
            let _ = creep.move_to(source);
        } else {
            let _ = creep.move_to(t);
        }
    }
}