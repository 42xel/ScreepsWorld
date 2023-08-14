use std::collections::hash_map;

use screeps::{StructureObject, constants::ResourceType, objects::Creep, find, HasPosition, Room, Position, Ruin, Source, StructureSpawn, StructureExtension, ConstructionSite, StructureController, HasTypedId, MaybeHasTypedId};
        
use crate::my_wasm::UnwrapJsExt;

use super::{Target};


/**
The game object corresponding to [`TargetByID`], valid only for one tick.

Technical choice
---
Because most the find functions provide directly objects,
and we need to work directly with objects for example to use their position,
I introduce this intermediary structure.

This structure owns its object for convenience, at the cost of a handful clones here and there, notably data which can't be moved out of the Vec produced by [`find`]
The function [`acquire_target`] needs a hold on the udnerlying data either way,
providing that hold through a single [`TargetByObj`] variable greatly reduce the number of variable (and reduce a bit memory usage),
compared to potentially owning one Vec of each variant type.

Unstable
---
If I ever do put some of the necessary Vec in memory for usage accross time and creeps, as opposed to making queries,
making this structure hold only reference might make more sens.
*/
#[derive(Debug)]
pub(self) enum TargetByObj {
    Source(Source),
    Ruin(Ruin),
    Spawn(StructureSpawn),
    Extension(StructureExtension),
    ConstructionSite(ConstructionSite),
    Controller(StructureController),
}
impl From< TargetByObj > for Target {
    fn from(value: TargetByObj) -> Self {
        match value {
            TargetByObj::Source(o) => Self::Source(o.id()),
            TargetByObj::Ruin(o) => Self::Ruin(o.id()),
            TargetByObj::Spawn(o) => Self::Spawn(o.id()),
            TargetByObj::Extension(o) => Self::Extension(o.id()),
            TargetByObj::ConstructionSite(o) => Self::ConstructionSite(o.try_id().unwrap_js()),
            TargetByObj::Controller(o) => Self::Controller(o.id()),
        }
    }
}
impl HasPosition for TargetByObj {
    #[doc = " Position of the object."]
    fn pos(&self) -> Position {
        match self {
            Self::Source(t) => t.pos(),
            Self::Ruin(t) => t.pos(),
            Self::Spawn(t) => t.pos(),
            Self::Extension(t) => t.pos(),
            Self::ConstructionSite(t) => t.pos(),
            Self::Controller(t) => t.pos(),
        }
    }
}

fn try_origin_ruin(room: &Room, pos: &Position) -> Option<screeps::Ruin> {
    room.find(find::RUINS, None).iter()
    .filter(|r| r.store().get_used_capacity(Some(ResourceType::Energy)) > 0)
    .min_by_key(|r| pos.get_range_to(r.pos())).cloned()
}

pub(crate) fn acquire_target(creep: &Creep, target_entry: hash_map::VacantEntry<'_, String, Target>) -> Option<()> {
    let pos = creep.pos();
    let room = creep.room().ok_or(()).ok()?;

    let used_capacity = creep.store().get_used_capacity(Some(ResourceType::Energy)) as i32;
    let free_capacity = creep.store().get_free_capacity(Some(ResourceType::Energy));

    // Where to take energy from
    let origin = if free_capacity == 0 { None }
    else {
        try_origin_ruin(&room, &pos).map(|r| TargetByObj::Ruin(r)).or_else(||
        pos.find_closest_by_path(find::SOURCES_ACTIVE, None).map(move |s| TargetByObj::Source(s.clone()))
        )
    };

    let vstructures;
    let vstructures_near;
    //let vconstructions_near;
    let vconstructions;
    // where to spend energy into
    let destination = if used_capacity == 0 { None }
    else {
        vstructures_near = pos.find_in_range(find::MY_STRUCTURES, 3);
        match vstructures_near.iter().find_map(|s| match s {
            StructureObject::StructureController(c) => Some(TargetByObj::Controller(c.clone())),
            StructureObject::StructureSpawn(s) if pos.get_range_to(s.pos()) <= 1
            && s.store().get_free_capacity(Some(ResourceType::Energy)) >= 50 =>
                Some(TargetByObj::Spawn(s.clone())),
            StructureObject::StructureExtension(x) if pos.get_range_to(x.pos()) <= 1
            && x.store().get_free_capacity(Some(ResourceType::Energy)) >= 50 =>
                Some(TargetByObj::Extension(x.clone())),
            _ => None,
            }) { Some(thing) => Some(thing),
        //None => { vconstructions_near = pos.find_in_range(find::MY_CONSTRUCTION_SITES, 1);
        //    match vconstructions_near.get(0) {
        //        Some(v) => Some(TargetByObj::ConstructionSite(v.clone())),
        None => {vstructures = room.find(find::MY_STRUCTURES, None);
            match vstructures.iter().filter_map(|o| match o {
                StructureObject::StructureSpawn(s) if s.store().get_free_capacity(Some(ResourceType::Energy)) > 50 => Some(s),
                _ => None,
            })
            .min_by_key(|s| pos.get_range_to(s.pos()))
            .map(|s| TargetByObj::Spawn(s.clone())) {
                Some(thing) => Some(thing),
        None => match vstructures.iter().filter_map(|o| match o {
                StructureObject::StructureExtension(x) if x.store().get_free_capacity(Some(ResourceType::Energy)) >= 50 => Some(x),
                _ => None,
            })
            .min_by_key(|x| pos.get_range_to(x.pos()))
            .map(|x| TargetByObj::Extension(x.clone())) {
                Some(thing) => Some(thing),
        None => { vconstructions = room.find(find::MY_CONSTRUCTION_SITES, None);
            match vconstructions.iter().min_by_key(|cs| pos.get_range_to(cs.pos()))
            .map(|cs| TargetByObj::ConstructionSite(cs.clone())) {
                Some(thing) => Some(thing),
        None => vstructures.iter().filter_map(|o| if let StructureObject::StructureController(c) = o {Some(c)} else {None})
            .min_by_key(|c| pos.get_range_to(c.pos()))
            .map(|c| TargetByObj::Controller(c.clone())),
            }
        },}}}}//}}
    
    };

    //if neither full nor empty, make a decision based on range.
    let target = Some(Target::from( match (destination, origin) {
        (Some(d), Some(o)) =>
            if (used_capacity * (pos.get_range_to(o.pos()) - 1) as i32) <= free_capacity * (pos.get_range_to(d.pos()) - 1) as i32 {
                o
            } else { d
            },
        (None, Some(o)) => o,
        (Some(d), None) => d,
        (None, None) => {return None;},
    }));
    target.map(|t| target_entry.insert(t)).as_deref();
    Some(())
}
