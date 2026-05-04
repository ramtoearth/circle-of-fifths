# Requirements Document

## Introduction

Custom Progression Builder adds a mode where the user constructs a chord progression from scratch
by clicking on diatonic chord slots rather than selecting a predefined one. The current play-along
mode is only accessible through a fixed library of 60 progressions keyed to major keys. Learners
who want to practise a specific sequence — for example, one they are learning from a song — must
find a matching predefined progression or cannot practise at all. The builder presents the seven
diatonic chords of the currently selected key as clickable tiles. A plain click appends the chord
to the end of the working progression; Shift+click appends it again (stacking repeated chords in
sequence). A Reset button clears the slate. When the user is satisfied with the sequence, a "Start
Play Along" button launches the existing wait-based play-along mode using the custom progression.
Inversions and seventh chords are out of scope for this release (V1); only root-position triads are
supported, consistent with the predefined progression library.

## Glossary

- **App**: The Circle of Fifths web application.
- **Custom_Progression_Builder**: The UI mode in which the user assembles a chord progression from
  diatonic chord tiles.
- **Diatonic_Chord_Tile**: A clickable button representing one of the seven diatonic chords of the
  selected key (I through VII).
- **Working_Progression**: The ordered list of `ScaleDegree` values the user has assembled so far
  in the builder. It is ephemeral — not persisted to storage.
- **ScaleDegree**: One of I, II, III, IV, V, VI, VII — the diatonic position of a chord within
  the selected key, as defined in `src/music_theory/mod.rs`.
- **Append**: Add one `ScaleDegree` to the end of the Working_Progression.
- **Pop**: Remove the most-recently-appended occurrence of a specific `ScaleDegree` from the
  Working_Progression (the last occurrence, not all occurrences).
- **Shift+Click**: A mouse click event where `event.shift_key() == true`.
- **Play_Along_Mode**: The existing wait-based play-along mode introduced in the play-along-redesign
  spec.
- **Selected_Key**: The key currently active in `AppState.selected_key`.
- **MIDI_Device**: A physical or virtual MIDI keyboard connected via the Web MIDI API.

---

## Requirements

### Requirement 1: Entry into the Builder

**User Story:** As a piano learner, I want a clear way to open the custom progression builder when
a key is selected, so that I can start assembling my own progression immediately.

#### Acceptance Criteria

1. WHEN a key is selected (`selected_key` is `Some`), THE App SHALL display a button or control
   to enter Custom_Progression_Builder mode.
2. WHEN no key is selected, THE entry control SHALL be disabled or hidden; the builder cannot be
   used without a key context.
3. WHEN the user enters Custom_Progression_Builder mode, the Working_Progression SHALL begin empty.
4. WHEN the user enters Custom_Progression_Builder mode, the seven Diatonic_Chord_Tiles for the
   current Selected_Key SHALL be displayed.
5. THE builder entry control SHALL be accessible from the progression panel area, consistent with
   the location of other progression controls.

---

### Requirement 2: Appending Chords via Plain Click

**User Story:** As a piano learner, I want to click a chord tile to add that chord to my
progression, so that I can build up a sequence one chord at a time.

#### Acceptance Criteria

1. WHEN the user clicks a Diatonic_Chord_Tile without holding Shift, THE App SHALL Append the
   corresponding `ScaleDegree` to the Working_Progression.
2. THE same chord tile MAY be clicked multiple times; each click Appends the degree again as a
   separate slot in the sequence.
3. AFTER each Append, THE Working_Progression display SHALL update immediately to show the new
   chord at the end.
4. THE App SHALL allow a Working_Progression of up to 16 chords; beyond that limit, clicks SHALL
   have no effect (the tile remains clickable but the Append is silently ignored).

---

### Requirement 3: Removing the Last Occurrence via Plain Click (Toggle Pop)

**User Story:** As a piano learner, I want to click a chord tile again to remove its last
occurrence from the progression, so that I can correct mistakes without resetting everything.

#### Acceptance Criteria

1. WHEN the user clicks a Diatonic_Chord_Tile without Shift AND the Working_Progression already
   contains at least one occurrence of that `ScaleDegree`, THE App SHALL remove the last occurrence
   of that degree from the Working_Progression (Pop).
2. WHEN the user clicks a Diatonic_Chord_Tile without Shift AND the Working_Progression does NOT
   contain that degree, THE App SHALL Append it (Requirement 2.1 applies).
3. AFTER a Pop, THE Working_Progression display SHALL update immediately.

---

### Requirement 4: Appending Additional Occurrences via Shift+Click

**User Story:** As a piano learner, I want to Shift+click a chord tile to always add that chord
to the progression regardless of whether it is already present, so that I can stack repeated chords
in sequence (e.g., I → V → I → IV).

#### Acceptance Criteria

1. WHEN the user Shift+clicks a Diatonic_Chord_Tile, THE App SHALL always Append the corresponding
   `ScaleDegree` to the Working_Progression, regardless of how many times it already appears.
2. Shift+click SHALL NOT remove any existing occurrence of the degree.
3. THE 16-chord limit from Requirement 2.4 SHALL also apply to Shift+click; beyond the limit,
   Shift+click SHALL have no effect.

---

### Requirement 5: Reset Button

**User Story:** As a piano learner, I want a Reset button that clears my progression so that I can
start over from scratch without leaving and re-entering the builder.

#### Acceptance Criteria

1. THE Custom_Progression_Builder SHALL display a Reset button at all times while the builder is
   active.
2. WHEN the user clicks Reset, THE Working_Progression SHALL be cleared to an empty list.
3. AFTER a Reset, the Diatonic_Chord_Tiles SHALL remain visible and the builder SHALL remain active
   so the user can immediately start a new progression.
4. WHEN the Working_Progression is already empty, the Reset button SHALL still be visible but MAY
   be visually disabled (non-destructive no-op).

---

### Requirement 6: Working Progression Display

**User Story:** As a piano learner, I want to see the chords I have added so far in order, so that
I can review my progression and understand what I am about to practise.

#### Acceptance Criteria

1. THE Custom_Progression_Builder SHALL display the Working_Progression as an ordered list of chord
   slots, with each slot showing the Roman numeral and chord name (e.g., "I – C", "V – G").
2. THE slots SHALL appear in the order they were appended (left-to-right or top-to-bottom).
3. WHEN the Working_Progression is empty, THE builder SHALL display a placeholder message (e.g.,
   "Click a chord below to start") instead of an empty list.
4. THE display SHALL update in real time after every Append, Pop, and Reset action.
5. Duplicate degrees at different positions SHALL be shown as distinct slots (e.g., I – V – I
   shows three separate slots).

---

### Requirement 7: Starting Play Along from the Builder

**User Story:** As a piano learner, I want a "Start Play Along" button that launches the play-along
mode with my custom progression, so that I can immediately practise what I built.

#### Acceptance Criteria

1. THE Custom_Progression_Builder SHALL display a "Start Play Along" button.
2. WHEN the Working_Progression contains at least one chord AND a MIDI_Device is connected, THE
   "Start Play Along" button SHALL be enabled.
3. WHEN the Working_Progression is empty OR no MIDI_Device is connected, THE "Start Play Along"
   button SHALL be disabled with a tooltip or label indicating why.
4. WHEN the user clicks "Start Play Along" (and preconditions in AC 2 are met), THE App SHALL enter
   Play_Along_Mode using the Working_Progression as the progression source.
5. THE Working_Progression SHALL be preserved after exiting Play_Along_Mode so the user can
   practise the same sequence again or modify it.
6. WHEN the user exits Play_Along_Mode (Stop button), THE App SHALL return to the
   Custom_Progression_Builder with the Working_Progression intact.

---

### Requirement 8: Exit from the Builder

**User Story:** As a piano learner, I want a way to close the builder and return to the normal
progression panel, so that I can switch back to predefined progressions whenever I want.

#### Acceptance Criteria

1. THE Custom_Progression_Builder SHALL display a "Back" or "Cancel" button.
2. WHEN the user clicks the exit control, THE App SHALL exit Custom_Progression_Builder mode and
   return to the normal progression view (the predefined progression list).
3. WHEN the user exits the builder, THE Working_Progression SHALL be discarded.
4. Entering the builder again SHALL start with an empty Working_Progression (Requirement 1.3).
