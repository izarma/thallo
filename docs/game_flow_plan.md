# Game Flow & Narrative-Time Plan

> Narrative time is addressable through a single `StoryBeat` value and a single
> `apply_beat` reset path. Game events, Retry, and dev jumps all land in the same
> place, so every beat is deterministic.

---

## 1. `StoryBeat` — the single source of truth

Defined in `src/engine/scripted_events.rs`:

```rust
pub enum StoryBeat {
    Act1Intro,      // desktop + chat + Act 1 file puzzle
    Act1Connecting, // [SECURE] folder injected, find omega
    Act2Sos,        // netripper + SOS countdown
    Win,            // terminal win state
    Lose,           // terminal lose state
}
```

`StoryProgress` owns `beat: StoryBeat`. The old booleans (`network_reconnected`,
`act`) are gone; `is_act2()` and `is_connecting_sunday()` are derived from the
enum.

---

## 2. `apply_beat` — the single reset path

`src/game/beats.rs::apply_beat(world, beat)` rebuilds every piece of state that
depends on narrative position:

1. **Story flags + program unlocks** — `UnlockState::apply_beat(beat)` resets
   `StoryProgress` and sets `ProgramsUnlocked` to the canonical unlocks for the
   beat.
2. **Filesystem** — `build_fs_for_beat(beat, ...)` in `src/game/files.rs`
   returns a fresh `FsHierarchy`:
   - `Act1Intro`: full Act 1 desktop, `Personal` locked, `[CORRUPTED]` encrypted.
   - `Act1Connecting`: Act 1 desktop with `Personal` unlocked,
     `[CORRUPTED]` decrypted, and `[SECURE]` injected.
   - `Act2Sos`: desktop stripped to `[SECURE]`, already decrypted.
   - `Win` / `Lose`: fully-progressed Act 1 filesystem (desktop is never shown).
3. **Dialogues + runner** — fresh `Dialogues` and a cleared `DialogueRunner`.
4. **File dialogue triggers** — `FileDialogueTriggers::clear()` wipes
   `triggers`, `opened`, `pending`, and the `delay` timer, then the beat's
   triggers are rebuilt. Triggers evaluate only after the chat has been idle
   for `FILE_TRIGGER_DELAY_SECONDS`.
5. **Act 2 timer** — active only for `Act2Sos`, reset to `ACT2_TIME_LIMIT`.
6. **Transient UI / game state** — `OpenWindows`, `OpenAlerts`, `ActiveMinigame`,
   and `Pause` are all reset.

Systems queue it via `ApplyBeatCommand(beat)` so they do not need `&mut World`.

---

## 3. Flow chart

```mermaid
flowchart TD
    APP[App start] --> TITLE[Screen: Title]
    TITLE --> STARTUP[Menu: Startup]

    STARTUP -->|No| SHUTDOWN[Menu: ShutDown]
    SHUTDOWN -->|5s| EXIT[App exit]
    STARTUP -->|Yes| LOADING[Screen: Loading]

    LOADING -->|gif done + assets| DESKTOP[Screen: Desktop]
    DESKTOP -->|OnEnter| APPLY[apply_beatcurrent beat]

    APPLY -->|Act1Intro| A1[Act 1: file puzzle]
    A1 -->|10s delayed event| OPENCHAT[OpenChat -> unlock chat]
    A1 -->|open incident_report.txt| D1[Chat exchange]
    A1 -->|open collective_complaint + notes + response| D2[Chat exchange]
    D2 -->|on_complete FileTransfer| BRUTE[programs.bruteforce = true]

    BRUTE -->|bruteforce [CORRUPTED], minigame win| CORR[Decrypt CORRUPTED folder]
    CORR -->|open behavioral_notes_thallo.txt| D3[Chat -> on_complete BeginReboot]

    D3 -->|NetworkConnect| AB1[Screen: ActBreak -> SUNDAY NETWORK]
    AB1 -->|Connect| LOADING
    LOADING -->|Desktop OnEnter apply_beatAct1Connecting| A1B[Find omega]

    A1B -->|open [UNIDENTIFIED] + omega.png| AB2[Screen: ActBreak -> ACT II]
    AB2 -->|Power On| LOADING
    LOADING -->|Desktop OnEnter apply_beatAct2Sos| A2[Act 2: SOS countdown]

    A2 -->|act2 chat line 1 on_complete| NR[programs.netripper = true]
    NR -->|netripper [SECURE], minigame wins| SECURE[secure_transmitted = true, chat]
    SECURE -->|netripper sos, minigame wins| SOS[TransmitSOS -> final chat -> Win]

    SOS --> WIN[apply_beatWin -> GameOverEvent::Win]
    WIN --> WINMENU[Screen: GameOver -> Menu: Win]
    WINMENU --> QUIT1[Quit]

    A2 -->|netripper minigame fail x3| RIPFAIL[apply_beatLose -> GameOverEvent::Lose]
    A2 -->|5min timer hits 0| TIMEOUT[apply_beatLose -> GameOverEvent::Lose]
    RIPFAIL --> LOSE[Screen: GameOver -> Menu: Lose]
    TIMEOUT --> LOSE
    LOSE -->|Retry| LOADING
    LOSE -->|Quit| QUIT2[Quit]
```

---

## 4. What triggers what

| Trigger | Handler | Result |
|---|---|---|
| 10 s after Desktop `OnEnter` | `effect_open_chat` | `programs.chat = true`, chatbox opened |
| Open `incident_report.txt` (`Any`) | `FileDialogueTriggers` (5s after chat idle) | chat lines |
| Open `collective_complaint` + `notes` + `response` (`All`) | `FileDialogueTriggers` (5s after chat idle) | chat lines → `ChatTrigger(FileTransfer(BruteForce))` |
| File-transfer alert finishes (`BruteForce`) | `ScriptedEventTrigger::FileTransferComplete` | `programs.bruteforce = true` |
| `bruteforce [CORRUPTED]` + minigame win | minigame system | decrypts `[CORRUPTED]` folder in VFS |
| Open `behavioral_notes_thallo.txt` (`All`) | `FileDialogueTriggers` (5s after chat idle) | `BeginReboot(NetworkConnect)` |
| `BeginReboot(NetworkConnect)` | `effect_reboot` | `apply_beat(Act1Connecting)`, `Screen::ActBreak` |
| Open `[UNIDENTIFIED]` + `omega.png` (`All`) | `FileDialogueTriggers` (5s after chat idle) | `BeginReboot(ActTrans)` |
| `BeginReboot(ActTrans)` | `effect_reboot` | `apply_beat(Act2Sos)`, `Screen::ActBreak` |
| Act 2 chat line 1 (`on_complete`) | `DialogueLine` | `ChatTrigger(FileTransfer(NetRipper))` |
| File-transfer alert finishes (`NetRipper`) | `ScriptedEventTrigger::FileTransferComplete` | `programs.netripper = true` |
| `netripper [SECURE]` + minigame wins | minigame system | `secure_transmitted = true`, adds chat lines |
| `netripper sos` + minigame wins | minigame system | `TransmitSOS` → `Win` |
| NetRipper minigame fail ×3 | minigame system | `RipperFailed` → `apply_beat(Lose)` |
| Act 2 timer reaches 0 | `tick_act2_timer` | `apply_beat(Lose)` |
| **Retry** in lose menu | `lose_menu` | `apply_beat(Act2Sos)` → `Screen::Loading` |
| **Dev panel button** | `beat_jump_panel` | `apply_beat(beat)` + `Screen::Desktop` (or win/lose trigger) |

---

## 5. Key files and types

| File | Role |
|---|---|
| `src/engine/scripted_events.rs` | `StoryBeat`, `StoryProgress`, `UnlockState`, `ScriptedEventTrigger`, `FileDialogueTriggers` (now defers evaluation until `DialogueRunner` is idle). |
| `src/game/beats.rs` | `apply_beat(world, beat)`, `ApplyBeatCommand`, per-beat dialogue/trigger dispatch. |
| `src/game/files.rs` | `build_fs_for_beat(beat)` and the canonical Act 1 / `[SECURE]` / Act 2 builders. |
| `src/game/mod.rs` | `setup_desktop_for_beat`: single `OnEnter(Desktop)` system that queues `ApplyBeatCommand`. |
| `src/game/dialogues.rs` | `build_act1_dialogues`, `build_act2_dialogues`, `build_act1_file_triggers`, `build_netconn_file_triggers`. |
| `src/ui/menus/startup.rs` | Single act-break title card (`act_break_title`) + break SFX spawn. |
| `src/ui/menus/lose.rs` | Retry calls `apply_beat(Act2Sos)` and skips the title card. |
| `src/ui/popups.rs` | File-transfer completion now triggers `ScriptedEventTrigger::FileTransferComplete` instead of mutating `UnlockState` directly. |
| `src/devtools.rs` | egui story-beat jump panel and F5/F6/F9/F10 hotkeys. |
