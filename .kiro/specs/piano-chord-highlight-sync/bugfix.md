# Bugfix Requirements Document

## Introduction

When a chord progression is selected, the piano component's highlighted keys do not update automatically as the active chord changes. The piano only reflects the correct chord highlight when the user manually navigates through the progression using the prev/next arrow buttons. This means the visual piano feedback is decoupled from the active chord state, making the progression selection feature misleading and incomplete.

## Bug Analysis

### Current Behavior (Defect)

1.1 WHEN a chord progression is selected THEN the system highlights only the first chord on the piano and does not update the highlight as the active chord changes over time

1.2 WHEN the active chord index in `active_progression` changes (e.g. via `SelectProgression` setting index 0, or audio playback advancing) THEN the system does not re-render the piano with the corresponding `highlighted_chord` for the new index

1.3 WHEN a progression is selected and the audio plays through its chords sequentially THEN the system keeps the piano frozen on the first chord's highlight instead of tracking the currently sounding chord

### Expected Behavior (Correct)

2.1 WHEN a chord progression is selected THEN the system SHALL immediately highlight the first chord's keys on the piano and update the highlight each time the active chord index changes

2.2 WHEN the active chord index in `active_progression` changes THEN the system SHALL update `highlighted_chord` in state to reflect the chord at the new index, causing the piano to re-render with the correct root, third, and fifth highlighted

2.3 WHEN a progression is selected and audio plays through its chords THEN the system SHALL keep the piano highlight synchronized with the chord currently being played

### Unchanged Behavior (Regression Prevention)

3.1 WHEN the user clicks the next (▶) or previous (◀) arrow buttons THEN the system SHALL CONTINUE TO advance the active chord index and update the piano highlight correctly

3.2 WHEN a single diatonic chord is clicked in the key info panel THEN the system SHALL CONTINUE TO highlight that chord's keys on the piano independently of any active progression

3.3 WHEN no progression is active THEN the system SHALL CONTINUE TO show scale note highlights (or no highlights) on the piano based on the selected key alone

3.4 WHEN a different key is selected from the circle THEN the system SHALL CONTINUE TO clear the active progression and highlighted chord

---

## Bug Condition

```pascal
FUNCTION isBugCondition(X)
  INPUT: X of type AppState
  OUTPUT: boolean

  // Bug is triggered when a progression is active but the piano highlight
  // does not match the chord at the current index
  RETURN X.active_progression IS SOME
    AND X.highlighted_chord DOES NOT MATCH chord_at(X.active_progression.id, X.active_progression.current_index)
END FUNCTION
```

```pascal
// Property: Fix Checking
FOR ALL X WHERE isBugCondition(X) DO
  result ← render_piano(X)
  ASSERT result.highlighted_chord = chord_at(X.active_progression.id, X.active_progression.current_index)
END FOR

// Property: Preservation Checking
FOR ALL X WHERE NOT isBugCondition(X) DO
  ASSERT F(X).highlighted_chord = F'(X).highlighted_chord
END FOR
```
