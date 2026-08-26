---
name: bevy-events-messages
description: >-
  Guides agents on when to use Events (triggered/observed via observers) vs Messages (buffered/written/read)
  in the Bevy game engine (0.17+). Use this skill whenever the user is writing Bevy Rust code that involves
  inter-system communication, reacting to state changes, sending data between systems, or anything
  related to Bevy's Event, EntityEvent, Message, Observer, On, Trigger, MessageWriter, MessageReader,
  world.trigger(), world.add_observer(), component lifecycle events (Add, Insert, Remove, Replace),
  or migrating from Bevy 0.16 Event/EventWriter/EventReader patterns. Also triggers on questions about
  Bevy event propagation, entity-targeted events, or the Observer pattern in Bevy.
---

# Bevy Events vs Messages (0.17+)

In Bevy 0.17, the old unified `Event` trait was split into two separate concepts. Getting this wrong leads to code that either fights the type system, performs unnecessarily, or simply doesn't compile. This reference covers every major variation of both patterns with real examples so you can confidently pick the right one.

## The Fundamental Distinction

**Event** = "something happened, react right now, synchronously, in this exact call stack."
**Message** = "here's some data, buffer it, and a system will process it later in the schedule."

The practical difference: when you call `world.trigger(my_event)`, every observer for that event type runs **immediately** before `trigger()` returns. When you call `writer.write(my_message)`, the message sits in a buffer and nothing happens until a system with a `MessageReader` runs later in the frame (or next frame).

## Events: Complete Reference

Events are for immediate, synchronous, observer-driven reactions. Every Event type defines an associated `Trigger` implementation that controls which observers fire and in what order.

### 1. Global Event — untargeted broadcast

Use when: something happened that isn't about any particular entity, and you want all observers to react immediately.

```rust
#[derive(Event)]
struct LevelCompleted {
    level: u32,
    score: u32,
}

// Trigger — all global observers run synchronously right here
world.trigger(LevelCompleted { level: 3, score: 5000 });

// Register a global observer
app.add_observer(|completed: On<LevelCompleted>| {
    info!("Level {} completed with score {}!", completed.level, completed.score);
});
```

### 2. EntityEvent — targeted at a specific entity

Use when: the event is about a specific entity, and you want observers on that entity (and possibly global observers) to react.

```rust
#[derive(EntityEvent)]
struct Damage {
    entity: Entity,
    amount: u32,
}

// Trigger for a specific entity
world.trigger(Damage { entity: enemy, amount: 25 });

// Global observer — runs for ALL Damage events regardless of target
app.add_observer(|damage: On<Damage>| {
    info!("{} took {} damage", damage.entity, damage.amount);
});

// Entity-scoped observer — only runs for Damage targeting THIS entity
app.entity_mut(specific_enemy).observe(|damage: On<Damage>| {
    info!("I'm the one being hit!");
});
```

The `entity` field name is the default target. If you want a different field name, use `#[event_target]`:

```rust
#[derive(EntityEvent)]
struct Attack {
    #[event_target]
    attacker: Entity,
    target: Entity,
}

// Observers on `attacker` will fire. `target` is just data.
```

### 3. Propagating EntityEvent — bubbles up hierarchy

Use when: the event should bubble from a child entity up through parent entities (following `ChildOf` relationships). UI click handling is the canonical example.

```rust
#[derive(EntityEvent)]
#[entity_event(propagate)]
struct Click {
    entity: Entity,
}

// Observer on the clicked entity
app.entity_mut(button).observe(|click: On<Click>| {
    info!("Button was clicked!");
    // Could stop propagation here:
    // click.propagate(false);
});

// Observer on a parent layout — catches clicks from any child button
app.entity_mut(layout_root).observe(|click: On<Click>| {
    info!("A descendant was clicked: {}", click.entity);
});
```

### 4. Component lifecycle events

Bevy 0.17 has built-in EntityEvents for component lifecycle. The old names (`OnAdd`, `OnInsert`, `OnRemove`, `OnReplace`) are now `Add`, `Insert`, `Remove`, `Replace`.

```rust
// Fires when a Player component is added to any entity
app.add_observer(|add: On<Add, Player>| {
    info!("Player spawned: {}", add.entity);
});

// Fires when Health is removed (entity died, component cleaned up, etc.)
app.add_observer(|remove: On<Remove, Health>| {
    info!("Entity {} lost its Health component", remove.entity);
});

// Insert fires on both initial add and subsequent re-inserts
app.add_observer(|insert: On<Insert, Visibility>| {
    // Runs when Visibility is first added AND when it's re-inserted
});
```

### 5. Triggering from within a system (via Commands)

```rust
fn check_game_over(
    query: Query<&Health>,
    mut commands: Commands,
) {
    for (entity, health) in query.iter() {
        if health.value == 0 {
            commands.trigger(PlayerDied { entity, score: 100 });
        }
    }
}

// Note: commands.trigger() defers to the next sync point.
// For truly immediate triggers within a system, use world.trigger()
// via a callback or a one-shot system.
```

### 6. Multiple observers, execution order

All observers for a given event type run when `trigger()` is called. If you need ordering, use observer descriptors:

```rust
app.add_observer(|a: On<Explosion>| {
    // This runs first (lower priority number = earlier)
})
.with_run_condition(RunCondition::always);

// Or use system ordering on observers registered as systems
app.add_observer(
    |b: On<Explosion>| { /* runs second */ }
).after(/* first observer */);
```

### 7. Accessing world data inside an observer

Observers are systems — they can take any normal system parameters:

```rust
app.add_observer(
    |mut spawn: On<SpawnExplosion>,
     mut commands: Commands,
     assets: Res<AssetServer>,
     query: Query<&Transform>|
    {
        let transform = query.get(spawn.entity).unwrap();
        commands.spawn((
            ParticleBundle {
                mesh: assets.load("explosion.glb#Mesh0"),
                transform: *transform,
                ..default()
            },
            AutoDespawn { timer: Timer::from_secs(1.0) },
        ));
    }
);
````

## Messages: Complete Reference

Messages are for buffered, asynchronous, batch-processed data. They are the direct successors to Bevy 0.16's `EventWriter`/`EventReader` pattern (which used the old `Event` trait for both purposes).

### 1. Basic message write and read

```rust
#[derive(Message)]
struct InputAction {
    action: ActionType,
    timestamp: f64,
}

fn collect_inputs(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<InputAction>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        writer.write(InputAction {
            action: ActionType::Jump,
            timestamp: 0.0,
        });
    }
}

fn process_inputs(mut reader: MessageReader<InputAction>) {
    for action in reader.read() {
        match action.action {
            ActionType::Jump => { /* handle jump */ }
            ActionType::Shoot => { /* handle shoot */ }
        }
    }
}
```

### 2. Multiple independent readers

Each `MessageReader` has its own cursor, so multiple systems can read the same messages without interfering:

```rust
fn play_sound_effects(mut reader: MessageReader<DamageOccurred>) {
    for msg in reader.read() {
        // Play hit sound for every damage event
    }
}

fn update_health_bar(mut reader: MessageReader<DamageOccurred>) {
    for msg in reader.read() {
        // Update UI health bar for every damage event
    }
}
// Both systems see the same set of messages independently.
```

### 3. Writing via World or Commands

```rust
// Directly on the world
world.write_message(NetworkPacket { data: bytes });

// Via commands (deferred to next sync point)
commands.write_message(NetworkPacket { data: bytes });

// Batch write
world.write_message_batch(vec![
    InputAction { action: ActionType::Jump, timestamp: 0.0 },
    InputAction { action: ActionType::Shoot, timestamp: 0.1 },
]);
```

### 4. The 2-frame read window

Messages use a two-frame window. When you call `reader.read()`, you get messages from both the current frame and the previous frame. This prevents messages from being missed if they're written late in a frame after a reader has already consumed. Messages are automatically cleaned up after two frames.

This means you should NOT use messages when you need guaranteed once-only processing. If a message must be processed exactly once, use Events/Observers instead (they run synchronously, so there's no window for duplicates).

### 5. Reading without consuming (peeking)

If you need to inspect messages without advancing the reader's cursor:

```rust
fn debug_log(mut reader: MessageReader<InputAction>) {
    for action in reader.read() {
        info!("Input: {:?}", action);
    }
    // After this, another reader in a later system can still read these same messages.
}
```

## Decision Guide

### Ask these questions in order:

**Q1: Does the consumer need to react immediately, in the same call stack as the producer?**
- Yes → Event. The reaction must happen before the trigger call returns.
- No → Message (or just system ordering / Resources).

**Q2: Is the communication about a specific entity?**
- Yes, and it should only trigger on that entity's observers → `EntityEvent`
- Yes, and it should bubble up a hierarchy → `EntityEvent` + `#[entity_event(propagate)]`
- No, it's a global notification → plain `Event`

**Q3: Do multiple things happen per frame that should be batch-processed?**
- Yes → `Message` (accumulate, then drain in a single system)
- No, each one needs its own immediate reaction → `Event` (trigger individually)

**Q4: Could a Resource or direct system ordering handle this instead?**
- If two systems just need to share state and they run in a known order → `Res<Resource>` or `Local<T>` with `before`/`after`
- If the data needs to persist across frames as a single value → `Res<Resource>`

### Real-world decision examples:

| Scenario | Choice | Why |
|----------|--------|-----|
| Player dies → game over screen | Event | Immediate reaction needed |
| Collecting keyboard inputs per frame | Message | Accumulate then process in batch |
| Enemy takes damage → flash red | EntityEvent | Immediate, entity-scoped reaction |
| UI button click → update parent layout | EntityEvent + propagate | Immediate, entity-scoped, hierarchy |
| Network packets arrive async | Message | Buffer them, process in dedicated system |
| Component added → play spawn sound | On<Add, T> | Lifecycle hook, immediate |
| Score changes → update 3 different UI elements | Message | Multiple independent consumers, batched |
| Ability used → cooldown manager notified | Event | Immediate so cooldown starts same frame |
| Collision detected → physics response | Message | Accumulate collisions, resolve together |
| Chat message received → show in UI now | Event (triggered from within message processor) | UI update must be immediate, but the packet buffering is a Message |

## Combining Both Patterns

Some systems genuinely need both. The typical pattern is a Message-based "transport" layer that feeds into Event-based "reaction" triggers:

```rust
#[derive(Message)]
struct IncomingNetworkPacket {
    payload: Vec<u8>,
}

#[derive(Event)]
struct ChatMessageReceived {
    from: String,
    text: String,
}

// The message processor reads buffered packets and triggers immediate events
fn process_network_packets(
    mut packet_reader: MessageReader<IncomingNetworkPacket>,
    world: &World,
) {
    for packet in packet_reader.read() {
        if let Some(chat) = parse_chat(packet) {
            // Immediate reaction — UI updates synchronously
            world.trigger(ChatMessageReceived { from: chat.from, text: chat.text });
        }
    }
}

// This observer fires immediately when trigger() is called above
app.add_observer(|chat: On<ChatMessageReceived>| {
    // Update UI right now, in this same call stack
});
```

You can also derive both on the same type, but this is rare and should only be done when the type semantically belongs to both worlds:

```rust
#[derive(Event, Message)]
struct DebugLog {
    message: String,
}
// Can be world.trigger()'d for immediate logging AND buffered for a debug overlay system
```

## Migration from Bevy 0.16

| Bevy 0.16 | Bevy 0.17+ |
|-----------|------------|
| `#[derive(Event)]` (for buffered events) | `#[derive(Message)]` |
| `#[derive(Event)]` (for observers) | `#[derive(Event)]` (same) |
| `EventWriter<E>` | `MessageWriter<M>` |
| `EventReader<E>` | `MessageReader<M>` |
| `Events<E>` (resource) | `Messages<M>` (resource) |
| `events.send(e)` | `writer.write(m)` |
| `world.send_event(e)` | `world.write_message(m)` |
| `commands.send_event(e)` | `commands.write_message(m)` |
| `Trigger<E>` (observer param) | `On<E>` (observer param) |
| `Trigger<OnAdd, T>` | `On<Add, T>` |
| `world.trigger_targets(e, entity)` | `world.trigger(EntityEvent { entity, .. })` |
| `OnAdd` / `OnInsert` / `OnRemove` | `Add` / `Insert` / `Remove` |

## Quick Reference

| Aspect | Event (Observer) | Message (Buffered) |
|--------|-------------------|---------------------|
| Trigger | `world.trigger(e)` / `commands.trigger(e)` | `MessageWriter::write(m)` / `world.write_message(m)` |
| Receive | `On<E>` in observer callback | `MessageReader<M>::read()` in system |
| Timing | Synchronous, in the same call stack | Deferred, consumed when reader's system runs |
| Targeting | Global or entity-scoped (`EntityEvent`) | Always global |
| Propagation | Yes, via `#[entity_event(propagate)]` | No |
| Multiple consumers | Each observer fires per trigger | Each reader has independent cursor |
| Buffering | None — runs once per trigger | 2-frame window, auto-cleanup |
| Derive macro | `#[derive(Event)]` / `#[derive(EntityEvent)]` | `#[derive(Message)]` |
| Best for | Immediate reactions, entity hooks, hierarchy | Input collection, network packets, batch processing |
