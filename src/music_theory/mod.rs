use serde::{Deserialize, Serialize};

/// The 12 pitch classes
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum PitchClass {
    C, Db, D, Eb, E, F, Gb, G, Ab, A, Bb, B,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Mode { Major, Minor }

/// A key is a pitch class + mode
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Key {
    pub root: PitchClass,
    pub mode: Mode,
}

/// Scale degree 1-7
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ScaleDegree { I, II, III, IV, V, VI, VII }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ChordQuality { Major, Minor, Diminished }

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DiatonicChord {
    pub degree: ScaleDegree,
    pub quality: ChordQuality,
    pub root: PitchClass,
    pub notes: [PitchClass; 3],
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct KeySignature {
    pub sharps: u8,
    pub flats: u8,
    pub notes: Vec<PitchClass>,
}

/// Chord highlight for piano panel
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ChordHighlight {
    pub root: PitchClass,
    pub third: PitchClass,
    pub fifth: PitchClass,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum KeyRole { Root, Third, Fifth, ScaleNote, None }

// ─────────────────────────── Progression / data types ────────────────────────
// Kept here so that `data` can import them without creating a circular
// dependency with `state`.

pub type ProgressionId = u32;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BorrowedChord {
    pub degree: ScaleDegree,
    pub source_key: Key,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ProgressionTag {
    Pop, Jazz, Blues, Classical, Melancholic, Uplifting,
    #[serde(other)]
    Custom,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Progression {
    pub id: ProgressionId,
    pub key: Key,
    pub chords: Vec<ScaleDegree>,
    pub tags: Vec<ProgressionTag>,
    pub borrowed_chord: Option<BorrowedChord>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActiveProgression {
    pub id: ProgressionId,
    pub current_index: usize,
}

impl PitchClass {
    /// Returns chromatic index 0..11 (C=0, Db=1, ..., B=11)
    pub fn to_index(self) -> u8 {
        match self {
            PitchClass::C  => 0,
            PitchClass::Db => 1,
            PitchClass::D  => 2,
            PitchClass::Eb => 3,
            PitchClass::E  => 4,
            PitchClass::F  => 5,
            PitchClass::Gb => 6,
            PitchClass::G  => 7,
            PitchClass::Ab => 8,
            PitchClass::A  => 9,
            PitchClass::Bb => 10,
            PitchClass::B  => 11,
        }
    }

    /// Inverse of to_index
    pub fn from_index(i: u8) -> Self {
        match i % 12 {
            0  => PitchClass::C,
            1  => PitchClass::Db,
            2  => PitchClass::D,
            3  => PitchClass::Eb,
            4  => PitchClass::E,
            5  => PitchClass::F,
            6  => PitchClass::Gb,
            7  => PitchClass::G,
            8  => PitchClass::Ab,
            9  => PitchClass::A,
            10 => PitchClass::Bb,
            _  => PitchClass::B,
        }
    }

    /// Add semitones (wraps around)
    pub fn add_semitones(self, n: u8) -> Self {
        PitchClass::from_index((self.to_index() + n) % 12)
    }

    /// Returns canonical display name (flat spelling for accidentals)
    pub fn name(self) -> &'static str {
        match self {
            PitchClass::C  => "C",
            PitchClass::Db => "D♭",
            PitchClass::D  => "D",
            PitchClass::Eb => "E♭",
            PitchClass::E  => "E",
            PitchClass::F  => "F",
            PitchClass::Gb => "G♭",
            PitchClass::G  => "G",
            PitchClass::Ab => "A♭",
            PitchClass::A  => "A",
            PitchClass::Bb => "B♭",
            PitchClass::B  => "B",
        }
    }

    /// Returns sharp spelling for enharmonic notes (e.g. Gb → "F♯")
    pub fn sharp_name(self) -> &'static str {
        match self {
            PitchClass::Db => "C♯",
            PitchClass::Eb => "D♯",
            PitchClass::Gb => "F♯",
            PitchClass::Ab => "G♯",
            PitchClass::Bb => "A♯",
            other          => other.name(),
        }
    }
}

impl Key {
    pub fn major(root: PitchClass) -> Key {
        Key { root, mode: Mode::Major }
    }

    pub fn minor(root: PitchClass) -> Key {
        Key { root, mode: Mode::Minor }
    }
}

/// Returns the 7 scale notes for a key.
/// Major: W-W-H-W-W-W-H intervals = [0, 2, 4, 5, 7, 9, 11]
/// Minor (natural): [0, 2, 3, 5, 7, 8, 10]
pub fn scale_notes(key: Key) -> [PitchClass; 7] {
    let intervals: [u8; 7] = match key.mode {
        Mode::Major => [0, 2, 4, 5, 7, 9, 11],
        Mode::Minor => [0, 2, 3, 5, 7, 8, 10],
    };
    let root = key.root.to_index();
    [
        PitchClass::from_index((root + intervals[0]) % 12),
        PitchClass::from_index((root + intervals[1]) % 12),
        PitchClass::from_index((root + intervals[2]) % 12),
        PitchClass::from_index((root + intervals[3]) % 12),
        PitchClass::from_index((root + intervals[4]) % 12),
        PitchClass::from_index((root + intervals[5]) % 12),
        PitchClass::from_index((root + intervals[6]) % 12),
    ]
}

/// Returns the key signature (sharps/flats count and affected notes).
/// For minor keys, uses the relative major's signature.
pub fn key_signature(key: Key) -> KeySignature {
    // For minor keys, compute via relative major
    let major_key = match key.mode {
        Mode::Major => key,
        Mode::Minor => relative_major(key),
    };

    // Circle of fifths: sharps keys (number of sharps)
    // C=0, G=1, D=2, A=3, E=4, B=5, F#/Gb=6, C#=7
    // Flats keys: F=1, Bb=2, Eb=3, Ab=4, Db=5, Gb=6, Cb=7
    //
    // Sharp order: F# C# G# D# A# E# B# -> stored as Gb Db Ab Eb Bb F C (enharmonics)
    // Flat order: Bb Eb Ab Db Gb Cb Fb -> stored as Bb Eb Ab Db Gb (Cb=B, Fb=E in our enum)

    // Sharps in circle of fifths order by root index:
    // G(7)=1, D(2)=2, A(9)=3, E(4)=4, B(11)=5, F#/Gb(6)=6, C#/Db(1)=7
    let sharp_roots: [(u8, u8); 7] = [
        (7, 1),  // G major: 1 sharp
        (2, 2),  // D major: 2 sharps
        (9, 3),  // A major: 3 sharps
        (4, 4),  // E major: 4 sharps
        (11, 5), // B major: 5 sharps
        (6, 6),  // F#/Gb major: 6 sharps
        (1, 7),  // C#/Db major: 7 sharps
    ];

    // Flat roots in circle of fifths order:
    // F(5)=1, Bb(10)=2, Eb(3)=3, Ab(8)=4, Db(1)=5, Gb(6)=6, Cb(11)=7
    let flat_roots: [(u8, u8); 7] = [
        (5, 1),  // F major: 1 flat
        (10, 2), // Bb major: 2 flats
        (3, 3),  // Eb major: 3 flats
        (8, 4),  // Ab major: 4 flats
        (1, 5),  // Db major: 5 flats
        (6, 6),  // Gb major: 6 flats
        (11, 7), // Cb major: 7 flats
    ];

    // Sharp notes in order: F# C# G# D# A# E# B#
    // Stored as PitchClass enharmonics: Gb Db Ab Eb Bb F C
    let sharp_notes: [PitchClass; 7] = [
        PitchClass::Gb, // F#
        PitchClass::Db, // C#
        PitchClass::Ab, // G#
        PitchClass::Eb, // D#
        PitchClass::Bb, // A#
        PitchClass::F,  // E#
        PitchClass::C,  // B#
    ];

    // Flat notes in order: Bb Eb Ab Db Gb Cb Fb
    // Stored as PitchClass: Bb Eb Ab Db Gb B E
    let flat_notes: [PitchClass; 7] = [
        PitchClass::Bb, // Bb
        PitchClass::Eb, // Eb
        PitchClass::Ab, // Ab
        PitchClass::Db, // Db
        PitchClass::Gb, // Gb
        PitchClass::B,  // Cb (enharmonic B)
        PitchClass::E,  // Fb (enharmonic E)
    ];

    let root_idx = major_key.root.to_index();

    // C major = 0 accidentals
    if root_idx == 0 {
        return KeySignature { sharps: 0, flats: 0, notes: vec![] };
    }

    // Check sharps
    for &(r, count) in &sharp_roots {
        if root_idx == r {
            let notes = sharp_notes[..count as usize].to_vec();
            return KeySignature { sharps: count, flats: 0, notes };
        }
    }

    // Check flats
    for &(r, count) in &flat_roots {
        if root_idx == r {
            let notes = flat_notes[..count as usize].to_vec();
            return KeySignature { sharps: 0, flats: count, notes };
        }
    }

    // Fallback (should not reach here for valid 12 pitch classes)
    KeySignature { sharps: 0, flats: 0, notes: vec![] }
}

/// Returns the 7 diatonic chords for the given key.
/// For major: I=Major, ii=Minor, iii=Minor, IV=Major, V=Major, vi=Minor, vii°=Diminished
/// For minor (natural): i=Minor, ii°=Diminished, III=Major, iv=Minor, v=Minor, VI=Major, VII=Major
pub fn diatonic_chords(key: Key) -> [DiatonicChord; 7] {
    let notes = scale_notes(key);
    let degrees = [
        ScaleDegree::I,
        ScaleDegree::II,
        ScaleDegree::III,
        ScaleDegree::IV,
        ScaleDegree::V,
        ScaleDegree::VI,
        ScaleDegree::VII,
    ];

    let qualities: [ChordQuality; 7] = match key.mode {
        Mode::Major => [
            ChordQuality::Major,
            ChordQuality::Minor,
            ChordQuality::Minor,
            ChordQuality::Major,
            ChordQuality::Major,
            ChordQuality::Minor,
            ChordQuality::Diminished,
        ],
        Mode::Minor => [
            ChordQuality::Minor,
            ChordQuality::Diminished,
            ChordQuality::Major,
            ChordQuality::Minor,
            ChordQuality::Minor,
            ChordQuality::Major,
            ChordQuality::Major,
        ],
    };

    let build_triad = |root: PitchClass, quality: ChordQuality| -> [PitchClass; 3] {
        let (third_interval, fifth_interval) = match quality {
            ChordQuality::Major     => (4u8, 7u8),
            ChordQuality::Minor     => (3u8, 7u8),
            ChordQuality::Diminished => (3u8, 6u8),
        };
        [
            root,
            root.add_semitones(third_interval),
            root.add_semitones(fifth_interval),
        ]
    };

    [
        DiatonicChord { degree: degrees[0], quality: qualities[0], root: notes[0], notes: build_triad(notes[0], qualities[0]) },
        DiatonicChord { degree: degrees[1], quality: qualities[1], root: notes[1], notes: build_triad(notes[1], qualities[1]) },
        DiatonicChord { degree: degrees[2], quality: qualities[2], root: notes[2], notes: build_triad(notes[2], qualities[2]) },
        DiatonicChord { degree: degrees[3], quality: qualities[3], root: notes[3], notes: build_triad(notes[3], qualities[3]) },
        DiatonicChord { degree: degrees[4], quality: qualities[4], root: notes[4], notes: build_triad(notes[4], qualities[4]) },
        DiatonicChord { degree: degrees[5], quality: qualities[5], root: notes[5], notes: build_triad(notes[5], qualities[5]) },
        DiatonicChord { degree: degrees[6], quality: qualities[6], root: notes[6], notes: build_triad(notes[6], qualities[6]) },
    ]
}

/// Returns the relative minor key for a given major key (6th scale degree = 9 semitones up).
pub fn relative_minor(major: Key) -> Key {
    Key::minor(major.root.add_semitones(9))
}

/// Returns the relative major key for a given minor key (3 semitones up).
pub fn relative_major(minor: Key) -> Key {
    Key::major(minor.root.add_semitones(3))
}

/// Returns (clockwise_neighbor, counterclockwise_neighbor) on the circle of fifths.
/// Clockwise = +7 semitones (perfect fifth up), counterclockwise = +5 semitones (perfect fourth up).
pub fn adjacent_keys(key: Key) -> (Key, Key) {
    let cw = Key { root: key.root.add_semitones(7), mode: key.mode };
    let ccw = Key { root: key.root.add_semitones(5), mode: key.mode };
    (cw, ccw)
}

/// Returns the key directly opposite on the circle (tritone = 6 semitones away).
pub fn opposite_key(key: Key) -> Key {
    Key { root: key.root.add_semitones(6), mode: key.mode }
}

/// Formats a ScaleDegree + ChordQuality as Roman numeral string.
pub fn roman_numeral(degree: ScaleDegree, quality: ChordQuality) -> &'static str {
    match (degree, quality) {
        (ScaleDegree::I,   ChordQuality::Major)      => "I",
        (ScaleDegree::I,   ChordQuality::Minor)      => "i",
        (ScaleDegree::I,   ChordQuality::Diminished) => "i°",
        (ScaleDegree::II,  ChordQuality::Major)      => "II",
        (ScaleDegree::II,  ChordQuality::Minor)      => "ii",
        (ScaleDegree::II,  ChordQuality::Diminished) => "ii°",
        (ScaleDegree::III, ChordQuality::Major)      => "III",
        (ScaleDegree::III, ChordQuality::Minor)      => "iii",
        (ScaleDegree::III, ChordQuality::Diminished) => "iii°",
        (ScaleDegree::IV,  ChordQuality::Major)      => "IV",
        (ScaleDegree::IV,  ChordQuality::Minor)      => "iv",
        (ScaleDegree::IV,  ChordQuality::Diminished) => "iv°",
        (ScaleDegree::V,   ChordQuality::Major)      => "V",
        (ScaleDegree::V,   ChordQuality::Minor)      => "v",
        (ScaleDegree::V,   ChordQuality::Diminished) => "v°",
        (ScaleDegree::VI,  ChordQuality::Major)      => "VI",
        (ScaleDegree::VI,  ChordQuality::Minor)      => "vi",
        (ScaleDegree::VI,  ChordQuality::Diminished) => "vi°",
        (ScaleDegree::VII, ChordQuality::Major)      => "VII",
        (ScaleDegree::VII, ChordQuality::Minor)      => "vii",
        (ScaleDegree::VII, ChordQuality::Diminished) => "vii°",
    }
}

/// Returns the full chord name like "C", "Dm", "Bdim".
pub fn chord_name(root: PitchClass, quality: ChordQuality) -> String {
    let root_name = root.name();
    match quality {
        ChordQuality::Major     => root_name.to_string(),
        ChordQuality::Minor     => format!("{}m", root_name),
        ChordQuality::Diminished => format!("{}dim", root_name),
    }
}

/// Returns the display string for a diatonic chord, e.g. "vi - Am".
pub fn chord_display(chord: &DiatonicChord) -> String {
    let rn = roman_numeral(chord.degree, chord.quality);
    let cn = chord_name(chord.root, chord.quality);
    format!("{} - {}", rn, cn)
}

mod tests;
