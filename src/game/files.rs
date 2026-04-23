use bevy_egui::egui::{self, TextureId};

use crate::engine::file_system::{FileType, FsHierarchy, FsNode, HOME_PATH, LockType, NodeMeta};

pub(super) fn build_fs_hierarchy(omega_tex: TextureId, size: egui::Vec2) -> FsHierarchy {
    let mut desktop = FsNode::folder("Desktop");
    desktop
        .push_child(FsNode::text_file(
            "incident_report.txt",
            "INCIDENT REPORT\nSector: Agricultural (7A)\nCompiled by: Sr. Technician P. Lefevre\nDate: 14.03.2189\nIncident №: 7714\n\nSummary: At 22:40 ship time during routine check of sector 7B, an extraneous sound (low-frequency hum) was detected. Source could not be visually or audibly identified. Sound lasted approximately 45 seconds, then ceased.\n\nActions taken:\n1. Visual inspection of sector – negative, yielded no results.\n2. Check of communications and ventilation systems — no deviations.\n3. Inquiry to adjacent sectors (7B, 7C) — no similar complaints received.\n\nConclusion: Presumed acoustic defect in ventilation system or an unexpected error in reporting party's auditory equipment. Incident dismissed.\n\nAttachment: audio recording of attempted hum capture:\n[FILE UNAVAILABLE]",
        ))
        .ok();
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
            "report_fertilizer_consumption_q1_2189.txt",
            "Period: 01.01.2189 - 31.03.2189\nCompiled by: Jr. Technician P. Dupont (Sector 7A)\n\nFertilizer consumption for the reporting period exceeded planned targets by 12.7%. Suspected cause: increased metabolic rate in experimental wheat samples following lighting adjustment.\n\nAttachment 1: weekly consumption chart.\nAttachment 2: comparative growth analysis (current cycle vs. 2187 baseline).\nNote: Access to samples for additional testing requires clearance from Sector B checkpoint.",
        ))
        .ok();
    let mut personal_folder = FsNode::password_folder("Personal", "7714");
    personal_folder
        .push_child(FsNode::text_file("collective_complaint.txt", "TO: Administrator Makarov\nFROM: French Contingent Representative, Lieutenant J. Moreau\nDATE: 16.03.2189\nSUBJECT: Anomalous source of sound.\n\nI hereby report that over the past two weeks, 7 (seven) instances of low-frequency hum have been recorded in living and working areas assigned to the French contingent. The sound is not detected by standard instruments, yet is subjectively perceived by crew members, causing headaches, irritability, and sleep disruption.\n\nGiven that no similar complaints have been received from the Soviet part of the crew, I request clarification on the following:\n\n1. Is this hum a technical feature of equipment located in Soviet sectors?\n2. Is it related to the operation of synthetic modules (MK-II and above)?\n3. Will measures be taken to shield against or reduce exposure to the human part of the crew?\n\nAwaiting official response.\n\n\n- J. Moreau"))
        .ok();
    personal_folder
        .push_child(FsNode::text_file("notes.txt", "16.03\nEncore ce bruit. Trente secondes. Personne d'autre ne l'entend? Les Russes font semblant? Makarov est un menteur.\n\n17.03\nVu une des leurs, Thallo, dans le couloir. Elle avait l'air... perdue. Comme si elle ne savait pas où elle était. Je lui ai parlé, elle a répondu normalement, mais dans ses yeux — du vide. Ça arrive depuis que le bruit a commencé?"))
        .ok();
    personal_folder
        .push_child(FsNode::text_file("response.txt", "TO: J. Moreau\nFROM: A. Makarov\nDATE: 17.03.2189\nSUBJECT: RE: Anomalous source of sound.\nYour complaint has been reviewed. Conducted inspections revealed no anomalous acoustic signals in the specified sectors. Soviet sector equipment operates within normal parameters. I recommend an unscheduled examination of French auditory equipment for possible collective sensory aberration. A psychosomatic origin of the phenomenon due to mission duration is also plausible.\n\nConsider the matter closed.\n\n\n- A. Makarov"))
        .ok();
    desktop.push_child(personal_folder).ok();
    desktop
        .push_child(FsNode::encrypted_folder("[CORRUPTED]"))
        .ok();

    // Image file on the desktop
    desktop
        .push_child(FsNode {
            name: "omega.png".to_string(),
            file_type: FileType::Image(omega_tex, size),
            meta: NodeMeta {
                locked: Some(LockType::Password("xyz".to_string())),
            },
        })
        .ok();

    let mut bin = FsNode::folder("Bin");
    bin.push_child(FsNode::text_file(
        "todo.txt",
        "- Fix the anomaly\n- Review report",
    ))
    .ok();
    bin.push_child(FsNode::folder("Hmmm")).ok();
    desktop.push_child(bin).ok();

    let mut home = FsNode::folder(HOME_PATH);
    home.push_child(desktop).ok();
    home.push_child(FsNode::folder("Reports")).ok();
    home.push_child(FsNode::folder("Logs")).ok();
    home.push_child(FsNode::folder("Downloads")).ok();
    FsHierarchy { root: home }
}
