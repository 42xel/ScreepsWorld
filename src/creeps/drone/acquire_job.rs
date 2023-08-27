use std::{collections::hash_map, convert::identity,};


use screeps::{
    constants::ResourceType, 
    objects::Creep, 
    StructureObject, 
    find, HasPosition, Room, Position, Ruin, Source, StructureSpawn, StructureExtension, ConstructionSite, StructureController, HasTypedId, MaybeHasTypedId
};
        
use crate::{my_wasm::UnwrapJsExt, utils::unord::UnOrd, creeps::CreepName};

use super::{TargetEnum};


/**
The game object corresponding to [`Target`], valid only for one tick.

Technical choice
---
Because most find functions provide directly objects,
and we need to work directly with objects for example to use their position,
I introduce this intermediary structure.

This structure owns its object for convenience.
The function [`acquire_target`] needs a hold on the underlying data either way,
providing that hold through a single [`TargetByObj`] variable greatly reduce the number of variable (and reduce a bit memory usage),
compared to potentially owning one Vec of each variant type.

Unstable
---
If I ever do put some of the necessary Vec in memory for usage accross time and creeps, as opposed to making queries,
making this structure hold only reference might make more sens.
*/
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord,)]
pub(self) enum TargetByObj {
    Ruin(UnOrd<Ruin>),
    Source(UnOrd<Source>),
    Extension(UnOrd<StructureExtension>),
    Spawn(UnOrd<StructureSpawn>),
    _ConstructionSiteMax,
    ConstructionSite(UnOrd<ConstructionSite>),
    Controller(UnOrd<StructureController>),
}

impl From<Ruin> for TargetByObj { fn from(o: Ruin) -> Self { Self::Ruin(o.into()) } }
impl From<Source> for TargetByObj { fn from(o: Source) -> Self { Self::Source(o.into()) } }
impl From<StructureExtension> for TargetByObj { fn from(o: StructureExtension) -> Self { Self::Extension(o.into()) } }
impl From<StructureSpawn> for TargetByObj { fn from(o: StructureSpawn) -> Self { Self::Spawn(o.into()) } }
impl From<ConstructionSite> for TargetByObj { fn from(o: ConstructionSite) -> Self { Self::ConstructionSite(o.into()) } }
impl From<StructureController> for TargetByObj { fn from(o: StructureController) -> Self { Self::Controller(o.into()) } }

impl From<TargetByObj> for TargetEnum {
    fn from(value: TargetByObj) -> Self {
        match value {
            TargetByObj::Source(o) => Self::Source(o.id()),
            TargetByObj::Ruin(o) => Self::Ruin(o.id()),
            TargetByObj::Extension(o) => Self::Extension(o.id()),
            TargetByObj::Spawn(o) => Self::Spawn(o.id()),
            TargetByObj::ConstructionSite(o) => Self::ConstructionSite(o.try_id().unwrap_js()),
            TargetByObj::Controller(o) => Self::Controller(o.id()),
            TargetByObj::_ConstructionSiteMax => unreachable!(),
        }
    }
}
impl HasPosition for TargetByObj {
    #[doc = " Position of the object."]
    fn pos(&self) -> Position {
        match self {
            Self::Source(t) => t.pos(),
            Self::Ruin(t) => t.pos(),
            Self::Extension(t) => t.pos(),
            Self::Spawn(t) => t.pos(),
            Self::ConstructionSite(t) => t.pos(),
            Self::Controller(t) => t.pos(),
            Self::_ConstructionSiteMax => unreachable!(),
        }
    }
}

/// returns the closest ruin containing energy, if any.
fn try_origin_ruin(room: &Room, pos: &Position) -> Option<screeps::Ruin> {
    room.find(find::RUINS, None).into_iter()
    .filter(|r| r.store().get_used_capacity(Some(ResourceType::Energy)) > 0)
    .min_by_key(|r| pos.get_range_to(r.pos()))
}

/**
Returns the best structure destination target in working range.
*/
fn try_dest_structure_near(pos: &Position) -> Option<TargetByObj> {
    pos.find_in_range(find::MY_STRUCTURES, 3)    
    .into_iter().filter_map(|o| match o {
        StructureObject::StructureController(s) => Some(TargetByObj::from(s)),
        StructureObject::StructureSpawn(s) if pos.get_range_to(s.pos()) <= 1
            && s.store().get_free_capacity(Some(ResourceType::Energy)) >= 50 =>
            Some(TargetByObj::from(s)),
        StructureObject::StructureExtension(s) if pos.get_range_to(s.pos()) <= 1
            && s.store().get_free_capacity(Some(ResourceType::Energy)) >= 1 =>
            Some(TargetByObj::from(s)),
        _ => None,
    }).min()
}

/**
Returns the best structure destination target in the room.
*/
fn try_dest_structure(room: &Room, pos: &Position) -> Option<TargetByObj> {
    room.find(find::MY_STRUCTURES, None)
    .into_iter().filter_map(|o| match o {
        StructureObject::StructureExtension(s)
            if s.store().get_free_capacity(Some(ResourceType::Energy)) >= 1 => { let p = s.pos(); Some((TargetByObj::from(s), pos.get_range_to(p))) },
        StructureObject::StructureSpawn(s)
            if s.store().get_free_capacity(Some(ResourceType::Energy)) >= 50  => { let p = s.pos(); Some((TargetByObj::from(s), pos.get_range_to(p))) },
        StructureObject::StructureController(s) => { let p = s.pos(); Some((TargetByObj::from(s), pos.get_range_to(p))) },
        _ => None,
    }).min().map(|p| p.0)
}

/**
Returns the closest structure destination target in the room.
*/
fn try_dest_structure_by_range(room: &Room, pos: &Position) -> Option<TargetByObj> {room.find(find::MY_STRUCTURES, None)
    .into_iter().filter_map(|o| match o {
        StructureObject::StructureExtension(s)
            if s.store().get_free_capacity(Some(ResourceType::Energy)) >= 1  => Some(TargetByObj::from(s)),
        StructureObject::StructureSpawn(s)
            if s.store().get_free_capacity(Some(ResourceType::Energy)) >= 50  => Some(TargetByObj::from(s)),
        StructureObject::StructureController(s) => Some(TargetByObj::from(s)),
        _ => None,
    }).min_by_key(|t| pos.get_range_to(t.pos()))
}

/**
Returns the closest construction site
 */
fn try_dest_construction_site(room: &Room, pos: &Position) -> Option<TargetByObj> {
    // we're only filtering by range, but I still copy try_dest_structure template instead of using a min_by_key for brainlessness and generalisation
    room.find(find::CONSTRUCTION_SITES, None)
    .iter().map(|cs| (TargetByObj::from(cs.clone()), pos.get_range_to(cs.pos())))
    .min().map(|p| p.0)
}

/**
Acquires a target for a creep without one.

The creep can be full, empty or in between, a smart decision will be taken.
This is particularly useful on resets, so as to not pickup a task on the other end of the room.

One consequence is that sometimes, a creep will empty only part of his inventory (eg in extension) and immediately go back fill it more as opposed to empty completely in several targe.
This is hardly a fault on the target acquisition part though, more of a sign that maybe the targets are close and the creep has too many carry parts consiidering the short trips it has to make.
 */
pub(crate) fn acquire_target(creep: &Creep, target_entry: hash_map::VacantEntry<'_, CreepName, TargetEnum>) -> Option<()> {
    let pos = creep.pos();
    let room = creep.room().ok_or(()).ok()?;

    let used_capacity = creep.store().get_used_capacity(Some(ResourceType::Energy)) as i32;
    let free_capacity = creep.store().get_free_capacity(Some(ResourceType::Energy));

    // Where to take energy from
    // follows priority (vaiant order in the enum) then range
    let origin = if free_capacity == 0 { None }
    else {
        try_origin_ruin(&room, &pos).map(|r| TargetByObj::Ruin(r.into())).or_else(||
        pos.find_closest_by_path(find::SOURCES_ACTIVE, None).map(|s| TargetByObj::Source(s.into()))
        )
    };

    // Where to spend energy into
    // if energy available is low, follows priority (variant order) then range.
    // else, follows range, except for controller, which is always a sink.
    let destination: Option<TargetByObj> = 'value: {if used_capacity == 0 { None }
    else if room.energy_available() < room.energy_capacity_available() / 2 {
        let structure = try_dest_structure(&room, &pos);
        if matches!(structure, Some(ref s) if *s > TargetByObj::_ConstructionSiteMax) {
            break 'value structure;
        };
        [
            structure,
            try_dest_construction_site(&room, &pos),
        ].into_iter().filter_map(|s| { 
           if let Some(s) = s {
                let p = s.pos();
               Some((s, pos.get_range_to(p)))
           }
           else {None} })
           //let p = (*s)?.clone().pos(); Some(( s, pos.get_range_to(p))) })
        .min().map(|p| p.0).clone()
    }
    else {
        if let Some(v) = try_dest_structure_near(&pos) {
            break 'value Some(v);
        }
        [
            try_dest_structure_by_range(&room, &pos),
            try_dest_construction_site(&room, &pos),
        ].into_iter().filter_map(identity).min_by_key(|t| t.pos())
    }};

    //if neither full nor empty, make a decision based on range.
    let target = Some(TargetEnum::from( match (destination, origin) {
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
