# Dictation Setup NUX

Dictation setup tests keep first-run microphone readiness, local model setup, and safe prompt actions explicit.

## Settings entry opens setup

The Settings built-in must expose a first-class Dictation Setup command so users can repair readiness without starting capture.

## Pure readiness model

The setup readiness model must keep microphone readiness required while treating hotkey configuration as optional guidance.

## Non-prompting microphone preflight

Opening setup must inspect microphone authorization status without triggering a hidden permission prompt.

## Missing readiness opens setup

Dictation start paths must open setup instead of starting capture when microphone readiness is missing.

## Safe download prompt actions

Model download prompt actions must keep repeated Enter safe and make background download behavior clear.

## Protocol Enter submits setup prompt

Automation Enter on the setup MiniPrompt must use the same submit helper as the real prompt key path.

## Hotkey guidance does not invent default

Ready-state copy must report the actual configured hotkey or launcher guidance without inventing a default shortcut.
