use std::{cell::RefCell, collections::{HashMap, hash_map}, marker::PhantomData};

use crate::{prelude::*, my_wasm::UnwrapJsExt};
use screeps::{
    constants::{Part, ResourceType},
    objects::Creep,
    Room, HasPosition, SharedCreepProperties, find, ObjectId, StructureSpawn, StructureController, Source, control, ErrorCode, Ruin, MaybeHasPosition, Position,
    traits::{HasId, HasTypedId, MaybeHasId, HasNativeId, MaybeHasTypedId, MaybeHasNativeId,}
};
use wasm_bindgen::{throw_val, throw_str};
use web_sys::console::warn;

use super::{JobError, Progress};

/// this enum will represent a drone's lock on a specific target object, storing a js reference
/// to the object id so that we can grab a fresh reference to the object each successive tick,
/// since screeps game objects become 'stale' and shouldn't be used beyond the tick they were fetched
#[derive(Debug, PartialEq, PartialOrd)]
pub(crate) enum TargetByID {
    Source(ObjectId<Source>),
    Ruin(ObjectId<Ruin>),
    Spawn(ObjectId<StructureSpawn>),
    Controller(ObjectId<StructureController>),
}
/// the game object corresponding to [`TargetByID`], valid only for one tick.
#[derive(Debug)]
pub(self) enum TargetByObj {
    Source(Source),
    Ruin(Ruin),
    Spawn(StructureSpawn),
    Controller(StructureController),
}
impl From< TargetByObj > for TargetByID {
    fn from(value: TargetByObj) -> Self {
        match value {
            TargetByObj::Source(o) => Self::Source(o.id()),
            TargetByObj::Ruin(o) => Self::Ruin(o.id()),
            TargetByObj::Spawn(o) => Self::Spawn(o.id()),
            TargetByObj::Controller(o) => Self::Controller(o.id()),
        }
    }
}
impl<'a> HasPosition for TargetByObj {
    #[doc = " Position of the object."]
    fn pos(&self) -> Position {
        match self {
            Self::Source(t) => t.pos(),
            Self::Ruin(t) => t.pos(),
            Self::Spawn(t) => t.pos(),
            Self::Controller(t) => t.pos(),
        }
    }
}

thread_local! {
    static TARGETS: RefCell<HashMap<String, TargetByID>> = Default::default();
}

fn move_drone_to<T: HasPosition>(creep: &Creep, target: T) -> Result<Progress, ErrorCode> {
    if let Err(err_m) = creep.move_to(target) { match err_m {
        ErrorCode::NoPath | ErrorCode::NotFound /*The creep has no memorized path to reuse. */ => {
            warn!("No path{:?}", err_m);
            let _ = creep.say("🚫", true);
            Err(err_m)
        },
        ErrorCode::Tired => { let _ = creep.say("😴", true); Ok(Progress::Todo) },
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
        ErrorCode::NoBodypart => {
            warn!("crippled creep {}, committing Sepuku", creep.name());
            let _ = creep.say("🤕☠️", true).and(creep.suicide());
            Err(e)
        },
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
        ErrorCode::NoBodypart => {
            warn!("crippled creep {}, committing Sepuku", creep.name());
            let _ = creep.say("🤕☠️", true).and(creep.suicide());
            Err(e)
        },
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
        ErrorCode::Full => Ok(Progress::Done),
        ErrorCode::NoBodypart => {
            warn!("crippled creep {}, committing Sepuku", creep.name());
            let _ = creep.say("🤕☠️", true).and(creep.suicide());
            Err(e)
        },
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
        ErrorCode::NoBodypart => {
            warn!("crippled creep {}, committing Sepuku", creep.name());
            let _ = creep.say("🤕☠️", true).and(creep.suicide());
            Err(e)
        },
        ErrorCode::NotEnough => Err(e),
        ErrorCode::Busy /* Still being spawned */ => Ok(Progress::Frozen),
        #[allow(unreachable_patterns)]
        _ | ErrorCode::NotOwner | ErrorCode::InvalidTarget | ErrorCode::NotEnough | ErrorCode::InvalidArgs => {
            throw_str(&format!("{:?}", e)) },
    }}
    else if creep.get_active_bodyparts(Part::Work) as u32 > creep.store().get_used_capacity(Some(ResourceType::Energy)) {
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
                TargetByID::Source(source) => harvest_source(creep, source.resolve().unwrap_js()),
                TargetByID::Spawn(spawn) => transfer_spawn(creep, spawn.resolve().unwrap_js()),
                TargetByID::Controller(controller) => upgrade_controller_controller(creep, controller.resolve().unwrap_js()),
                TargetByID::Ruin(ruin) => withdraw_from_ruin(creep, ruin.resolve().unwrap_js()),
            }
        } else { Ok(Progress::Frozen)}; // no need to clear
        match r {
            Ok(Progress::Done) | Err(_) => {targets.remove(&name);},
            _ => (),
        }
        if let hash_map::Entry::Vacant(target) = targets.entry(name) {
            acquire_target::acquire_target(creep, target);
        };
//        if r != Ok(Progress::Done) {break;};  //Too unstable for now, must ensure the same task is not attempted inefinitely the same tick
    });
}
