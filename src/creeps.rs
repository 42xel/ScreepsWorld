use crate::prelude::*;

use screeps::{
    objects::Creep, SharedCreepProperties, ErrorCode,
};

use crate::{my_wasm::*, };
use self::drone::run_drone;

pub mod count {
    use super::super::*;

    thread_local! {
        //TODO make it Room local
        //TODO make a struct/trait for several roles with several bodies (>3)
        pub static DRONE: RefCell<u32> = Default::default();
        pub static UNKNOWN: RefCell<u32> = Default::default();

        pub static MAX_DRONE: u32 = 12;
    }
}


#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash,
)]
#[derive(Default)]
pub enum Progress {
    /// The tas is frozen and shouldn't be run normally (it is to be flagged)
    Frozen,
    /// The task has not been started yet.
    /// As a return value, it means some conditions are not met to execute the task yet and the task is to be reran.
    #[default]
    Todo,
    /// The task is in progress.
    Doing,
    /// The task is nearing its completion.
    /// 
    /// In the context of task sequence or cycle, upon a task returning `Progress::Soon`, the next task is tried immediately, like after a `Progress::Done` result.
    /// Unlike after a `Progress::Done` however, if this subsequent task wasn't ready (`Progress::Todo`), the state is backtracked and the task tried again next tick.
    /// 
    /// In the context of Job, going from a state smaller to one greater or equal to `Progress::Soon` triggers the callback.
    /// Unlike `Progress::Done`, `Progress::Soon` doesn't remove the job from the Queue.
    Soon,
    /// The task is done.
    Done,
}
#[allow(dead_code)]
#[derive(
    Debug, PartialEq, Eq, Clone, Hash,
)]
pub(self) enum JobError {
    // /// An auxiliary variant, for homogeneity.
    // NoError,
    /// The creep suppose to accomplish the job can't be found, presumably he is dead
    ErrorCode(ErrorCode),
    NoCreep,
    /// A target the job relies on can't be found, presumably the corresponding structure disapeared
    NoTarget,
    /// A position the job relies on can't be found, presumably the corresponding structure disapeared
    NoPosition,
    ///
    NoJob,
    /// An error which should never occur
    Impossible(String),
}
impl From<ErrorCode> for JobError {
    fn from(value: ErrorCode) -> Self {
        Self::ErrorCode(value)
    }
}

pub(crate) fn run_creep(creep: &Creep) {
    if creep.spawning() {return;}

    let name = creep.name();
    debug!("running creep {}", name);

    //let target = creep_targets.entry(name);
    let role = creep.name().chars().next().expect_js("creep doesn't have a name !".into());

    match role {
        'd'|'H' => run_drone(creep),
        _ => (),
    }
}

mod drone;