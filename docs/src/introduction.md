# Introduction

Herder is a Bevy remake of an older SDL/C++ game written in 2008. The goal is not an exact port. The remake keeps the central idea of herding sheep with a shepherd and dog, while rebuilding the code around Bevy's ECS, schedules, states, resources, and asset loading.

The current version focuses on a playable core loop:

- start a run from the menu
- move the shepherd around the map
- draw dog routes with the mouse
- scare and guide sheep toward the finish tile
- score sheep as they enter the finish
- save a local highscore with a player name

The code is organized around small gameplay systems instead of large object-oriented classes. That makes it easier to extend the map, add obstacles, tune sheep behavior, and evolve the UI without carrying over the old project's structure.
