use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::{Rc, Weak}, ops::{Add, AddAssign, Sub, SubAssign}, borrow::Borrow};

use crate::creeps::CreepName;

use super::{JobKey, CreepJob};

mod max_match;
// /**
// (re-)disribute idle creeps among jobs.
// */
// pub fn hire(all_jobs: RefCell< HashMap<JobKey, CreepJob> >, idle_creeps: &mut HashSet<CreepName>) {
//     //Distribute creeps among jobs with no care for the limits.
//     idle_creeps.drain().map(|creep|
//         all_jobs
//     );
// }
