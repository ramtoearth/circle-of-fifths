use yew::prelude::*;
use std::f64::consts::PI;
use crate::music_theory::{Key, Mode, PitchClass, key_signature};

const MAJOR_ROOTS: [PitchClass; 12] = [
    PitchClass::C, PitchClass::G, PitchClass::D, PitchClass::A,
    PitchClass::E, PitchClass::B, PitchClass::Gb, PitchClass::Db,
    PitchClass::Ab, PitchClass::Eb, PitchClass::Bb, PitchClass::F,
];

const MINOR_ROOTS: [PitchClass; 12] = [
    PitchClass::A, PitchClass::E, PitchClass::B, PitchClass::Gb,
    PitchClass::Db, PitchClass::Ab, PitchClass::Eb, PitchClass::Bb,
    PitchClass::F, PitchClass::C, PitchClass::G, PitchClass::D,
];

fn deg_to_rad(deg: f64) -> f64 {
    deg * PI / 180.0
}

fn arc_path(cx: f64, cy: f64, r_outer: f64, r_inner: f64, start_deg: f64, end_deg: f64) -> String {
    let s = deg_to_rad(start_deg);
    let e = deg_to_rad(end_deg);
    let x1 = cx + r_outer * s.cos();
    let y1 = cy + r_outer * s.sin();
    let x2 = cx + r_outer * e.cos();
    let y2 = cy + r_outer * e.sin();
    let x3 = cx + r_inner * e.cos();
    let y3 = cy + r_inner * e.sin();
    let x4 = cx + r_inner * s.cos();
    let y4 = cy + r_inner * s.sin();
    format!(
        "M {x1:.2} {y1:.2} A {r_outer} {r_outer} 0 0 1 {x2:.2} {y2:.2} L {x3:.2} {y3:.2} A {r_inner} {r_inner} 0 0 0 {x4:.2} {y4:.2} Z"
    )
}

fn mid_point(cx: f64, cy: f64, r: f64, mid_deg: f64) -> (f64, f64) {
    let rad = deg_to_rad(mid_deg);
    (cx + r * rad.cos(), cy + r * rad.sin())
}

/// Returns the circle index (column) for the selected key.
/// For major: search MAJOR_ROOTS. For minor: search MINOR_ROOTS.
fn selected_index(selected_key: Option<Key>) -> Option<usize> {
    selected_key.map(|k| {
        match k.mode {
            Mode::Major => MAJOR_ROOTS.iter().position(|&r| r == k.root),
            Mode::Minor => MINOR_ROOTS.iter().position(|&r| r == k.root),
        }
        .unwrap_or(0)
    })
}

fn segment_class(i: usize, selected_key: Option<Key>) -> &'static str {
    let sel_idx = match selected_index(selected_key) {
        Some(idx) => idx,
        None => return "segment",
    };

    if i == sel_idx {
        return "segment selected";
    }

    // Check adjacency: indices one step away (mod 12)
    let cw = (sel_idx + 1) % 12;
    let ccw = (sel_idx + 11) % 12;
    if i == cw || i == ccw {
        return "segment adjacent";
    }

    // Check opposite: 6 steps away
    let opp = (sel_idx + 6) % 12;
    if i == opp {
        return "segment opposite";
    }

    "segment"
}

fn accidental_label(key: Key) -> String {
    let sig = key_signature(key);
    if sig.sharps > 0 {
        format!("{}♯", sig.sharps)
    } else if sig.flats > 0 {
        format!("{}♭", sig.flats)
    } else {
        String::new()
    }
}

#[derive(Properties, PartialEq)]
pub struct CircleViewProps {
    pub selected_key: Option<Key>,
    pub on_segment_click: Callback<Key>,
}

#[function_component(CircleView)]
pub fn circle_view(props: &CircleViewProps) -> Html {
    let cx = 250.0_f64;
    let cy = 250.0_f64;

    // Outer ring (major): r_outer=230, r_inner=160
    let major_r_outer = 230.0_f64;
    let major_r_inner = 160.0_f64;
    let major_mid_r = (major_r_outer + major_r_inner) / 2.0;

    // Inner ring (minor): r_outer=155, r_inner=95
    let minor_r_outer = 155.0_f64;
    let minor_r_inner = 95.0_f64;
    let minor_mid_r = (minor_r_outer + minor_r_inner) / 2.0;

    let gap = 1.0_f64; // 1 degree gap on each side
    let segment_span = 30.0_f64;

    let mut segments = Vec::new();

    for i in 0..12usize {
        let base_deg = -90.0 + (i as f64) * segment_span;
        let start_deg = base_deg + gap;
        let end_deg = base_deg + segment_span - gap;
        let mid_deg = base_deg + segment_span / 2.0;

        // ── Major segment ──────────────────────────────────────────────────
        let major_root = MAJOR_ROOTS[i];
        let major_key = Key::major(major_root);
        let major_path = arc_path(cx, cy, major_r_outer, major_r_inner, start_deg, end_deg);
        let major_class = segment_class(i, props.selected_key);
        let (major_label_x, major_label_y) = mid_point(cx, cy, major_mid_r, mid_deg);
        let major_name = major_root.name().to_string();
        let major_acc = accidental_label(major_key);

        // Name label slightly above center, accidental slightly below
        let major_name_y = major_label_y - 7.0;
        let major_acc_y = major_label_y + 7.0;

        let on_click_major = {
            let cb = props.on_segment_click.clone();
            Callback::from(move |_: MouseEvent| cb.emit(major_key))
        };

        // ── Minor segment ──────────────────────────────────────────────────
        let minor_root = MINOR_ROOTS[i];
        let minor_key = Key::minor(minor_root);
        let minor_path = arc_path(cx, cy, minor_r_outer, minor_r_inner, start_deg, end_deg);
        let minor_class = segment_class(i, props.selected_key);
        let (minor_label_x, minor_label_y) = mid_point(cx, cy, minor_mid_r, mid_deg);
        let minor_name = format!("{}m", minor_root.name());
        let minor_acc = accidental_label(minor_key);

        let minor_name_y = minor_label_y - 6.0;
        let minor_acc_y = minor_label_y + 6.0;

        let on_click_minor = {
            let cb = props.on_segment_click.clone();
            Callback::from(move |_: MouseEvent| cb.emit(minor_key))
        };

        segments.push(html! {
            <>
                // Major segment path
                <path
                    d={major_path}
                    class={major_class}
                    style="cursor:pointer"
                    onclick={on_click_major}
                />
                // Major key name
                <text
                    x={format!("{major_label_x:.2}")}
                    y={format!("{major_name_y:.2}")}
                    text-anchor="middle"
                    dominant-baseline="middle"
                    style="font-size:12px;pointer-events:none"
                >
                    {major_name}
                </text>
                // Major accidental label
                if !major_acc.is_empty() {
                    <text
                        x={format!("{major_label_x:.2}")}
                        y={format!("{major_acc_y:.2}")}
                        text-anchor="middle"
                        dominant-baseline="middle"
                        style="font-size:11px;pointer-events:none"
                    >
                        {major_acc}
                    </text>
                }

                // Minor segment path
                <path
                    d={minor_path}
                    class={minor_class}
                    style="cursor:pointer"
                    onclick={on_click_minor}
                />
                // Minor key name
                <text
                    x={format!("{minor_label_x:.2}")}
                    y={format!("{minor_name_y:.2}")}
                    text-anchor="middle"
                    dominant-baseline="middle"
                    style="font-size:10px;pointer-events:none"
                >
                    {minor_name}
                </text>
                // Minor accidental label
                if !minor_acc.is_empty() {
                    <text
                        x={format!("{minor_label_x:.2}")}
                        y={format!("{minor_acc_y:.2}")}
                        text-anchor="middle"
                        dominant-baseline="middle"
                        style="font-size:9px;pointer-events:none"
                    >
                        {minor_acc}
                    </text>
                }
            </>
        });
    }

    html! {
        <svg viewBox="0 0 500 500" xmlns="http://www.w3.org/2000/svg" style="width:500px;height:500px">
            { for segments.into_iter() }

            // Center circle
            <circle cx="250" cy="250" r="85" class="center-circle" />
            <text
                x="250"
                y="250"
                text-anchor="middle"
                dominant-baseline="middle"
                style="font-size:13px;pointer-events:none"
            >
                {"Circle of Fifths"}
            </text>
        </svg>
    }
}
