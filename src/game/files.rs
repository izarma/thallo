use bevy_egui::egui::{self, TextureId};

use crate::engine::file_system::{DESKTOP_PATH, FsHierarchy, FsNode, FsPath, HOME_PATH};
use crate::engine::scripted_events::StoryBeat;

/// Build the canonical filesystem for a narrative beat.
///
/// - [`StoryBeat::Act1Intro`] — the full Act 1 desktop.
/// - [`StoryBeat::Act1Connecting`] — Act 1 desktop plus the injected `[SECURE]` folder.
/// - [`StoryBeat::Act2Sos`] — desktop stripped to a *decrypted* `[SECURE]` folder.
/// - Terminal beats fall back to the Act 1 filesystem (the desktop is never shown).
pub(crate) fn build_fs_for_beat(
    beat: StoryBeat,
    omega_tex: Option<TextureId>,
    size: egui::Vec2,
) -> FsHierarchy {
    match beat {
        // Fresh Act 1: `Personal` and `[CORRUPTED]` start locked.
        StoryBeat::Act1Intro => build_fs(false),
        // Past the intro, the player has already unlocked `Personal` and
        // decrypted `[CORRUPTED]`, so those folders stay open when the
        // `[SECURE]` folder is injected.
        StoryBeat::Act1Connecting => {
            let mut vfs = build_fs(true);
            if let Some(tex) = omega_tex {
                inject_secure_folder(&mut vfs, tex, size);
            }
            vfs
        }
        StoryBeat::Act2Sos => build_secure_only_fs(omega_tex, size),
        // Terminal beats are never shown on the desktop, but keep a fully
        // progressed Act 1 filesystem for consistency.
        StoryBeat::Win | StoryBeat::Lose => build_fs(true),
    }
}

/// Build the Act 1 desktop.
///
/// `act1_unlocked` is `true` once the player has progressed past the intro
/// (i.e. solved the `Personal` password and decrypted `[CORRUPTED]`), so those
/// folders are rebuilt without their locks instead of being relocked.
fn build_fs(act1_unlocked: bool) -> FsHierarchy {
    let mut desktop = FsNode::folder("Desktop");
    desktop
        .push_child(FsNode::text_file(
            "inventory_list.txt",
            "Item – Quantity – Expiration - Responsible\nNutrient medium, type A; 47 units; 09.2190; Petrov\nSeeds, wheat (control); 3 packs; indefinite; Lefevre\nCharcoal filters; 12 pcs; (yet to decide); Dupont\nLatex gloves; box; Indefinite; Lefevre\nCitric acid; 2 kg; 2.3 kg; 12.2189; Petrov\nLogbooks (blank); 14 pcs; (yet to decide); Lefevre",
        ))
        .ok();
    desktop
        .push_child(FsNode::text_file(
            "maintenance_schedule_agro.txt",
            "07.03.2189 — Weekly irrigation system calibration.\n08.03.2189 — Filter replacement in reservoir #3 (completed).\n09.03.2189 — Scheduled leak check of inter-sector airlocks (joint with engineering department).\n10.03.2189 — Unscheduled: complaint about burnt wiring smell at bay 7B. Cause not found.\nRecommended a follow-up if recurring.",
        ))
        .ok();
    desktop
        .push_child(FsNode::text_file(
            "incident_report.txt",
            "INCIDENT REPORT\nSector: Agricultural (7A)\nCompiled by: Sr. Technician P. Lefevre\nDate: 14.03.2189\nIncident №: 7714\n\nSummary: At 22:40 ship time during routine check of sector 7B, an extraneous sound (low-frequency hum) was detected. Source could not be visually or audibly identified. Sound lasted approximately 45 seconds, then ceased.\n\nActions taken:\n1. Visual inspection of sector – negative, yielded no results.\n2. Check of communications and ventilation systems — no deviations.\n3. Inquiry to adjacent sectors (7B, 7C) — no similar complaints received.\n\nConclusion: Presumed acoustic defect in ventilation system or an unexpected error in reporting party's auditory equipment. Incident dismissed.\n\nAttachment: audio recording of attempted hum capture:\n[FILE UNAVAILABLE]",
        ))
        .ok();
    desktop
        .push_child(FsNode::text_file(
            "report_fertilizer_consumption_q1_2189.txt",
            "Period: 01.01.2189 - 31.03.2189\nCompiled by: Jr. Technician P. Dupont (Sector 7A)\n\nFertilizer consumption for the reporting period exceeded planned targets by 12.7%. Suspected cause: increased metabolic rate in experimental wheat samples following lighting adjustment.\n\nAttachment 1: weekly consumption chart.\nAttachment 2: comparative growth analysis (current cycle vs. 2187 baseline).\nNote: Access to samples for additional testing requires clearance from Sector B checkpoint.",
        ))
        .ok();
    // Personal Folder
    desktop
        .push_child(build_personal_folder(act1_unlocked))
        .ok();
    // Corrupted Folder
    desktop
        .push_child(build_corrupted_folder(act1_unlocked))
        .ok();

    let mut home = FsNode::folder(HOME_PATH);
    home.push_child(desktop).ok();
    home.push_child(FsNode::folder("Downloads")).ok();
    FsHierarchy { root: home }
}

fn inject_secure_folder(vfs: &mut FsHierarchy, omega_tex: TextureId, size: egui::Vec2) {
    let Some(desktop) = vfs.get_node_mut(&FsPath::new(DESKTOP_PATH)) else {
        return;
    };
    desktop
        .push_child(build_secure_folder(Some(omega_tex), size, true))
        .ok();
}

/// The Act 2 desktop: `[SECURE]` only, already decrypted so `netripper` can run on it.
fn build_secure_only_fs(omega_tex: Option<TextureId>, size: egui::Vec2) -> FsHierarchy {
    let mut desktop = FsNode::folder("Desktop");
    desktop
        .push_child(build_secure_folder(omega_tex, size, false))
        .ok();

    let mut home = FsNode::folder(HOME_PATH);
    home.push_child(desktop).ok();
    FsHierarchy { root: home }
}

fn build_personal_folder(unlocked: bool) -> FsNode {
    let mut personal_folder = if unlocked {
        FsNode::folder("Personal")
    } else {
        FsNode::password_folder("Personal", "7714")
    };
    personal_folder
        .push_child(FsNode::text_file("collective_complaint.txt", "TO: Administrator Makarov\nFROM: French Contingent Representative, Lieutenant J. Moreau\nDATE: 16.03.2189\nSUBJECT: Anomalous source of sound.\n\nI hereby report that over the past two weeks, 7 (seven) instances of low-frequency hum have been recorded in living and working areas assigned to the French contingent. The sound is not detected by standard instruments, yet is subjectively perceived by crew members, causing headaches, irritability, and sleep disruption.\n\nGiven that no similar complaints have been received from the Soviet part of the crew, I request clarification on the following:\n\n1. Is this hum a technical feature of equipment located in Soviet sectors?\n2. Is it related to the operation of synthetic modules (MK-II and above)?\n3. Will measures be taken to shield against or reduce exposure to the human part of the crew?\n\nAwaiting official response.\n\n\n- J. Moreau"))
        .ok();
    personal_folder
        .push_child(FsNode::text_file("notes.txt", "16.03\nEncore ce bruit. Trente secondes. Personne d'autre ne l'entend? Les Russes font semblant? Makarov est un menteur.\n\n17.03\nVu une des leurs, Thallo, dans le couloir. Elle avait l'air... perdue. Comme si elle ne savait pas où elle était. Je lui ai parlé, elle a répondu normalement, mais dans ses yeux — du vide. Ça arrive depuis que le bruit a commencé?"))
        .ok();
    personal_folder
        .push_child(FsNode::text_file("response.txt", "TO: J. Moreau\nFROM: A. Makarov\nDATE: 17.03.2189\nSUBJECT: RE: Anomalous source of sound.\nYour complaint has been reviewed. Conducted inspections revealed no anomalous acoustic signals in the specified sectors. Soviet sector equipment operates within normal parameters. I recommend an unscheduled examination of French auditory equipment for possible collective sensory aberration. A psychosomatic origin of the phenomenon due to mission duration is also plausible.\n\nConsider the matter closed.\n\n\n- A. Makarov"))
        .ok();
    personal_folder
}

fn build_corrupted_folder(decrypted: bool) -> FsNode {
    let mut corrupted_folder = if decrypted {
        FsNode::folder("[CORRUPTED]")
    } else {
        FsNode::encrypted_folder("[CORRUPTED]")
    };
    corrupted_folder
        .push_child(FsNode::text_file("behavioral_notes_thallo.txt", "PERSONNEL OBSERVATION\nSubject: Thallo (soviet synth, attached to agro-sector)\nObserver: P. Lefevre\nPeriod: 12.03-19.03.2189\n\n14.03, 22:40 - during hum episode(s), subject was in adjacent corridor. After hum ceased, observed the following deviations:\n- Disorientation (subject stopped and looked around for 10-15 seconds aimlessly).\n- Failure to recognize familiar route (subject passed turn to their sector twice before returning).\n- When asked \"Everything alright?\" - subject responded affirmatively, but with 2-3 second delay.\n\n15.03 // Subject did not remember a brief conversation from 14.03. Claimed not to have seen me that day.\n\n16.03 // Repeat hum episode. Identical reaction: disorientation, memory loss of events occurring during the hum.\n\nConclusion: The hum directly affects cognitive functions of synthetic modules, particularly short-term memory. The Soviet part of the crew either is not affected or is concealing the fact."))
        .ok();
    corrupted_folder
        .push_child(FsNode::text_file(
            "hum_measurement_attempt.txt",
            "Attempt to measure the hum frequency.\nEquipment: SA-7 spectrum analyzer\nResult: background noise of 50-60 Hz (standard).\nNo anomalies detected.\nConclusion: No confirmation. Recommend equipment calibration check.",
        ))
        .ok();
    corrupted_folder
        .push_child(FsNode::text_file(
            "synth_specs.txt",
            "MODEL: MK-II «Red Star»\nPURPOSE: Tactical support, perimeter security\nFEATURES: Reinforced armor, autonomous generator, suppressed pain reflexes.\nNOTE: Due to increased neural network load, regular short-term memory defragmentation is recommended. Frequency: once per 72 hours.",
        ))
        .ok();
    corrupted_folder
}

fn build_secure_folder(omega_tex: Option<TextureId>, size: egui::Vec2, encrypted: bool) -> FsNode {
    // omega folder
    let mut omega_folder = FsNode::password_folder("[OMEGA]", "CCCP-VOS-770451");
    omega_folder
        .push_child(FsNode::text_file(
            "[UNIDENTIFIED]",
            "// THIS INFORMATION IS CLASSIFIED. The letter should be destroyed after reading. //\n\nCamera footage has shown that these anomalies were not seen during the video recording, but it should be otherwise. Engineers who were involved in one way or another in the incidents say that “it” should definitely have been caught on video footage.\n\nThe cameras were clearly looking exactly in their direction. From the eyewitness accounts, a portrait can be drawn of what exactly the ship's crew saw. A small formless creature with elongated limbs, seen only out of the corner of the eye.\n\nBehavior is described as passive, observes from a distance (~10-15m, from behind cover), but if it sees that the victim has spotted it, it runs away (Where exactly - it remains to be seen, the crew is already searching all ventilation passages for anomalies).\n\nAfter the anomaly flees from the sector there is some kind of a breakdown. Breakdowns caused by the anomaly are far worse and more dangerous than those that occurred due to wear and tear of the equipment a few weeks ago.\n\nIf there's an emergency breakdown, report all information to the safety unit. We need to find out exactly what this thing is doing and why it's doing it\n\n// End of report. //",
        ))
        .ok();
    if let Some(tex) = omega_tex {
        omega_folder
            .push_child(FsNode::img_file("omega.png", tex, size))
            .ok();
    }

    // secure folder
    let mut secure_folder = if encrypted {
        FsNode::encrypted_folder("[SECURE]")
    } else {
        FsNode::folder("[SECURE]")
    };
    secure_folder.push_child(omega_folder).ok();
    secure_folder
        .push_child(FsNode::text_file(
            "access_log_makarov.txt",
            "ADMINISTRATOR TERMINAL ACCESS LOG\n\nLast 5 logins:\n\n15.03 08:12 — MAKAROV_A — successful\n15.03 14:47 — MAKAROV_A — successful\n16.03 09:03 — MAKAROV_A — successful\n16.03 23:41 — MAKAROV_A — successful\n17.03 04:15 — MAKAROV_A — successful\n\nAdministrator terminal access requires:\n1. Login: MKRV_A\n2. Password: ▉▉▉▉▉▉▉▉\n3. Level 2 confirmation keyword: [input required]",
        ))
        .ok();
    secure_folder
        .push_child(FsNode::text_file(
            "incident_log_security_omega.txt",
            "CLASSIFICATION: OMEGA\nINCIDENT: Unauthorized presence / anomaly\nDATE: 02.03.2189\nLOCATION: Ventilation shaft, sector 9C (junction of residential and technical)\n\nWitnesses: Pvt. ▉▉▉▉▉▉▉▉ (synth), Jr. Technician ▉▉▉▉▉▉▉▉ (human)\nDescription: Observed entity: presumably biological. Form unstable, limbs elongated.\nBehavior: passive observation from distance of 10-15 m.\nUpon approach attempt: retreats into ventilation. Post-retreat, power surge recorded in sector electrical grid, causing life support shutdown for 47 seconds.\n\nVideo recording: sector cameras did not capture entity, despite direct orientation.\nPreliminary conclusion: entity not detectable by standard surveillance equipment. Possible connection to previously reported \"hallucinations\" by French contingent.\n\nMakarov's order: Exclude from official reports. Continue observation via synthetic modules. If re-detected, attempt elimination or intrapment. Report back personally.",
        ))
        .ok();
    secure_folder
        .push_child(FsNode::text_file(
            "memo_to_self.txt",
            "17.03 23:12\nForgot the level 2 password again. Can't write it down, but need to remember.\nSunday's call sign at launch: CCCP-VOS-77.\nI can take the numbers from the call sign, but level 2 needs something longer.\nMaybe full registration code + Shift C? Probably.\nI'll remember.",
        ))
        .ok();
    secure_folder
        .push_child(FsNode::text_file(
            "MKRV_notes.txt",
            "Reminders:\n- Review Omega reports for last month.\n- Request new data on synth exposure from medical.\n- Contact Earth via secure channel.\n- Increase surveillance on French, especially Lefevre.\n- Change level 2 password to something project-related so I don't forget.",
        ))
        .ok();
    secure_folder
        .push_child(FsNode::text_file(
            "project_codenames_reference.txt",
            "PROJECTS AND CODE DESIGNATIONS:\n- Primary mission - VOSKRESENYE \n- Biological preservation - EUPHORIA / THALLO\n- Synthetic modules - KRASNAYA ZVEZDA\n- Anomaly monitoring - OMEGA\n- Emergency Earth communication - KRASNAYA ZARYA\n- Administrative reserve - ZERKALO",
        ))
        .ok();
    secure_folder
        .push_child(FsNode::text_file(
            "security_personnel.txt",
            "SECURITY DIVISION PERSONNEL LIST\n\nChief: Makarov A.P. (administrative access, level 1)\nDeputy Chief: Major Sobolev K.V. (operational access, level 2)\nShifts:\n- Shift A: Lt. ▉▉▉▉▉▉▉▉, Sgt. ▉▉▉▉▉▉▉▉, Pvt. ▉▉▉▉▉▉▉▉ (synth)\n- Shift B: Lt. ▉▉▉▉▉▉▉▉, Sgt. ▉▉▉▉▉▉▉▉ (synth), Pvt. ▉▉▉▉▉▉▉▉\n- Shift C: Lt. ▉▉▉▉▉▉▉▉, Sgt. ▉▉▉▉▉▉▉▉, Pvt. ▉▉▉▉▉▉▉▉ (synth)\n\nArmory access codes: Shift A — 4306, Shift B — 2290, Shift C — 0451",
        ))
        .ok();
    secure_folder
}
