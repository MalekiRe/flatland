use std::fs::File;
use std::io::Read;
use winit::event::{ElementState, KeyboardInput, ModifiersState, VirtualKeyCode};
use anyhow::{anyhow, Error, Result};
use kdl::{KdlDocument, KdlNode};
use slog::o;
use crate::input_window::InputWindow;
use crate::key_shortcuts::Modifier::{Alt, Ctrl, Mod, Shift};

pub fn keyboard_shortcuts(window: &mut InputWindow, input: KeyboardInput) {
    if let Some(key_input) = input.virtual_keycode {
        match input.state {
            ElementState::Pressed => {
                window.persistent_key_state.0.push(key_input);
            }
            ElementState::Released => {
                window.persistent_key_state.0.retain(|key| *key != key_input);
            }
        }
    }
    let movement = &window.config_file.1.movement;
    let input_combos = KeyCombo {
        modifiers: convert_modifiers(input.modifiers),
        keys: window.persistent_key_state.0.clone()
    };
    if movement.forward == input_combos {
        println!("IT MATCHES");
    }
    // window.flatland.lock().with_focused(|panel_item| {
    //     panel_item.
    // });
    // if let Some(focused) = window.flatland.lock().focused.clone().upgrade() {
    //     focused.lock().item.with_node(|panel_item| {
    //         panel_item.resize(3000, 3000).unwrap();
    //     });
    // }
}
pub struct PersistentKeyState(pub Vec<VirtualKeyCode>);

pub struct KeyCombo {
    modifiers: Vec<Modifier>,
    keys: Vec<VirtualKeyCode>
}
impl KeyCombo {
    pub fn new(modifiers: Vec<Modifier>, keys: Vec<VirtualKeyCode>) -> KeyCombo{
        Self {
            modifiers,
            keys
        }
    }
    pub fn matches(&self, modifiers: &Vec<Modifier>, keys: &Vec<VirtualKeyCode>) -> bool {
        for modifier in modifiers {
            if !self.modifiers.contains(&modifier) {
                return false;
            }
        }
        for key in keys {
            if !self.keys.contains(&key) {
                return false;
            }
        }
        true
    }
}
impl PartialEq for KeyCombo {
    fn eq(&self, other: &Self) -> bool {
        self.matches(&other.modifiers, &other.keys)
    }
}

#[derive(PartialEq)]
pub enum Modifier {
    Ctrl,
    Alt,
    Mod,
    Shift,
}

pub fn convert_modifiers(modifiers_state: ModifiersState) -> Vec<Modifier> {
    let mut modifiers = vec![];
    modifiers_state.alt().then(|| modifiers.push(Alt));
    modifiers_state.ctrl().then(|| modifiers.push(Ctrl));
    modifiers_state.logo().then(|| modifiers.push(Mod));
    modifiers_state.shift().then(|| modifiers.push(Shift));
    modifiers
}

pub struct KeyboardShortcuts {
    movement: Movement
}
impl KeyboardShortcuts {
    pub fn new(file: &mut File) -> Option<Self>{
        let mut file_str = String::new();
        file.read_to_string(&mut file_str).unwrap();
        let doc: KdlDocument = file_str.parse().unwrap();
        let keyboard_shortcuts = doc.get("KeyboardShortcuts")?.children()?;
        let movement = keyboard_shortcuts.get("Movement")?.children()?;
        let rotation = keyboard_shortcuts.get("Rotation")?.children()?;
        let resize = keyboard_shortcuts.get("Resize")?.children()?;
        Some(KeyboardShortcuts {
            movement: Movement {
                up: parse_key_combo(movement.get("Up")?)?,
                down: parse_key_combo(movement.get("Down")?)?,
                left: parse_key_combo(movement.get("Left")?)?,
                right: parse_key_combo(movement.get("Right")?)?,
                forward: parse_key_combo(movement.get("Forward")?)?,
                backward: parse_key_combo(movement.get("Backward")?)?
            }
        })
    }
}
pub fn parse_key_combo(x: &KdlNode) -> Option<KeyCombo> {
    let mut modifiers = vec![];
    let mut keys = vec![];
    x.entries().iter().map(|entry| entry.value().as_string()).try_for_each(|entry| {
        let entry = entry?;
        match entry {
            "Alt" => { modifiers.push(Alt) }
            "Shift" => { modifiers.push(Shift) }
            "Ctrl" => { modifiers.push(Ctrl) }
            "Mod" => { modifiers.push(Mod) }
            &_ => {
                let virtual_key_code: VirtualKeyCode = serde_json::from_str(&entry).ok()?;
                keys.push(virtual_key_code);
            }
        }
        Some(())
    });
    Some(KeyCombo{
        modifiers,
        keys
    })
}
pub struct Movement {
    up: KeyCombo,
    down: KeyCombo,
    left: KeyCombo,
    right: KeyCombo,
    forward: KeyCombo,
    backward: KeyCombo
}
pub struct Rotation {
    up: KeyCombo,
    down: KeyCombo,
    left: KeyCombo,
    right: KeyCombo,
    clockwise: KeyCombo,
    counter_clockwise: KeyCombo
}
pub struct Resize {
    up: KeyCombo,
    down: KeyCombo,
    left: KeyCombo,
    right: KeyCombo,
}