use yew::prelude::*;

use crate::data::{format_progression, progressions_for_key, resolve_roman};
use crate::midi::MidiStatus;
use crate::music_theory::{ActiveProgression, Key, Mode, Progression, ProgressionId, ProgressionTag};

// ─────────────────────────── Props ───────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct ProgressionPanelProps {
    pub selected_key: Option<Key>,
    pub active_progression: Option<ActiveProgression>,
    pub favorites: Vec<ProgressionId>,
    pub on_progression_click: Callback<ProgressionId>,
    pub on_next: Callback<()>,
    pub on_prev: Callback<()>,
    pub on_favorite_toggle: Callback<ProgressionId>,
    pub midi_status: MidiStatus,
    pub on_enter_play_along: Callback<ProgressionId>,
}

// ─────────────────────────── Helpers ─────────────────────────────────────────

fn tag_label(tag: &ProgressionTag) -> &'static str {
    match tag {
        ProgressionTag::Pop        => "Pop",
        ProgressionTag::Jazz       => "Jazz",
        ProgressionTag::Blues      => "Blues",
        ProgressionTag::Classical  => "Classical",
        ProgressionTag::Melancholic => "Melancholic",
        ProgressionTag::Uplifting  => "Uplifting",
        ProgressionTag::Custom     => "Custom",
    }
}

fn key_display(key: Key) -> String {
    let mode = match key.mode {
        Mode::Major => "major",
        Mode::Minor => "minor",
    };
    format!("{} {}", key.root.name(), mode)
}

fn borrowed_label(progression: &Progression) -> Option<String> {
    let bc = progression.borrowed_chord.as_ref()?;
    let rn = resolve_roman(progression.key, bc.degree, Some(bc));
    Some(format!("{} borrowed from {}", rn, key_display(bc.source_key)))
}

// ─────────────────────────── Component ───────────────────────────────────────

#[function_component(ProgressionPanel)]
pub fn progression_panel(props: &ProgressionPanelProps) -> Html {
    let key = match props.selected_key {
        Some(k) => k,
        None => {
            return html! {
                <div class="progression-panel progression-panel--empty">
                    <p class="progression-panel__placeholder">
                        {"Select a key from the circle to see chord progressions."}
                    </p>
                </div>
            };
        }
    };

    let progressions = progressions_for_key(key);

    if progressions.is_empty() {
        return html! {
            <div class="progression-panel progression-panel--empty">
                <p class="progression-panel__placeholder">
                    {"No progressions available for this key."}
                </p>
            </div>
        };
    }

    let items = progressions.iter().map(|prog| {
        let id = prog.id;
        let is_active = props.active_progression
            .as_ref()
            .map_or(false, |ap| ap.id == id);
        let is_favorite = props.favorites.contains(&id);
        let current_index = if is_active {
            props.active_progression.as_ref().map(|ap| ap.current_index)
        } else {
            None
        };

        // Callbacks
        let on_click = {
            let cb = props.on_progression_click.clone();
            Callback::from(move |_: MouseEvent| cb.emit(id))
        };
        let on_fav = {
            let cb = props.on_favorite_toggle.clone();
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                cb.emit(id);
            })
        };
        let on_prev = {
            let cb = props.on_prev.clone();
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                cb.emit(());
            })
        };
        let on_next = {
            let cb = props.on_next.clone();
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                cb.emit(());
            })
        };
        let on_play_along = {
            let cb = props.on_enter_play_along.clone();
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                cb.emit(id);
            })
        };
        let midi_status = props.midi_status;

        // Tag chips
        let tags_html: Html = prog.tags.iter().map(|t| html! {
            <span class="progression-item__tag">{tag_label(t)}</span>
        }).collect();

        // Chord display — highlight the active chord if this progression is selected
        let chord_display = if is_active {
            let borrowed = prog.borrowed_chord.as_ref();
            let chords_html: Html = prog.chords.iter().enumerate().map(|(i, &d)| {
                let rn = resolve_roman(prog.key, d, borrowed);
                let chord_name = crate::data::resolve_chord_name(prog.key, d, borrowed);
                let active_cls = if current_index == Some(i) { " progression-chord--active" } else { "" };
                html! {
                    <span class={format!("progression-chord{}", active_cls)}
                          title={chord_name}>
                        {rn}
                    </span>
                }
            }).collect();
            html! { <div class="progression-item__chords">{chords_html}</div> }
        } else {
            let display = format_progression(prog);
            html! { <div class="progression-item__chords">{display}</div> }
        };

        // Borrowed chord annotation
        let borrowed_html = borrowed_label(prog).map(|label| html! {
            <div class="progression-item__borrowed">
                <span class="progression-item__borrowed-icon">{"⟵"}</span>
                {label}
            </div>
        });

        // Next / prev controls — only shown on the active progression
        let controls_html = if is_active {
            let play_along_html = if midi_status == MidiStatus::Connected {
                html! {
                    <button class="progression-btn progression-btn--play-along"
                            onclick={on_play_along}
                            aria-label="Play Along">
                        {"Play Along"}
                    </button>
                }
            } else {
                html! {
                    <span class="progression-panel__midi-hint">
                        {"Connect a MIDI keyboard to use Play Along"}
                    </span>
                }
            };
            html! {
                <div class="progression-item__controls">
                    <button class="progression-btn progression-btn--prev"
                            onclick={on_prev}
                            aria-label="Previous chord">
                        {"◀"}
                    </button>
                    <button class="progression-btn progression-btn--next"
                            onclick={on_next}
                            aria-label="Next chord">
                        {"▶"}
                    </button>
                    {play_along_html}
                </div>
            }
        } else {
            html! {}
        };

        let item_class = if is_active {
            "progression-item progression-item--active"
        } else {
            "progression-item"
        };
        let fav_label = if is_favorite { "♥" } else { "♡" };
        let fav_class = if is_favorite {
            "progression-item__favorite progression-item__favorite--on"
        } else {
            "progression-item__favorite"
        };

        html! {
            <li class={item_class} onclick={on_click}>
                <div class="progression-item__header">
                    <div class="progression-item__tags">{tags_html}</div>
                    <button class={fav_class} onclick={on_fav}
                            aria-label="Toggle favorite">
                        {fav_label}
                    </button>
                </div>
                {chord_display}
                {borrowed_html}
                {controls_html}
            </li>
        }
    }).collect::<Html>();

    html! {
        <div class="progression-panel">
            <h3 class="progression-panel__title">
                {format!("Progressions in {}", key_display(key))}
            </h3>
            <ul class="progression-panel__list">
                {items}
            </ul>
        </div>
    }
}
