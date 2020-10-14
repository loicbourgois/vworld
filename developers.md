# Welcome to VWorld - developer's guide

## Useful scripts

```bash
export vworld_root_folder=$HOME/github/vworld
$vworld_root_folder/scripts/check.sh
$vworld_root_folder/scripts/front.sh
$vworld_root_folder/scripts/vworld-build.sh
$vworld_root_folder/scripts/vworld-run-no-docker.sh
configuration_name="demo" $vworld_root_folder/scripts/vworld-server-start.sh
configuration_name="demo" x="0" y="0" $vworld_root_folder/scripts/vworld-server-start-singlechunk.sh
```

## Tasks
- make eye see
  collision detection beetween eyes line of sigh and particles
- refactor luuids and pairs
  there might be some duplication here
- entities and particles count available for client
- create new entities if min_entities_count too low
- add reproduction
- add fitness function based on distance traveled
- allow for dynamic change of parameters from client
  - link strengh
  - drag_coefficient
- move client synchronisation in main loop
  this will avoid blocking between threads
  only initial connection should happen outside of main thread
- performance timers
- rename PLOP
- rename bob
- add proper wall configuration
- test uuid only instead of puuids
  can a uuid be used instead of a puuids ?
  can a u128 be used instead of a uuid ?
  if yes, maybe remove these aliases

# Perf

2020-10-02
-  1000 entities of 1 particle
  - plugged in: 90 steps per second
  - unplugged:  90 steps per second
-  2000 entities of 1 particle
  - plugged in: 42 steps per second
  - unplugged:  33 steps per second
2020-10-03
- 300 entities of 3 particle
  plugged in: 69 steps per second
- 100 entities of 9 particles
  plugged in: 54 steps per second
- 100 entities of 10 particles
  plugged in: 49 steps per second

## Specs

### how do we create new cells ?

we need 3 cells connected in a triangle
if the 3 cells

### how do we handle deletion of item ?

use uuid and hashmap

## Quotes

- Welcome to VWorld.
- It's not a bug, it's a maladaptation to the current universe.
- One too many is too many.
- Value is at the edge.
