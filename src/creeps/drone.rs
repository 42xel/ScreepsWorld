use std::{cell::RefCell, collections::{HashMap, hash_map}};

use crate::{prelude::*, my_wasm::UnwrapJsExt};
use screeps::{
    constants::{Part, ResourceType},
    objects::Creep, HasPosition, SharedCreepProperties, ObjectId, StructureSpawn, StructureController, Source, ErrorCode, Ruin, StructureExtension, ConstructionSite, MoveToOptions
};
use wasm_bindgen::{throw_str};


use super::{Progress};

/// this enum will represent a drone's lock on a specific target object, storing a js reference
/// to the object id so that we can grab a fresh reference to the object each successive tick,
/// since screeps game objects become 'stale' and shouldn't be used beyond the tick they were fetched
#[derive(Debug, PartialEq, PartialOrd)]
pub(crate) enum Target {
    Source(ObjectId<Source>),
    Ruin(ObjectId<Ruin>),
    Extension(ObjectId<StructureExtension>),
    Spawn(ObjectId<StructureSpawn>),
    _ConstructionSiteMax,
    ConstructionSite(ObjectId<ConstructionSite>),
    Controller(ObjectId<StructureController>),
}

thread_local! {
    static TARGETS: RefCell<HashMap<String, Target>> = Default::default();
}

fn error_no_body_part(creep: &Creep) -> Result<Progress, ErrorCode> {
    warn!("crippled creep {}, committing Sepuku", creep.name());
    let _ = creep.say("🤕☠️", true).and(creep.suicide());
    Err(ErrorCode::NoBodypart)
}

fn move_drone_to<T: HasPosition>(creep: &Creep, target: T) -> Result<Progress, ErrorCode> {
    if let Err(err_m) = creep.move_to_with_options(target, Some(MoveToOptions::new().ignore_creeps(true)))
    { match err_m {
        ErrorCode::NoPath | ErrorCode::NotFound /*The creep has no memorized path to reuse. */ => {
            warn!("No path{:?}", err_m);
            let _ = creep.say("🚫", true);
            Err(err_m)
        },
        ErrorCode::Tired => { let _ = creep.say("🐌", true); Ok(Progress::Todo) },
        ErrorCode::Busy => Ok(Progress::Todo),
        #[allow(unreachable_patterns)]
        _ | ErrorCode::NotOwner | ErrorCode::InvalidTarget => throw_str(&format!("{:?}", err_m)),
    }} else {
        Ok(Progress::Doing)
    }
}

fn harvest_source(creep: &Creep, source: Source) -> Result<Progress, ErrorCode> {
    if let Err(e) = creep.harvest(&source) { match e {
        ErrorCode::NotInRange => match move_drone_to(creep, source) {
            Err(ErrorCode::NotFound) => Ok(Progress::Todo),
            Err(ErrorCode::NoPath) => { Err(ErrorCode::NoPath)},
            r => r
        },
        ErrorCode::Busy /* Still being spawned */ => Ok(Progress::Frozen),
        ErrorCode::NoBodypart => error_no_body_part(creep),
        ErrorCode::NotEnough => Err(e),
        #[allow(unreachable_patterns)]
        _ | ErrorCode::NotOwner | ErrorCode::NotFound | ErrorCode::Tired | ErrorCode::InvalidTarget => {
            throw_str(&format!("{:?}", e)) },
    }}
    else if creep.get_active_bodyparts(Part::Work) as i32 * 2 > creep.store().get_free_capacity(Some(ResourceType::Energy)) {
        Ok(Progress::Done)
    } else { Ok(Progress::Doing) }
}

fn withdraw_from_ruin(creep: &Creep, ruin: Ruin) -> Result<Progress, ErrorCode> {
    if let Err(e) = creep.withdraw(&ruin, ResourceType::Energy, None) { match e {
        ErrorCode::NotInRange => match move_drone_to(creep, ruin) {
            Err(ErrorCode::NotFound) => Ok(Progress::Todo),
            Err(ErrorCode::NoPath) => { Err(ErrorCode::NoPath)},
            r => r
        },
        ErrorCode::Full | ErrorCode::NotEnough => Ok(Progress::Done),
        ErrorCode::Busy /* Still being spawned */ => Ok(Progress::Frozen),
        ErrorCode::NoBodypart => error_no_body_part(creep),
        #[allow(unreachable_patterns)]
        _ | ErrorCode::NotOwner | ErrorCode::InvalidArgs | ErrorCode::InvalidTarget => {
            throw_str(&format!("{:?}", e)) },
    }}
    else if creep.get_active_bodyparts(Part::Work) as i32 * 2 > creep.store().get_free_capacity(Some(ResourceType::Energy)) {
        Ok(Progress::Done)
    } else { Ok(Progress::Doing) }
}

fn transfer_spawn(creep: &Creep, spawn: StructureSpawn) -> Result<Progress, ErrorCode> {
    if let Err(e) = creep.transfer(&spawn, ResourceType::Energy, None) { match e {
        ErrorCode::NotInRange => match move_drone_to(creep, spawn) {
            Err(ErrorCode::NotFound) => Ok(Progress::Todo),
            Err(ErrorCode::NoPath) => { Err(ErrorCode::NoPath)},
            r => r
        },
        ErrorCode::Full | ErrorCode::NotEnough => Ok(Progress::Done),
        ErrorCode::NoBodypart => error_no_body_part(creep),
        #[allow(unreachable_patterns)]
        _ | ErrorCode::NotOwner | ErrorCode::InvalidTarget | ErrorCode::NotEnough | ErrorCode::Busy | ErrorCode::InvalidArgs => {
            throw_str(&format!("{:?}", e)) },
    }}
    else {
        Ok(Progress::Done)
    }
}

fn transfer_extension(creep: &Creep, extension: StructureExtension) -> Result<Progress, ErrorCode> {
    if let Err(e) = creep.transfer(&extension, ResourceType::Energy, None) { match e {
        ErrorCode::NotInRange => match move_drone_to(creep, extension) {
            Err(ErrorCode::NotFound) => Ok(Progress::Todo),
            Err(ErrorCode::NoPath) => { Err(ErrorCode::NoPath)},
            r => r
        },
        ErrorCode::Full | ErrorCode::NotEnough => Ok(Progress::Done),
        ErrorCode::NoBodypart => error_no_body_part(creep),
        #[allow(unreachable_patterns)]
        _ | ErrorCode::NotOwner | ErrorCode::InvalidTarget | ErrorCode::NotEnough | ErrorCode::Busy | ErrorCode::InvalidArgs => {
            throw_str(&format!("{:?}", e)) },
    }}
    else {
        Ok(Progress::Done)
    }
}

fn upgrade_controller_controller(creep: &Creep, controller: StructureController) -> Result<Progress, ErrorCode> {
    if let Err(e) = creep.upgrade_controller(&controller) { match e {
        ErrorCode::NotInRange => match move_drone_to(creep, controller) {
            Err(ErrorCode::NotFound) => Ok(Progress::Todo),
            Err(ErrorCode::NoPath) => { Err(ErrorCode::NoPath)},
            r => r
        },
        ErrorCode::NoBodypart => error_no_body_part(creep),
        ErrorCode::NotEnough => Err(e),
        ErrorCode::Busy /* Still being spawned */ => Ok(Progress::Frozen),
        #[allow(unreachable_patterns)]
        _ | ErrorCode::NotOwner | ErrorCode::InvalidTarget | ErrorCode::NotEnough | ErrorCode::InvalidArgs => {
            throw_str(&format!("{:?}", e)) },
    }}
    else if creep.get_active_bodyparts(Part::Work) as u32 > creep.store().get_used_capacity(Some(ResourceType::Energy)) {
        Ok(Progress::Done)
    } else { let _ = move_drone_to(creep, controller); Ok(Progress::Doing) }
}

fn build(creep: &Creep, construction_site: ConstructionSite) -> Result<Progress, ErrorCode> {
    if let Err(e) = creep.build(&construction_site) { match e {
        ErrorCode::NotInRange => match move_drone_to(creep, construction_site) {
            Err(ErrorCode::NotFound) => Ok(Progress::Todo),
            Err(ErrorCode::NoPath) => { Err(ErrorCode::NoPath)},
            r => r
        },
        ErrorCode::NotEnough => Ok(Progress::Done),
        ErrorCode::NoBodypart => error_no_body_part(creep),
        ErrorCode::Busy /* Still being spawned */ => Ok(Progress::Frozen),
        ErrorCode::InvalidTarget => {
            warn!("The target is not a valid construction site object or the structure cannot be built here (probably because of a creep at the same square).");
            Err(ErrorCode::InvalidTarget)
        },
        #[allow(unreachable_patterns)]
        _ | ErrorCode::NotOwner => {
            throw_str(&format!("{:?}", e)) },
    }}
    else if creep.get_active_bodyparts(Part::Work) as u32 * 5 > creep.store().get_used_capacity(Some(ResourceType::Energy)) {
        Ok(Progress::Done)
    } else { Ok(Progress::Doing) }
}

pub mod acquire_target;


pub(super) fn run_drone(creep: &Creep) {
    let name = creep.name();
    TARGETS.with_borrow_mut(|targets| //loop
    {
        let r = if let Some(target) = targets.get(&name) {
            match target {
                Target::Source(source) => harvest_source(creep, source.resolve().unwrap_js()),
                Target::Spawn(spawn) => transfer_spawn(creep, spawn.resolve().unwrap_js()),
                Target::Controller(controller) => upgrade_controller_controller(creep, controller.resolve().unwrap_js()),
                Target::Ruin(ruin) => withdraw_from_ruin(creep, ruin.resolve().unwrap_js()),
                Target::Extension(extension) => transfer_extension(creep, extension.resolve().unwrap_js()),
                Target::ConstructionSite(construction_site) => build(creep, construction_site.resolve().unwrap_js()),
                Target::_ConstructionSiteMax => unreachable!(),
            }
        } else { Ok(Progress::Frozen)}; // no need to clear
        match r {
            Ok(Progress::Done) | Err(_) => {targets.remove(&name);},
            _ => (),
        }
        match r {
            Ok(p) => match p {
                Progress::Frozen => creep.say("❄️", true).unwrap_or_default(),
                Progress::Todo if creep.fatigue() == 0 => creep.say("😴", true).unwrap_or_default(),
                Progress::Done => creep.say("✅", true).unwrap_or_default(),
                _ => (),
            },
            Err(e) => match e {
                ErrorCode::NoPath | ErrorCode::Busy | ErrorCode::NotFound | ErrorCode::NotInRange | ErrorCode::Tired | ErrorCode::NoBodypart => (),
                _ => creep.say("❎", true).unwrap_or_default(),
            },
        }
        if let hash_map::Entry::Vacant(target) = targets.entry(name) {
            acquire_target::acquire_target(creep, target);
        };
//        if r != Ok(Progress::Done) {break;};  //Too unstable for now, must ensure the same task is not attempted inefinitely the same tick
    });
}
