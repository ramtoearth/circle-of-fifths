use crate::music_theory::{Key, Mode, PitchClass, ScaleDegree};
use crate::state::{BorrowedChord, Progression, ProgressionId, ProgressionTag};

#[cfg(test)]
mod tests;

// ── Internal helpers ──────────────────────────────────────────────────────────

fn degree_idx(degree: ScaleDegree) -> usize {
    match degree {
        ScaleDegree::I   => 0,
        ScaleDegree::II  => 1,
        ScaleDegree::III => 2,
        ScaleDegree::IV  => 3,
        ScaleDegree::V   => 4,
        ScaleDegree::VI  => 5,
        ScaleDegree::VII => 6,
    }
}

fn pitch_semitone(pc: PitchClass) -> u8 {
    match pc {
        PitchClass::C  => 0,  PitchClass::Db => 1,  PitchClass::D  => 2,
        PitchClass::Eb => 3,  PitchClass::E  => 4,  PitchClass::F  => 5,
        PitchClass::Gb => 6,  PitchClass::G  => 7,  PitchClass::Ab => 8,
        PitchClass::A  => 9,  PitchClass::Bb => 10, PitchClass::B  => 11,
    }
}

/// Returns (roman_numeral, chord_name) for each of the 7 diatonic degrees of a major key.
fn major_chord_table(root: PitchClass) -> [(&'static str, &'static str); 7] {
    use PitchClass::*;
    match root {
        C  => [("I","C"),   ("ii","Dm"),   ("iii","Em"),   ("IV","F"),   ("V","G"),   ("vi","Am"),   ("vii°","Bdim") ],
        G  => [("I","G"),   ("ii","Am"),   ("iii","Bm"),   ("IV","C"),   ("V","D"),   ("vi","Em"),   ("vii°","F#dim")],
        D  => [("I","D"),   ("ii","Em"),   ("iii","F#m"),  ("IV","G"),   ("V","A"),   ("vi","Bm"),   ("vii°","C#dim")],
        A  => [("I","A"),   ("ii","Bm"),   ("iii","C#m"),  ("IV","D"),   ("V","E"),   ("vi","F#m"),  ("vii°","G#dim")],
        E  => [("I","E"),   ("ii","F#m"),  ("iii","G#m"),  ("IV","A"),   ("V","B"),   ("vi","C#m"),  ("vii°","D#dim")],
        B  => [("I","B"),   ("ii","C#m"),  ("iii","D#m"),  ("IV","E"),   ("V","F#"),  ("vi","G#m"),  ("vii°","A#dim")],
        Gb => [("I","Gb"),  ("ii","Abm"),  ("iii","Bbm"),  ("IV","Cb"),  ("V","Db"),  ("vi","Ebm"),  ("vii°","Fdim") ],
        Db => [("I","Db"),  ("ii","Ebm"),  ("iii","Fm"),   ("IV","Gb"),  ("V","Ab"),  ("vi","Bbm"),  ("vii°","Cdim") ],
        Ab => [("I","Ab"),  ("ii","Bbm"),  ("iii","Cm"),   ("IV","Db"),  ("V","Eb"),  ("vi","Fm"),   ("vii°","Gdim") ],
        Eb => [("I","Eb"),  ("ii","Fm"),   ("iii","Gm"),   ("IV","Ab"),  ("V","Bb"),  ("vi","Cm"),   ("vii°","Ddim") ],
        Bb => [("I","Bb"),  ("ii","Cm"),   ("iii","Dm"),   ("IV","Eb"),  ("V","F"),   ("vi","Gm"),   ("vii°","Adim") ],
        F  => [("I","F"),   ("ii","Gm"),   ("iii","Am"),   ("IV","Bb"),  ("V","C"),   ("vi","Dm"),   ("vii°","Edim") ],
    }
}

/// Returns the flat Roman numeral for a chord borrowed into `host_root` from `source_root`.
fn borrowed_roman(host_root: PitchClass, source_root: PitchClass) -> &'static str {
    let offset = (pitch_semitone(source_root) + 12 - pitch_semitone(host_root)) % 12;
    match offset {
        1  => "bII",
        3  => "bIII",
        6  => "bV",
        8  => "bVI",
        10 => "bVII",
        _  => "N",
    }
}

fn make_key(root: PitchClass) -> Key {
    Key { root, mode: Mode::Major }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Resolve the chord name for a scale degree in a key, using the borrowed chord override if applicable.
pub fn resolve_chord_name(key: Key, degree: ScaleDegree, borrowed: Option<&BorrowedChord>) -> String {
    if let Some(bc) = borrowed {
        if bc.degree == degree {
            // The borrowed chord is the I chord of the source key
            return major_chord_table(bc.source_key.root)[0].1.to_string();
        }
    }
    major_chord_table(key.root)[degree_idx(degree)].1.to_string()
}

/// Resolve the Roman numeral for a scale degree, using the borrowed label if applicable.
pub fn resolve_roman(key: Key, degree: ScaleDegree, borrowed: Option<&BorrowedChord>) -> &'static str {
    if let Some(bc) = borrowed {
        if bc.degree == degree {
            return borrowed_roman(key.root, bc.source_key.root);
        }
    }
    major_chord_table(key.root)[degree_idx(degree)].0
}

/// Format a progression as "I - V - vi - IV = C - G - Am - F".
pub fn format_progression(progression: &Progression) -> String {
    let borrowed = progression.borrowed_chord.as_ref();
    let romans: Vec<&str> = progression.chords.iter()
        .map(|&d| resolve_roman(progression.key, d, borrowed))
        .collect();
    let names: Vec<String> = progression.chords.iter()
        .map(|&d| resolve_chord_name(progression.key, d, borrowed))
        .collect();
    format!("{} = {}", romans.join(" - "), names.join(" - "))
}

/// Return all curated progressions for a given major key.
pub fn progressions_for_key(key: Key) -> Vec<Progression> {
    all_progressions().into_iter().filter(|p| p.key == key).collect()
}

/// All curated progressions for the 12 major keys (5 per key, 60 total).
pub fn all_progressions() -> Vec<Progression> {
    use PitchClass::*;

    // (key_root, base_id, bvii_root) — bVII = root + 10 semitones (mod 12)
    let key_data: [(PitchClass, ProgressionId, PitchClass); 12] = [
        (C,  0,  Bb), (G,  5,  F),  (D,  10, C),
        (A,  15, G),  (E,  20, D),  (B,  25, A),
        (Gb, 30, E),  (Db, 35, B),  (Ab, 40, Gb),
        (Eb, 45, Db), (Bb, 50, Ab), (F,  55, Eb),
    ];

    let mut out = Vec::with_capacity(60);
    for (root, base_id, bvii_root) in key_data {
        out.extend(key_progressions(root, base_id, bvii_root));
    }
    out
}

// ── Per-key progression builder ───────────────────────────────────────────────

/// Produces the 5 curated progressions for one major key.
///
/// Tags coverage per key: Pop, Uplifting, Classical, Jazz, Melancholic, Blues (6 distinct, ≥3 required).
/// Borrowed chord: progression 5 has bVII borrowed from `bvii_root` (parallel Mixolydian).
fn key_progressions(root: PitchClass, base_id: ProgressionId, bvii_root: PitchClass) -> Vec<Progression> {
    use ScaleDegree::*;
    use ProgressionTag::*;
    let key = make_key(root);
    let bvii_key = make_key(bvii_root);

    vec![
        // 1. I - V - vi - IV  (Pop, Uplifting) — e.g. C - G - Am - F
        Progression {
            id: base_id,
            key,
            chords: vec![I, V, VI, IV],
            tags: vec![Pop, Uplifting],
            borrowed_chord: None,
        },
        // 2. I - IV - V - I  (Classical) — traditional authentic cadence
        Progression {
            id: base_id + 1,
            key,
            chords: vec![I, IV, V, I],
            tags: vec![Classical],
            borrowed_chord: None,
        },
        // 3. ii - V - I - IV  (Jazz) — jazz turnaround variant
        Progression {
            id: base_id + 2,
            key,
            chords: vec![II, V, I, IV],
            tags: vec![Jazz],
            borrowed_chord: None,
        },
        // 4. I - vi - IV - V  (Melancholic, Pop) — 50s / doo-wop progression
        Progression {
            id: base_id + 3,
            key,
            chords: vec![I, VI, IV, V],
            tags: vec![Melancholic, Pop],
            borrowed_chord: None,
        },
        // 5. I - bVII - IV - I  (Blues, Pop) — bVII borrowed from parallel Mixolydian
        Progression {
            id: base_id + 4,
            key,
            chords: vec![I, VII, IV, I],
            tags: vec![Blues, Pop],
            borrowed_chord: Some(BorrowedChord {
                degree: VII,
                source_key: bvii_key,
            }),
        },
    ]
}
