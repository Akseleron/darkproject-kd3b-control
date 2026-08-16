# KD3B rev.2 factory layer and onboard profiles

Status: **partially evidenced, exact KD3B rev.2 mapping not yet recovered**.

This document exists to keep host-side profiles in KD3B Control separate from the keyboard's own factory/default layer, Fn layer, macros and onboard memory.

## Confirmed project boundary

The project currently has a validated volatile Direct RGB transport for interface 2. It does **not** yet have a validated read/write protocol for onboard key remapping, Fn assignments, macros or persistent profiles. Those features must not be inferred from the Direct RGB packet format.

## External evidence

Dark Project's software tutorial for this generation explicitly distinguishes:

- a Default Layer;
- an Fn1 Layer;
- key remapping;
- multimedia assignments through Fn1;
- macros;
- backup/configuration operations.

Reference: https://rutube.ru/video/30270860f9a48c98ed2fb3059d488f8d/

Secondary guides for the KD1/KD3 rev.2 family also describe multiple keyboard profiles selected through Fn combinations. One such guide reports `Fn + F1` through `Fn + F4` as profile switching for that family:

https://preg.aker.cx.ua/articles/dark-project-klaviatura-uvimknuti-pidsvichuvannja.html

However, generic Dark Project manuals for newer/different models assign media functions to `Fn + F1..F12`, so those mappings are **not interchangeable across models**. Therefore KD3B Control must not hardcode `Fn + F1..F4` as an exact KD3B rev.2 fact until a model-specific manual, OEM configuration dump, passive input observation or USB capture confirms it.

## Phase 4 UI consequence

KD3B Control exposes two clearly separate concepts:

1. **Host profiles**: application-owned lighting state containing software effect parameters or a complete 87-key Direct RGB frame. These are safe to save/load without touching onboard memory.
2. **Factory / onboard state**: Default Layer, Fn layer, macros, remaps and persistent profiles. Phase 4 displays this as a read-only capability boundary and does not write it.

## Required evidence before onboard editing

At least one of the following is required before implementing onboard profile/Fn writes:

- exact KD3B rev.2 OEM manual with the relevant mappings;
- exact OEM application configuration export for this keyboard;
- controlled USB captures of one isolated remap/profile/Fn change at a time;
- a validated readback command that can establish the current onboard state without modifying it.

Until then, factory assignments remain a Phase 6 research task rather than a Phase 4 write path.
