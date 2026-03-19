# Requirements Document

## Introduction

Circle of Fifths is a browser-based web application targeting piano learners and music producers who want a clean, ad-free tool for exploring music theory through the circle of fifths. The frontend is implemented in Rust compiled to WebAssembly using the Yew framework, bundled and served via Trunk. The app is fully static with no backend — all data is hardcoded or computed in Rust/WASM and persistence is handled via the browser's localStorage API. Core features include an interactive diagram, diatonic chord display, chord progression recommendations, a piano keyboard visualizer, quiz mode, and audio playback.

## Glossary

- **App**: The Circle of Fifths web application.
- **Circle**: The interactive circle of fifths SVG diagram.
- **Key**: A musical key (e.g. C major, A minor).
- **Major_Key**: A major key occupying the outer ring of the Circle.
- **Minor_Key**: A relative minor key occupying the inner ring of the Circle.
- **Segment**: A clickable arc section of the Circle representing one Key.
- **Key_Signature**: The set of sharps or flats associated with a Key.
- **Diatonic_Chord**: A chord built from the notes of a given Key using scale degrees I through VII.
- **Roman_Numeral**: A Roman numeral label (I, ii, iii, IV, V, vi, vii°) denoting a chord's scale degree and quality.
- **Chord_Quality**: The classification of a chord as major, minor, or diminished.
- **Progression**: An ordered sequence of Diatonic_Chords identified by Roman_Numerals.
- **Progression_Tag**: A mood or genre label attached to a Progression (e.g. "melancholic", "uplifting", "jazz", "pop").
- **Borrowed_Chord**: A chord taken from a parallel or neighboring key (modal interchange).
- **Piano_Panel**: The scrollable piano keyboard UI component displayed below the Circle.
- **Key_Role**: The harmonic role of a note within a chord: root, third, or fifth.
- **Quiz**: A flashcard-style question-and-answer session testing music theory knowledge.
- **Session**: A single continuous Quiz run from start to completion or abandonment.
- **Audio_Engine**: The WebAssembly audio synthesis component responsible for sound playback.
- **Theme**: The visual color scheme of the App, either dark or light.

---

## Requirements

### Requirement 1: Interactive Circle of Fifths Diagram

**User Story:** As a piano learner, I want an interactive circle of fifths diagram, so that I can visually explore key relationships and navigate the app.

#### Acceptance Criteria

1. THE App SHALL render the Circle as an SVG diagram with 12 outer Segments for Major_Keys and 12 inner Segments for their relative Minor_Keys.
2. WHEN a user clicks a Segment, THE App SHALL mark that Segment as selected and update all dependent panels to reflect the chosen Key.
3. WHILE a Segment is selected, THE Circle SHALL display that Segment in a visually distinct highlighted state.
4. THE Circle SHALL display the Key_Signature accidental count (number of sharps or flats) on each Segment.
5. WHILE a Segment is selected, THE Circle SHALL highlight the two adjacent Segments as closely related keys and visually distinguish the opposite Segment as the most distant key.
6. WHEN a user clicks an already-selected Segment, THE App SHALL deselect it and return all dependent panels to their default state.

---

### Requirement 2: Key Info Panel

**User Story:** As a piano learner, I want to see detailed information about a selected key, so that I can understand its notes, chords, and key signature at a glance.

#### Acceptance Criteria

1. WHILE a Key is selected, THE App SHALL display a Key Info Panel showing the Key name, Key_Signature (sharps or flats count and names), and the seven scale notes in order.
2. WHILE a Key is selected, THE App SHALL display all seven Diatonic_Chords for that Key, each labeled with its Roman_Numeral and Chord_Quality.
3. THE App SHALL display each Diatonic_Chord with both its Roman_Numeral (e.g. "vi") and its full chord name (e.g. "Am").
4. IF no Key is selected, THEN THE App SHALL display a placeholder prompt instructing the user to select a key from the Circle.

---

### Requirement 3: Diatonic Chords Display

**User Story:** As a piano learner, I want to see the diatonic chords for any key, so that I can understand which chords naturally belong together.

#### Acceptance Criteria

1. WHILE a Key is selected, THE App SHALL compute and display the seven Diatonic_Chords using the major scale formula (W-W-H-W-W-W-H).
2. THE App SHALL label each Diatonic_Chord with its Roman_Numeral using uppercase for major chords (I, IV, V), lowercase for minor chords (ii, iii, vi), and lowercase with a degree symbol for diminished chords (vii°).
3. WHEN a user clicks a Diatonic_Chord in the panel, THE App SHALL highlight the corresponding notes on the Piano_Panel.

---

### Requirement 4: Chord Progression Recommender

**User Story:** As a music producer learning piano, I want curated chord progression suggestions for a selected key, so that I can find progressions that match the mood or genre I'm going for.

#### Acceptance Criteria

1. WHILE a Key is selected, THE App SHALL display a list of curated Progressions for that Key, each labeled with at least one Progression_Tag.
2. THE App SHALL display each Progression showing both Roman_Numeral notation and resolved chord names (e.g. "I - V - vi - IV = C - G - Am - F").
3. WHEN a user clicks a Progression, THE App SHALL set it as the active Progression and update the Piano_Panel to show the first chord.
4. WHILE a Progression is active, THE App SHALL provide next and previous controls to cycle through the chords in the Progression.
5. THE App SHALL allow a user to mark a Progression as a favorite and persist favorites across App sessions.
6. THE App SHALL include at least one Progression per Key that demonstrates a Borrowed_Chord, with a label identifying the source key of the Borrowed_Chord.
7. THE App SHALL provide at least 4 Progressions per Key covering at least 3 distinct Progression_Tags.

---

### Requirement 5: Piano Keyboard Panel

**User Story:** As a piano learner, I want a visual piano keyboard that highlights notes and chords, so that I can see exactly which keys to press.

#### Acceptance Criteria

1. THE App SHALL render the Piano_Panel as a scrollable horizontal piano keyboard spanning at least 3 octaves.
2. WHILE a Key is selected, THE Piano_Panel SHALL highlight all notes belonging to that Key's scale.
3. WHEN a user selects a Diatonic_Chord or activates a Progression chord, THE Piano_Panel SHALL highlight the notes of that chord, color-coded by Key_Role: root (one color), third (a second color), fifth (a third color).
4. WHILE a Progression is animating, THE Piano_Panel SHALL highlight each chord's notes in sequence, advancing on a fixed interval of 1 second per chord.
5. THE App SHALL provide a toggle control to show or hide note name labels on Piano_Panel keys.
6. THE App SHALL provide an octave range selector allowing the user to shift the visible octave range up or down by 1 octave.
7. IF a chord contains notes outside the currently visible octave range, THEN THE Piano_Panel SHALL scroll automatically to bring those notes into view.

---

### Requirement 6: Quiz Mode

**User Story:** As a piano learner, I want flashcard-style quizzes on music theory, so that I can test and reinforce my knowledge of the circle of fifths.

#### Acceptance Criteria

1. THE App SHALL provide a Quiz Mode accessible from the main navigation.
2. THE App SHALL include quiz questions of at least the following three types:
   a. Given a Major_Key, identify the number of sharps or flats in its Key_Signature.
   b. Given a Major_Key, identify its relative Minor_Key.
   c. Given a Key, name all seven scale notes in order.
3. WHEN a Quiz Session starts, THE App SHALL present questions in a randomized order drawn from the full question pool.
4. WHEN a user submits an answer, THE App SHALL immediately display whether the answer is correct or incorrect, and reveal the correct answer if the submission was incorrect.
5. THE App SHALL track the number of correct answers and total questions answered within a Session and display the running score.
6. WHEN a Session ends, THE App SHALL display a summary showing the final score and the percentage of correct answers.
7. THE App SHALL persist the best score per question type across Sessions.

---

### Requirement 7: Audio Playback

**User Story:** As a piano learner, I want to hear chords and progressions played back, so that I can train my ear alongside the visual learning.

#### Acceptance Criteria

1. WHEN a user activates audio playback for a selected Key, THE Audio_Engine SHALL play all scale notes of that Key in ascending order, one note per 300ms.
2. WHEN a user activates audio playback for a selected chord, THE Audio_Engine SHALL play all notes of that chord simultaneously.
3. WHEN a user activates audio playback for an active Progression, THE Audio_Engine SHALL play each chord in sequence, holding each chord for 1 second before advancing.
4. WHILE audio is playing, THE Piano_Panel SHALL animate in sync with the Audio_Engine, highlighting the notes being played at each moment.
5. IF the Audio_Engine fails to initialize, THEN THE App SHALL display an error message and continue to function without audio.
6. THE App SHALL provide a playback control to stop audio at any time.
7. THE App SHALL provide a mute control that silences all audio output without stopping playback state, and an unmute control that restores audio output.
8. THE App SHALL persist the mute state across browser sessions so that the Audio_Engine initializes in the user's last-set mute state on subsequent visits.

---

### Requirement 8: Theme and Visual Design

**User Story:** As a user, I want a clean, minimal interface with dark and light mode support, so that I can use the app comfortably in any lighting environment.

#### Acceptance Criteria

1. THE App SHALL support a dark Theme and a light Theme.
2. THE App SHALL provide a toggle control to switch between dark and light Themes.
3. THE App SHALL persist the user's Theme preference across App sessions.
4. THE App SHALL apply the selected Theme to all UI components including the Circle, Key Info Panel, Piano_Panel, and Quiz Mode.
5. WHEN a user zooms into a selected Key on the Circle, THE App SHALL display an enlarged detail view of that Segment showing the Key name, Key_Signature, and scale notes.
6. THE App SHALL render all UI components using a minimal design with no advertisements or third-party promotional content.
