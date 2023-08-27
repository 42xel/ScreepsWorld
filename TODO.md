# Jobs as flowers (creeps as bees)
## DONE
## TODO
- Generating jobs.
    - Generating baseline jobs.
        - serving spawns and extensions
        - saturating own energy sources
        - minerals mining
        - repairs
        - ruins
    - generating new jobs for idle workers
        - remote mining.
        - controller upgrade.
        - recycle self.
- assigning job to suitable nearby creeps.
- assigning job to creep when they ask for one.
- suggesting creep bodies to spawn.

- asking for creeps for urgent jobs.
- MEM: saving/loading jobs from JS mem.
- scheduling

# Refactor
- validate/clean:
    - have ways to load or lazily memoize valid JS object for a tick, and to clean them before the end of a tick.
- resource lock, ownership wrapper (see dedicated sections)

# Creep
- refine partial store affectaion. currently I suppose creep is between origin and destination.

- renew

- atomisiing tasks ?

# Auto building
- controller upgrade
- walls
- tower
- roads
- mining district (range 1 road, range 2 container + extensions )

# Military
Rangers

# Expansion
- Auto Claimant
- room by room automation

# CLI
- hello CLI

# infrastructure
- Containers
- roads
- walls
- Repairs

- train/rails

# Spawn
- computing/hard code the drone variant depending on energy available
- computing/hard code the drone variant accounting for containers 

- computing the drone variant accounting for road (make an algo similar to roads ?)

# resource lock

# Ownership wrapper
add wrapp around JSobjects to have ownership and memoization
- usage:
    - memoize the objects rather than build them every time
        - Not sure it really matters. Beyond the tradeoff of creating a level of indirection, optimizations with he same benefit could be/have been done in the screeps crate, without changing the API. Indeed, such objects have interfaces consisting entirely of non mutable methods, so they may very well be shells containg raw pointers to data preloaded by JS (of which there is plenty, even room.find has some memoization/preloading going under the hood). Todo : look at the code and check for such optimization.
    - add ownership to such objects.
        - may be used to make creeps owned by their job.
        - may be used to commit an exclusive ressource for a tick (a creep body part, the spot around a source, the filling of an extension...).
    - lighten syntax for persistent memory
        - sync trait or custom thread_local macro
        - Deref and AsRef\<T\>, to store name, id, or id_and_name accross ticks and seemlessly obtain the JSobject from them.
    - wrapp memory serialization.
- How?
    - an Rc\<RefCell\<Data\>\> owned by the main loop. Data is essentially a table/HasMap. At the end of the main loop (id est each tick), that Rc and its content is dropped, thus JSobjects are not carried as stale for the next tick.
    - A static weak reference to Data, for ease of use, thus not having to pass a ref to data in every single function.
    - Various getters going through the static weak ref to the main owned data variable, getting JSobjects, which are lazily memoized into data, and owned by it.
    The most commonly useful value returned by getters would be smart pointers:
        - acting as immutable reference
        - persistent (structure holding name/id and going through data to Deref)
        - non persistent immutable ref directly to JSobjects (but with unicity guaranty on this JSobject)
    - custom allocator??
    - unsafe method in a safe wrapper??
