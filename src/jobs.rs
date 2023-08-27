//! Bee can choose flowers, or flowers can choose bee.
//! The later is less intuitive and straigtforward to implements, but more efficient.
//! Indeed, it prevents collisions and enables finer relation between needs and means.
//! Jobs are flower, this module implements this paradigm.

use std::{default, collections::{HashSet, HashMap, hash_map}, cell::RefCell, fmt::Debug, borrow::BorrowMut, any::Any, cmp::min};

use screeps::{ObjectId, Source, Ruin, StructureExtension, StructureSpawn, ConstructionSite, StructureController, Creep, Room, find, RoomName, RawObjectId, game, RoomObjectProperties, HasNativeId, HasId, SharedCreepProperties, MaybeHasPosition, HasPosition};

use crate::creeps::{self, CreepName, cost,};

type Credit = i32;

// // Todo: add u32 data representing how long or up until when the status is valid ? Maybe not in the enum
// #[derive(Debug, Default)]
// pub enum Status {
//     #[default]
//     Open,
//     SoftLocked,
//     Locked,
// }

/// this enum will represent a drone's lock on a specific target object, storing a js reference
/// to the object id so that we can grab a fresh reference to the object each successive tick,
/// since screeps game objects become 'stale' and shouldn't be used beyond the tick they were fetched
#[derive(Debug, Default, PartialEq, PartialOrd)]
pub enum TargetEnum {
    Source(ObjectId<Source>),
    Ruin(ObjectId<Ruin>),
    Extension(ObjectId<StructureExtension>),
    SupplySpawn(target::SupplySpawn),
    Spawn(ObjectId<StructureSpawn>),
    _ConstructionSiteMax,
    ConstructionSite(ObjectId<ConstructionSite>),
    Controller(ObjectId<StructureController>),
    #[default]
    _None,
}


#[derive(Debug, PartialEq, Eq, Hash)]
pub enum JobKey {
    RawObjectId(RawObjectId),
}
impl From<RawObjectId> for JobKey {
    fn from(value: RawObjectId) -> Self {
        Self::RawObjectId(value)
    }
}
pub trait IntoJobKey<Marker: IntoJobKeyMarker> {
    fn into(self) -> JobKey;
}
impl JobKey {
    fn from<T: IntoJobKey<Marker>, Marker: IntoJobKeyMarker>(value: T) -> JobKey{
        value.into()
    }
}
pub trait IntoJobKeyMarker{}
struct MarkerFromRawId{} impl IntoJobKeyMarker for MarkerFromRawId{}
impl IntoJobKey<MarkerFromRawId> for RawObjectId {
    fn into(self) -> JobKey {
        JobKey::RawObjectId(self)
    }
}
struct MarkerFromHasId{} impl IntoJobKeyMarker for MarkerFromHasId{}
impl<T: HasId> IntoJobKey<MarkerFromHasId> for T {
    fn into(self) -> JobKey {
        JobKey::RawObjectId(self.raw_id())
    }
}

#[derive(Debug)]
pub enum CreepJobError {
    NotEnough,
    NotHere,
    //AlreadyExists(),
    Other(String),
}

thread_local! {
    static ALL_JOBS: RefCell< HashMap<JobKey, CreepJob> > = { let all_jobs = RefCell::new(HashMap::new());
        all_jobs.borrow_mut().extend(game::spawns().values().into_iter().map(|spawn| {
            let job = CreepJob::from::<supply_spawn::Target, MarkerFromHasId>(&spawn);
            (JobKey::from(spawn), job)
        }));
        all_jobs
    };

//    static JOBS_BY_CREEPS: RefCell< HashMap<CreepName, JobKey> > = Default::default();
}

//TODO inspect code error when removing static and the necessity of a lifetime parameter error[E0310]
pub trait Target: Debug + MaybeHasPosition + 'static
{
    /// the number of creeps who can concurrently work a given job.
    fn capacity(&self) -> usize;

    /// executes the job for the given creep.
    /// returns whether the job came to an end.
    fn execute(&mut self, creep: &Creep) -> bool;

    /// 
    fn offer(&self, creep: &Creep) -> Credit;
    // ///
    //fn train(&self, u32: energy_limit) -> CreepPrototype {
    //    
    //}
}
pub trait TargetAux<Marker: IntoJobKeyMarker>: Target + PartialEq + PartialOrd{
    /// The type of the world object to create the target from.
    type Object: Debug + IntoJobKey<Marker>;
    fn new(obj: &Self::Object) -> Self;
}

#[derive(Debug)]
pub struct CreepJob
{
//    status: Status,
    pub creeps: Vec<CreepName>,
    pub target: Box<dyn Target>,
}

impl CreepJob {
    /**
    Creates a new job from a type `T`.
        */
    fn from<T: TargetAux<Marker>, Marker: IntoJobKeyMarker>
    (value: &T::Object) -> Self
    {
        let t: T = TargetAux::new(value);
        Self {
            creeps: Vec::with_capacity(min(32, t.capacity())),
            target: {
                Box::new(t)
            },
            //status: Default::default(),
        }
    }
    /**
    Creates a new job from a type `T` and keep tracks of it
        */
    fn new<T: TargetAux<Marker>, Marker: IntoJobKeyMarker>
    (value: T::Object, all_jobs: &mut HashMap<JobKey, CreepJob>)
    -> Result<&mut CreepJob, hash_map::OccupiedError<'_, JobKey, CreepJob>>
    {
        let r = Self::from::<T, Marker>(&value);
        all_jobs.try_insert(JobKey::from(value), r)
    }

    // pub fn try_assign(&mut self, creep: &Creep, idle_creeps: &mut HashSet<CreepName>) -> Result<(), CreepJobError> {
    //     if self.creeps.len() < self.target.capacity() {
    //         let name = creep.name();
    //         idle_creeps.remove(&name);
    //         self.creeps.push(name);
    //         Ok(())
    //     } else {
    //         Err(CreepJobError::NotEnough)
    //     }
    // }

    // pub fn try_free(&mut self, creep: &Creep, idle_creeps: &mut HashSet<CreepName>) -> Result<(), CreepJobError> {
    //     let name = creep.name();
    //     match self.creeps.extract_if(|c| *c == name).count() {
    //         1 => {
    //             idle_creeps.insert(name);
    //             Ok(())
    //         },
    //         0 => Err(CreepJobError::NotHere),
    //         _ => {
    //             idle_creeps.insert(name);
    //             Err(CreepJobError::Other("duplicate name in job creep list.".to_owned()))
    //         }
    //     }
    // }
    
    fn relocation_cost(&self, creep: &Creep) -> Credit {
        let Some(pos) = self.target.try_pos() else { return 0; };
        (pos.get_range_to(creep.pos()) * cost(creep)) as Credit
    }

   // quit
   // fire
   // look/search applicants
   // 
}

pub mod hiring;

pub(super) mod supply_spawn {
    use std::{usize, cmp};

    use log::warn;
    use screeps::{ObjectId, StructureSpawn, HasTypedId, Structure, RawObjectId, ResourceType, SharedCreepProperties, ErrorCode, Creep, game, MaybeHasPosition, Position, HasPosition};
    use wasm_bindgen::throw_str;
    use crate::{creeps::{move_creep_to, error_no_body_part}, my_wasm::UnwrapJsExt};

    use super::{JobKey, MarkerFromHasId, Credit, CreepJob};

    #[derive(Debug, PartialEq, PartialOrd)]
    pub(super) struct Target{
        spawn: ObjectId<StructureSpawn>,
    }
    impl Target {
        fn store_size(&self) -> u32 {
            let Some(spawn) = self.spawn.resolve() else { return 300; };
            spawn.store().get_capacity(Some(ResourceType::Energy))
        }
        fn carry_size(&self) -> u32 {
            let Some(spawn) = self.spawn.resolve() else { return 50; };
            cmp::min(spawn.room().unwrap_js().energy_capacity_available() / 4, self.store_size())
        }
    }
    impl MaybeHasPosition for Target {
        fn try_pos(&self) -> Option<Position> {
            self.spawn.resolve().and_then(|s| Some(s.pos()))
        }
    }
    impl super::Target for Target {
        fn capacity(&self) -> usize { self.store_size().div_ceil(self.carry_size()) as usize }

        fn execute(&mut self, creep: &screeps::Creep) -> bool {
            let Some(spawn) = self.spawn.resolve() else {
                warn!("jobs::supply_spawn::Target : spawn resolution failed");
                return true;
            };
            if let Err(e) = creep.transfer(&spawn, ResourceType::Energy, None) { match e {
                ErrorCode::NotInRange => match move_creep_to(creep, spawn) {
                    Ok(_) => false,
                    Err(e_move) => {warn!("jobs::supply_spawn::Target : {e_move:?}");
                        true
                    },
                },
                ErrorCode::Full | ErrorCode::NotEnough => true,
                //TODO recycle instead of suiciding
                ErrorCode::NoBodypart => {
                    error_no_body_part(creep);
                    true
                },
                #[allow(unreachable_patterns)]
                _ | ErrorCode::NotOwner | ErrorCode::InvalidTarget | ErrorCode::NotEnough | ErrorCode::Busy | ErrorCode::InvalidArgs => {
                    throw_str(&format!("{:?}", e)) },
            }}
            else {
                true
            }
//        }
        }

        fn offer(&self, creep: &Creep) -> Credit {
            let Some(spawn) = self.spawn.resolve() else {return 0; };
            let free_capacity: i32 = spawn.store().get_free_capacity(Some(ResourceType::Energy))
                / self.capacity() as i32;
            2 * 50 * cmp::min(creep.store().get_used_capacity(Some(ResourceType::Energy)) as i32, free_capacity) as Credit
        }
    }
    impl<'a> super::TargetAux<MarkerFromHasId> for Target {
        type Object = StructureSpawn;

        fn new(spawn: &Self::Object) -> Self {
            Target {
                spawn: spawn.id(),
            }
        }
    }
}


// pub(super) mod source {
//     use screeps::{ObjectId, Source, HasTypedId};

//     use super::JobKey;

//     #[derive(Debug)]
//     pub(super) struct Target{
//         source: ObjectId<Source>,
//     }
//     impl super::Target for Target {
//         fn into(&self) -> JobKey {
//             JobKey::RawObjectId(self.source.into())
//         }
//     }
//     impl From<Source> for Target {
//         fn from(value: Source) -> Self {
//             Self { source: value.id() }
//         }
//     }
//     impl super::TargetNewJob<Source> for Target {}
// }

pub(super) mod target {
    pub(super) use super::supply_spawn::Target as SupplySpawn;
    //pub(super) use super::source::Target as Source;
}


// /// Generate the baseline jobs for each owned room.
// pub fn init() {
//     ALL_JOBS.with_borrow_mut(|all_jobs| {
//         // for spawn in game::spawns().values() {
//         //     all_jobs.entry(JobKey::from(&spawn))
//         //     .or_insert(Box::new(target::Spawn::new_job(spawn))) ;
//         // }

//         // for source in game::rooms().values()
//         // .map(|room| room.find(find::SOURCES, None)).flatten()
//         // {
//         //     all_jobs.entry(JobKey::from(&source))
//         //     .or_insert(Box::new(target::Source::new_job(source))) ;
//         // }
//     });
//     todo!()
// }
