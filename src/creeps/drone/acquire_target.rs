use std::collections::hash_map;

use screeps::{StructureObject, constants::ResourceType, objects::Creep, find, HasPosition};
        
use super::{TargetByObj, TargetByID};

pub(crate) fn acquire_target(creep: &Creep, target_entry: hash_map::VacantEntry<'_, String, TargetByID>) -> Option<()> {
    let pos = creep.pos();
    let room = creep.room().ok_or(()).ok()?;

    let used_capacity = creep.store().get_used_capacity(Some(ResourceType::Energy)) as i32;
    let free_capacity = creep.store().get_free_capacity(Some(ResourceType::Energy));

    // Where to take energy from
    let origin = if free_capacity == 0 { None }
    //TODO proximity then spawn, then extension, then construction site, then controller
    else { match room.find(find::RUINS, None).iter().filter(|r|
        r.store().get_used_capacity(Some(ResourceType::Energy)) > 0)
        .min_by_key(|r| pos.get_range_to(r.pos())) {
        Some(r) => Some(TargetByObj::Ruin(r.clone())),
    None => { match pos.find_closest_by_path(find::SOURCES_ACTIVE, None) {
        Some(s) => Some(TargetByObj::Source(s)),
        None => None,
    }}}};

    let vstructures;
    let vstructures_near;
    let vconstructions_near;
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
        None => { vconstructions_near = pos.find_in_range(find::MY_CONSTRUCTION_SITES, 1);
            match vconstructions_near.get(0) {
                Some(v) => Some(TargetByObj::ConstructionSite(v.clone())),
        None => {vstructures = room.find(find::MY_STRUCTURES, None);
            match vstructures.iter().filter_map(|o| match o {
                StructureObject::StructureSpawn(s) if s.store().get_free_capacity(Some(ResourceType::Energy)) > 50 => Some(s),
                _ => None,
            })
            .min_by_key(|s| pos.get_range_to(s.pos()))
            .map(|s| TargetByObj::Spawn(s.clone())) {
                Some(thing) => Some(thing),
        None => match vstructures.iter().filter_map(|o| match o {
                StructureObject::StructureExtension(x) if x.store().get_free_capacity(Some(ResourceType::Energy)) > 50 => Some(x),
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
        },}}}}}}
    
    };

    //if neither full nor empty, make a decision based on range.
    let target = Some(TargetByID::from( match (destination, origin) {
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
