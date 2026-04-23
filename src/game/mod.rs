use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiTextureHandle,
    egui::{self, TextureId},
};

use crate::engine::{
    asset_tracking::LoadResource,
    file_system::{FileType, FsHierarchy, FsNode, HOME_PATH, LockType, NodeMeta},
    screens::Screen,
    scripted_events::{DialogueLine, Dialogues, FileDialogueTriggers},
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<Act1Assets>();
    app.add_systems(
        OnEnter(Screen::Desktop),
        setup_act1_stuffs
            .run_if(resource_exists::<Act1Assets>)
            .run_if(not(resource_exists::<FsHierarchy>)),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct Act1Assets {
    #[dependency]
    pub omega: Handle<Image>,
}

impl FromWorld for Act1Assets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            omega: assets.load("game/act1/omega_r.png"),
        }
    }
}

fn setup_act1_stuffs(
    mut contexts: EguiContexts,
    assets: Res<Act1Assets>,
    images: Res<Assets<Image>>,
    mut file_triggers: ResMut<FileDialogueTriggers>,
    mut cmd: Commands,
) {
    // File hierarchy
    let size = images
        .get(&assets.omega)
        .map(|img| {
            let s = img.size_f32();
            egui::Vec2::new(s.x, s.y)
        })
        .unwrap_or(egui::Vec2::splat(64.0));
    let omega_tex = contexts.add_image(EguiTextureHandle::Weak(assets.omega.id()));
    cmd.insert_resource(build_fs_hierarchy(omega_tex, size));

    // Dialogues
    cmd.insert_resource(build_dialogues());
    build_act1_file_triggers(&mut file_triggers);
}

fn build_dialogues() -> Dialogues {
    Dialogues {
        lines: vec![
            DialogueLine::new(false, "Hey, you’re in?"), // false is anon
            DialogueLine::new(true, "i think so. what are we looking for exactly?"), // true is player
            DialogueLine::new(
                false,
                "Nothing too concrete yet. There are rumors that got to me that this terminal has some encrypted files. I want to look into them.",
            ),
            DialogueLine::new(true, "isnt this terminal connected to the rest?"),
            DialogueLine::new(
                false,
                "Nope. Cut the lines before you came in. Said to them that the terminal needs some maintenance and some more bullshit. Are you sure nobody saw you on the way?",
            ),
            DialogueLine::new(true, "yeah. i should be safe for an hour or so."),
            DialogueLine::new(
                false,
                "Good. Still, make it quick. I suggest giving a look at some files related to your sector.",
            ),
            DialogueLine::new(true, "god i hate plants."),
            DialogueLine::new(
                false,
                "No shit, but I still need for you to read through them carefully. I overlooked the desktop and there should be a few encrypted folders. You know whose terminal is this? Can be connected to that.",
            ),
            DialogueLine::new(
                true,
                "no fucking clue. probably some french nerd i was talking to the other day. he is awful…",
            ),
            DialogueLine::new(false, "HAHAHAH, YOU MEAN PIERRE??"),
            DialogueLine::new(
                true,
                "yeah he thinks im into him. poor guy doesnt even have a clue we used him.",
            ),
            DialogueLine::new(
                false,
                "Hahaha, yeah. OK, let’s stop fooling around. Your terminal is only connected to mine, so I can see what you’re doing. I’ll try to help along the way.",
            ),
            DialogueLine::new(true, "yup."),
        ],
        index: 0,
    }
}

fn build_act1_file_triggers(file_triggers: &mut FileDialogueTriggers) {
    file_triggers.register(
        "incident_report.txt",
        vec![
            DialogueLine::new(false, "Hey, might be important."),
            DialogueLine::new(true, "doesnt seem like it."),
            DialogueLine::new(
                false,
                "Jesus, just take a closer look! It’s his report after all.",
            ),
            DialogueLine::new(true, "okay okay."),
        ],
    );

    // Future file triggers go here
}

fn build_fs_hierarchy(omega_tex: TextureId, size: egui::Vec2) -> FsHierarchy {
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
