use serde::{Deserialize, Serialize};

use crate::state::{AppState, ProgressionId, Theme};

#[allow(dead_code)]
const KEY_THEME: &str = "cof_theme";
#[allow(dead_code)]
const KEY_MUTED: &str = "cof_muted";
#[allow(dead_code)]
const KEY_FAVORITES: &str = "cof_favorites";
#[allow(dead_code)]
const KEY_METRONOME_ACTIVE: &str = "cof_metronome_active";

/// The subset of `AppState` that is persisted to localStorage.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersistedState {
    pub theme: Theme,
    pub muted: bool,
    pub favorites: Vec<ProgressionId>,
    pub metronome_active: bool,
}

impl Default for PersistedState {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            muted: false,
            favorites: Vec::new(),
            metronome_active: false,
        }
    }
}

// --- Pure (de)serialization helpers — testable without WASM ---

pub fn serialize_theme(theme: Theme) -> String {
    match theme {
        Theme::Dark => "dark".to_string(),
        Theme::Light => "light".to_string(),
    }
}

pub fn deserialize_theme(s: &str) -> Theme {
    match s {
        "light" => Theme::Light,
        _ => Theme::Dark,
    }
}

pub fn serialize_muted(muted: bool) -> String {
    if muted { "true".to_string() } else { "false".to_string() }
}

pub fn deserialize_muted(s: &str) -> bool {
    s == "true"
}

pub fn serialize_favorites(favorites: &[ProgressionId]) -> String {
    serde_json::to_string(favorites).unwrap_or_else(|_| "[]".to_string())
}

pub fn deserialize_favorites(s: &str) -> Vec<ProgressionId> {
    serde_json::from_str(s).unwrap_or_default()
}

pub fn serialize_metronome_active(active: bool) -> String {
    if active { "true".to_string() } else { "false".to_string() }
}

pub fn deserialize_metronome_active(s: &str) -> bool {
    s == "true"
}

// --- localStorage I/O (WASM only) ---

#[cfg(target_arch = "wasm32")]
fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

#[cfg(target_arch = "wasm32")]
fn ls_get(key: &str) -> Option<String> {
    local_storage()?.get_item(key).ok()?
}

#[cfg(target_arch = "wasm32")]
fn ls_set(key: &str, value: &str) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(key, value);
    }
}

/// Load persisted fields from localStorage. Falls back to defaults on any error.
pub fn load_state() -> PersistedState {
    #[cfg(target_arch = "wasm32")]
    {
        let theme = ls_get(KEY_THEME)
            .map(|s| deserialize_theme(&s))
            .unwrap_or(Theme::Dark);
        let muted = ls_get(KEY_MUTED)
            .map(|s| deserialize_muted(&s))
            .unwrap_or(false);
        let favorites = ls_get(KEY_FAVORITES)
            .map(|s| deserialize_favorites(&s))
            .unwrap_or_default();
        let metronome_active = ls_get(KEY_METRONOME_ACTIVE)
            .map(|s| deserialize_metronome_active(&s))
            .unwrap_or(false);
        PersistedState { theme, muted, favorites, metronome_active }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        PersistedState::default()
    }
}

/// Persist the relevant fields of `AppState` to localStorage. Fails silently.
pub fn save_state(state: &AppState) {
    #[cfg(target_arch = "wasm32")]
    {
        ls_set(KEY_THEME, &serialize_theme(state.theme));
        ls_set(KEY_MUTED, &serialize_muted(state.muted));
        ls_set(KEY_FAVORITES, &serialize_favorites(&state.favorites));
        ls_set(KEY_METRONOME_ACTIVE, &serialize_metronome_active(state.metronome_active));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = state; // no-op in non-WASM targets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------ //
    // Feature: circle-of-fifths, Property 18: localStorage round-trip    //
    // ------------------------------------------------------------------ //
    //
    // We test the serde round-trip of each persisted field against its
    // string representation (the same values that would be written to and
    // read from localStorage). No browser APIs are required.

    #[test]
    fn theme_round_trip_dark() {
        let original = Theme::Dark;
        let serialized = serialize_theme(original);
        assert_eq!(serialized, "dark");
        assert_eq!(deserialize_theme(&serialized), original);
    }

    #[test]
    fn theme_round_trip_light() {
        let original = Theme::Light;
        let serialized = serialize_theme(original);
        assert_eq!(serialized, "light");
        assert_eq!(deserialize_theme(&serialized), original);
    }

    #[test]
    fn muted_round_trip_true() {
        let serialized = serialize_muted(true);
        assert_eq!(serialized, "true");
        assert_eq!(deserialize_muted(&serialized), true);
    }

    #[test]
    fn muted_round_trip_false() {
        let serialized = serialize_muted(false);
        assert_eq!(serialized, "false");
        assert_eq!(deserialize_muted(&serialized), false);
    }

    #[test]
    fn favorites_round_trip_empty() {
        let original: Vec<ProgressionId> = vec![];
        let s = serialize_favorites(&original);
        assert_eq!(deserialize_favorites(&s), original);
    }

    #[test]
    fn favorites_round_trip_nonempty() {
        let original: Vec<ProgressionId> = vec![1, 5, 42, 999];
        let s = serialize_favorites(&original);
        assert_eq!(deserialize_favorites(&s), original);
    }

    // ------------------------------------------------------------------ //
    // Task 6.2 — storage error handling                                   //
    // ------------------------------------------------------------------ //

    #[test]
    fn deserialize_theme_unknown_value_falls_back_to_dark() {
        assert_eq!(deserialize_theme("garbage"), Theme::Dark);
        assert_eq!(deserialize_theme(""), Theme::Dark);
        assert_eq!(deserialize_theme("LIGHT"), Theme::Dark); // case-sensitive
    }

    #[test]
    fn deserialize_muted_unknown_value_falls_back_to_false() {
        assert_eq!(deserialize_muted("yes"), false);
        assert_eq!(deserialize_muted("1"), false);
        assert_eq!(deserialize_muted("True"), false);
        assert_eq!(deserialize_muted(""), false);
    }

    #[test]
    fn deserialize_favorites_invalid_json_falls_back_to_empty() {
        let empty: Vec<ProgressionId> = vec![];
        assert_eq!(deserialize_favorites("not-json"), empty);
        assert_eq!(deserialize_favorites("{bad}"), empty);
        assert_eq!(deserialize_favorites(""), empty);
    }

    #[test]
    fn load_state_returns_defaults_in_native_target() {
        // In non-WASM builds, load_state() always returns defaults.
        let state = load_state();
        assert_eq!(state.theme, Theme::Dark);
        assert_eq!(state.muted, false);
        assert!(state.favorites.is_empty());
        assert_eq!(state.metronome_active, false);
    }

    #[test]
    fn metronome_active_round_trip_true() {
        let serialized = serialize_metronome_active(true);
        assert_eq!(serialized, "true");
        assert_eq!(deserialize_metronome_active(&serialized), true);
    }

    #[test]
    fn metronome_active_round_trip_false() {
        let serialized = serialize_metronome_active(false);
        assert_eq!(serialized, "false");
        assert_eq!(deserialize_metronome_active(&serialized), false);
    }

    #[test]
    fn deserialize_metronome_active_unknown_value_falls_back_to_false() {
        assert_eq!(deserialize_metronome_active("yes"), false);
        assert_eq!(deserialize_metronome_active("1"), false);
        assert_eq!(deserialize_metronome_active("True"), false);
        assert_eq!(deserialize_metronome_active(""), false);
    }
}
