#!/usr/bin/bash
cd assets
aseprite --data "basic_bomb.json" -b --all-layers --split-layers --merge-duplicates --ignore-layer "Normalmap" --list-layers --list-tags --filename-format "{tag}-{layer}-{frame}" --list-slices --format json-array --sheet "basic_bomb.png" --ignore-empty basic_bomb.aseprite
