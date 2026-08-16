# KD3B rev.2 factory layer and onboard profiles

Status: **strong model-family evidence exists for the factory shortcuts and three onboard profile slots; the USB persistence/readback protocol is still unknown**.

This document keeps host-side profiles in KD3B Control separate from the keyboard's own factory/default layer, Fn layer, macros and onboard memory.

## Confirmed project boundary

The project currently has a validated volatile Direct RGB transport for interface 2. It does **not** yet have a validated read/write protocol for onboard key remapping, Fn assignments, macros or persistent profiles. Those features must not be inferred from the Direct RGB packet format.

## KD3 family shortcut evidence

A detailed contemporary review covers the KD1A/KD1B/KD3A/KD3B family and states that the A/B variants differ by keycaps, while the KD3 models share the 87-key platform and software functionality. It documents the following on-keyboard groups:

- `Fn + F1..F4`: sound control;
- `Fn + F5..F9`: lighting-mode controls;
- `Fn + F10..F12`: switch between three profiles;
- additional Fn functions include brightness, Win-key locking and effect-speed adjustment.

The same source states that the OEM application's offline mode saves configuration into one of **three profiles in keyboard memory**.

Reference: https://4frag.ru/obzory_klaviatur/obzor-mehanicheskih-klaviatur-dark-project-kd1b-i-kd3a-pervyy-rassvet-297.html

This is strong KD3-family evidence and is materially more specific than generic Dark Project manuals. It still does not expose the USB packet format used to read or write those onboard profiles.

## OEM software-layer evidence

Dark Project's software tutorial for this software generation separately shows:

- a Default Layer;
- an Fn1 Layer;
- key remapping;
- multimedia assignments through Fn1;
- macros;
- backup/configuration operations;
- App mode versus Onboard Editor concepts.

Reference: https://rutube.ru/video/30270860f9a48c98ed2fb3059d488f8d/

## Conflicting secondary material

Some generic/republished Dark Project shortcut guides assign different actions to `Fn + F1..F12`. Those documents cover other models or mixed generations and must not override the KD3-family evidence above. This is why KD3B Control records provenance instead of silently treating every Dark Project shortcut table as interchangeable.

## Phase 4 UI consequence

KD3B Control exposes two clearly separate concepts:

1. **Host profiles**: application-owned lighting state containing software effect parameters or a complete 87-key Direct RGB frame. These are safe to save/load without touching onboard memory.
2. **Factory / onboard state**: three KD3-family profile slots, Default Layer, Fn layer, macros, remaps and persistent settings. Phase 4 may display these as read-only documented capabilities, but it does not write them.

The UI should therefore show the three factory profile slots and Fn shortcut groups so they are not lost from the product model, while clearly marking them as onboard/read-only until the persistence protocol is recovered.

## Required evidence before onboard editing

At least one of the following is required before implementing onboard profile/Fn writes:

- exact KD3/KD3B OEM configuration export or backup containing the three profile slots;
- controlled USB captures of one isolated profile, Fn assignment or remap change at a time;
- a validated readback command that can establish current onboard state without modifying it;
- model-specific protocol documentation.

Until then, onboard profile/Fn **editing** remains a Phase 6 research task. Their existence and user-facing role, however, are now part of the Phase 4 device model and UI.
