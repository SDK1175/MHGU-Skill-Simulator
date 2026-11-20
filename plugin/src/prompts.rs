//! Button icons following the Recomp button layout (mhgurecomp.ini [Actions.Pad]). The button_prompts approach of Wishlist (our mod).
//! Input is read from Recomp menu words (with the key layout applied, input.rs), so only the icons need to change.
//! But +/- and directions are read from the raw input, not menu words, so their icons stay.
//! Keyboard / mouse prompts: while the last input came from the keyboard or mouse (input::kbm_active) and Recomp's
//! [Controls] KeyboardPrompts is on, a button shows its key (atlas picture "kbm-<key>"). Our window reads menu input, so
//! the keys are Recomp's menu actions ([Actions.Kbm], game settings > Keybinds > Menus) when actions manage menus,
//! else the plain [Keybinds] rows.
use crate::app::Button;
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Prompt {
    /// Unchanged
    Original,
    /// Use another button's icon
    Button(Button),
    /// Input without an icon (L3, combos, ...): show text
    Text(String),
    /// A keyboard key or mouse button, by its [Keybinds] name ("Space", "Mouse Left")
    Key(String),
}

/// Recomp menu action name -> our button (only for input read from menu words)
const ACTIONS: [(&str, Button); 8] = [
    ("MenuConfirm", Button::A),
    ("MenuCancel", Button::B),
    ("MenuX", Button::X),
    ("MenuY", Button::Y),
    ("MenuTabL", Button::L),
    ("MenuTabR", Button::R),
    ("MenuPageL", Button::ZL),
    ("MenuPageR", Button::ZR),
];

/// Recomp menu actions with a keyboard binding ([Actions.Kbm]) -> our button
const KBM_ACTIONS: [(&str, Button); 14] = [
    ("MenuConfirm", Button::A),
    ("MenuCancel", Button::B),
    ("MenuX", Button::X),
    ("MenuY", Button::Y),
    ("MenuTabL", Button::L),
    ("MenuTabR", Button::R),
    ("MenuPageL", Button::ZL),
    ("MenuPageR", Button::ZR),
    ("MenuPlus", Button::Plus),
    ("MenuMinus", Button::Minus),
    ("MenuUp", Button::Up),
    ("MenuDown", Button::Down),
    ("MenuLeft", Button::Left),
    ("MenuRight", Button::Right),
];

/// [Keybinds] row of each button
const ROWS: [(&str, Button); 14] = [
    ("A", Button::A),
    ("B", Button::B),
    ("X", Button::X),
    ("Y", Button::Y),
    ("L", Button::L),
    ("R", Button::R),
    ("ZL", Button::ZL),
    ("ZR", Button::ZR),
    ("Plus", Button::Plus),
    ("Minus", Button::Minus),
    ("DpadUp", Button::Up),
    ("DpadDown", Button::Down),
    ("DpadLeft", Button::Left),
    ("DpadRight", Button::Right),
];

pub struct Config {
    /// Controller icons per menu action (ACTIONS order)
    pub pad: [Prompt; 8],
    /// Keys bound to each button (ROWS order), as the file writes them
    pub keys: [Vec<String>; 14],
    /// [Controls] KeyboardPrompts (on when missing)
    pub kbm: bool,
    /// Recomp actions manage the game menus ([Actions] Enable and ManageMenus)
    pub menus: bool,
    /// [Actions.Kbm] menu actions the player changed (KBM_ACTIONS order); None = default
    pub menu_keys: [Option<Vec<String>>; 14],
}

pub fn parse(ini: &str) -> Config {
    let mut out: [Prompt; 8] = core::array::from_fn(|_| Prompt::Original);
    let mut keys: [Vec<String>; 14] = core::array::from_fn(|_| Vec::new());
    let mut kbm = true;
    let mut menu_keys: [Option<Vec<String>>; 14] = core::array::from_fn(|_| None);
    let mut section = "";
    let (mut enabled, mut menus) = (true, true);
    for line in ini.lines() {
        let line = line.split(';').next().unwrap_or("").trim();
        if line.starts_with('[') && line.ends_with(']') {
            section = &line[1..line.len() - 1];
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        let (key, value) = (key.trim(), value.trim());
        let on = !(value.eq_ignore_ascii_case("off") || value.eq_ignore_ascii_case("false") || value == "0");
        if section.eq_ignore_ascii_case("Actions") {
            if key.eq_ignore_ascii_case("Enable") {
                enabled = on;
            }
            if key.eq_ignore_ascii_case("ManageMenus") {
                menus = on;
            }
        }
        if section.eq_ignore_ascii_case("Controls") && key.eq_ignore_ascii_case("KeyboardPrompts") {
            kbm = on;
        }
        if section.eq_ignore_ascii_case("Keybinds") {
            if let Some(i) = ROWS.iter().position(|(name, _)| key.eq_ignore_ascii_case(name)) {
                keys[i] = value
                    .split(',')
                    .map(str::trim)
                    .filter(|k| !k.is_empty() && !k.eq_ignore_ascii_case("none"))
                    .map(String::from)
                    .collect();
            }
            continue;
        }
        if section.eq_ignore_ascii_case("Actions.Kbm") {
            if let Some(i) = KBM_ACTIONS.iter().position(|(name, _)| {
                key.eq_ignore_ascii_case(name)
                    || (*name == "MenuConfirm" && key.eq_ignore_ascii_case("Confirm"))
                    || (*name == "MenuCancel" && key.eq_ignore_ascii_case("Cancel"))
            }) {
                if !(value.is_empty() || value.eq_ignore_ascii_case("default") || value.eq_ignore_ascii_case("original")) {
                    menu_keys[i] = Some(action_keys(value));
                }
            }
            continue;
        }
        if !section.eq_ignore_ascii_case("Actions.Pad") {
            continue;
        }
        let Some(i) = ACTIONS.iter().position(|(name, _)| {
            key.eq_ignore_ascii_case(name)
                || (*name == "MenuConfirm" && key.eq_ignore_ascii_case("Confirm"))
                || (*name == "MenuCancel" && key.eq_ignore_ascii_case("Cancel"))
        }) else {
            continue;
        };
        out[i] = binding(value);
    }
    if !enabled || !menus {
        out = core::array::from_fn(|_| Prompt::Original);
    }
    Config { pad: out, keys, kbm, menus: enabled && menus, menu_keys }
}

/// Inputs of an [Actions.Kbm] line ("key:Space | mouse:Left") as [Keybinds]-style names ("Space", "Mouse Left").
/// Combos and pad inputs are skipped.
fn action_keys(value: &str) -> Vec<String> {
    value
        .split('|')
        .map(str::trim)
        .filter(|v| !v.contains('+') || v.ends_with('+'))
        .filter_map(|v| {
            let lower = v.to_ascii_lowercase();
            if lower.starts_with("key:") {
                Some(String::from(v[4..].trim()))
            } else if lower.starts_with("mouse:") {
                Some(alloc::format!("Mouse {}", v[6..].trim()))
            } else {
                None
            }
        })
        .collect()
}

/// Keys of a button in Recomp's menus: the changed menu action, else the menu default (confirm = left click,
/// cancel = right click, X = Space, the rest as [Keybinds]); without menu actions, the [Keybinds] row
fn menu_keys(c: &Config, b: Button) -> Vec<String> {
    let row = |b: Button| ROWS.iter().position(|(_, x)| *x == b).map_or(Vec::new(), |i| c.keys[i].clone());
    if !c.menus {
        return row(b);
    }
    if let Some(Some(keys)) = KBM_ACTIONS.iter().position(|(_, x)| *x == b).map(|i| &c.menu_keys[i]) {
        return keys.clone();
    }
    match b {
        Button::A => alloc::vec![String::from("Mouse Left")],
        Button::B => alloc::vec![String::from("Mouse Right")],
        Button::X => alloc::vec![String::from("Space")],
        _ => row(b),
    }
}

fn binding(value: &str) -> Prompt {
    if value.is_empty() || value.eq_ignore_ascii_case("default") || value.eq_ignore_ascii_case("original") {
        return Prompt::Original;
    }
    if value.eq_ignore_ascii_case("none") {
        return Prompt::Text("—".into());
    }
    // With several inputs ('|') show the first. Combos ('+') as text.
    let first = value.split('|').next().unwrap_or("").trim();
    let single = match first.to_ascii_lowercase().as_str() {
        // Names by the Switch layout: bottom B, right A, top X, left Y (same as Wishlist)
        "pad:south" => Some(Button::B),
        "pad:east" => Some(Button::A),
        "pad:north" => Some(Button::X),
        "pad:west" => Some(Button::Y),
        "pad:ltrigger" | "pad:lefttrigger" | "pad:lt" => Some(Button::ZL),
        "pad:rtrigger" | "pad:righttrigger" | "pad:rt" => Some(Button::ZR),
        "pad:lshoulder" | "pad:leftshoulder" => Some(Button::L),
        "pad:rshoulder" | "pad:rightshoulder" => Some(Button::R),
        "pad:start" => Some(Button::Plus),
        "pad:back" => Some(Button::Minus),
        _ => None,
    };
    if let Some(b) = single {
        return Prompt::Button(b);
    }
    let mut label = String::new();
    for part in first.split('+') {
        if !label.is_empty() {
            label.push('+');
        }
        let lower = part.trim().to_ascii_lowercase();
        let name = match lower.as_str() {
            "pad:south" => "B",
            "pad:east" => "A",
            "pad:north" => "X",
            "pad:west" => "Y",
            "pad:lshoulder" | "pad:leftshoulder" => "L",
            "pad:rshoulder" | "pad:rightshoulder" => "R",
            "pad:ltrigger" | "pad:lefttrigger" | "pad:lt" => "ZL",
            "pad:rtrigger" | "pad:righttrigger" | "pad:rt" => "ZR",
            "pad:lstick" => "L3",
            "pad:rstick" => "R3",
            "pad:dpleft" => "←",
            "pad:dpright" => "→",
            "pad:dpup" => "↑",
            "pad:dpdown" => "↓",
            "pad:start" => "+",
            "pad:back" => "−",
            _ => {
                let t = part.trim();
                t.strip_prefix("pad:").or_else(|| t.strip_prefix("sw:")).unwrap_or(t)
            }
        };
        label.push_str(name);
    }
    Prompt::Text(label.to_string())
}

/// Mouse buttons and wheel notches bound to a button do not work in our window (its clicks are ours: left = A, right = B)
fn is_mouse(key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    k.starts_with("mouse") || k.starts_with("wheel")
}

fn pad_prompt(c: &Config, b: Button) -> Prompt {
    ACTIONS.iter().position(|(_, x)| *x == b).map_or(Prompt::Original, |i| c.pad[i].clone())
}

/// The prompt of a button for a keyboard / mouse player
fn kbm_prompt(c: &Config, b: Button) -> Prompt {
    match b {
        Button::A => return Prompt::Key("Mouse Left".into()),
        Button::B => return Prompt::Key("Mouse Right".into()),
        _ => {}
    }
    match menu_keys(c, b).iter().find(|k| !is_mouse(k)) {
        Some(k) => Prompt::Key(k.clone()),
        // Only a mouse binding: unusable in our window, keep the controller icon (the prompt itself is clickable)
        None => pad_prompt(c, b),
    }
}

/// Atlas picture name of a key: "kbm-" + the name in lower case, spaces as '-', punctuation as words
/// ("Left Shift" -> "kbm-left-shift", "-" -> "kbm-minus"). Pictures: tools/import_kenney_icons.py
pub fn key_sprite(key: &str) -> String {
    let key = key.trim();
    let word = match key {
        "-" => Some("minus"),
        "=" => Some("equals"),
        "[" => Some("bracket-open"),
        "]" => Some("bracket-close"),
        ";" => Some("semicolon"),
        "'" => Some("apostrophe"),
        "," => Some("comma"),
        "." => Some("period"),
        "/" => Some("slash"),
        "\\" => Some("backslash"),
        "`" => Some("grave"),
        "Keypad +" => Some("keypad-plus"),
        _ => None,
    };
    let mut s = String::from("kbm-");
    match word {
        Some(w) => s.push_str(w),
        None => {
            for ch in key.chars() {
                s.push(if ch == ' ' { '-' } else { ch.to_ascii_lowercase() });
            }
        }
    }
    s
}

/// Short text for a key without a picture (about 3 letters at most)
pub fn key_label(key: &str) -> String {
    let lower = key.trim().to_ascii_lowercase();
    let short = match lower.as_str() {
        "up" => "↑",
        "down" => "↓",
        "left" => "←",
        "right" => "→",
        "space" => "SPC",
        "return" | "enter" | "keypad enter" => "ENT",
        "backspace" => "BS",
        "escape" => "ESC",
        "left shift" | "right shift" => "SFT",
        "left ctrl" | "right ctrl" => "CTL",
        "left alt" | "right alt" => "ALT",
        "page up" => "PGU",
        "page down" => "PGD",
        "delete" => "DEL",
        "insert" => "INS",
        "mouse left" => "LMB",
        "mouse right" => "RMB",
        "mouse middle" => "MMB",
        _ => {
            let t = key.trim();
            let t: String = t.strip_prefix("Keypad ").unwrap_or(t).chars().filter(|c| *c != ' ').collect();
            return t.to_uppercase().chars().take(3).collect();
        }
    };
    short.to_string()
}

#[cfg(target_arch = "arm")]
mod runtime {
    use super::*;
    use forge::sys::imgui as sys;
    static mut PROMPTS: Option<Config> = None;
    static mut WAIT: u64 = 0;
    /// Every frame (game thread): on Recomp only, re-read the config file every second
    pub fn refresh() {
        if !crate::input::is_recomp() {
            return;
        }
        unsafe {
            let now = crate::clock::ticks();
            if now < WAIT {
                return;
            }
            WAIT = now + 1000 * crate::clock::PER_MS;
            // Open PC paths only on Recomp (never send PC paths to nn::fs on hardware/emulators)
            let file = sys::igImFileOpen(c"mhgurecomp.ini".as_ptr(), c"rb".as_ptr());
            if file.is_null() {
                return;
            }
            let size = sys::igImFileGetSize(file);
            if size > 128 * 1024 {
                sys::igImFileClose(file);
                return;
            }
            let mut bytes = alloc::vec![0u8; size as usize];
            let n = sys::igImFileRead(bytes.as_mut_ptr().cast(), 1, size, file);
            sys::igImFileClose(file);
            if n == size {
                if let Ok(text) = core::str::from_utf8(&bytes) {
                    PROMPTS = Some(parse(text));
                }
            }
        }
    }
    fn config() -> Option<&'static Config> {
        unsafe { (*core::ptr::addr_of!(PROMPTS)).as_ref() }
    }
    /// Keyboard / mouse prompts are shown now
    pub fn kbm() -> bool {
        crate::input::kbm_active() && config().is_some_and(|c| c.kbm)
    }
    pub fn get(b: Button) -> Prompt {
        let Some(c) = config() else { return Prompt::Original };
        if kbm() { kbm_prompt(c, b) } else { pad_prompt(c, b) }
    }
}
#[cfg(target_arch = "arm")]
pub use runtime::{get, kbm, refresh};
#[cfg(not(target_arch = "arm"))]
pub fn refresh() {}
#[cfg(not(target_arch = "arm"))]
pub fn get(_: Button) -> Prompt {
    Prompt::Original
}
#[cfg(not(target_arch = "arm"))]
pub fn kbm() -> bool {
    false
}
