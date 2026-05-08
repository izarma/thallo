use crate::engine::scripted_events::{
    ChatTriggerType, DialogueLine, Dialogues, FileDialogueTriggers, FileTriggerOperation,
    NewFileReceiving, RebootSequence, ScriptedEventTrigger,
};

pub(super) fn build_act1_dialogues() -> Dialogues {
    Dialogues {
        lines: vec![
            DialogueLine::new(false, "Hey, you’re in?").on_complete(
                ScriptedEventTrigger::ChatTrigger(ChatTriggerType::FileTransfer(
                    NewFileReceiving::BruteForce,
                )),
            ), // false is anon
            DialogueLine::new(true, "i think so. what are we looking for exactly?"), // true is player
            DialogueLine::new(
                false,
                "Nothing too concrete yet. There are rumors that got to me that this terminal has some encrypted files. I want to look into them.",
            ).on_complete(
                ScriptedEventTrigger::ChatTrigger(ChatTriggerType::FileTransfer(
                    NewFileReceiving::NetRipper,
                )),
            ),
            DialogueLine::new(true, "isn't this terminal connected to the rest?"),
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

pub(super) fn build_act1_file_triggers(file_triggers: &mut FileDialogueTriggers) {
    file_triggers.register(
        &["incident_report.txt"],
        FileTriggerOperation::Any,
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

    file_triggers.register(
        &["collective_complaint.txt", "notes.txt", "response.txt"],
        FileTriggerOperation::All,
        vec![
            DialogueLine::new(false, "Okay, that’s… weird."),
            DialogueLine::new(true, "yeah I though the same. have you ever heard the hum?"),
            DialogueLine::new(false, "Nope. Not even once."),
            DialogueLine::new(true, "ok this is getting ridiculous."),
            DialogueLine::new(false, "?"),
            DialogueLine::new(true, "i mean it doesnt make sense. there cant be a possibility that one part of the crew just suddenly hallucinates?"),
            DialogueLine::new(false, "Well, it’s either that or there is something wrong here. We probably need to dig deeper. There should be another encrypted folder on the desktop."),
            DialogueLine::new(true, "yeah, there is. any ideas how to open this?"),
            DialogueLine::new(false, "Well, this one guy I know helped me out and sent some sort of a brute force file. Might come in handy on your side. Sending it right now.").on_complete(
                ScriptedEventTrigger::ChatTrigger(ChatTriggerType::FileTransfer(
                    NewFileReceiving::BruteForce,
                )),
            ),
            DialogueLine::new(false, "Try Opening the Encrypted Folder now"),
        ],
    );

    file_triggers.register(
        &["behavioral_notes_thallo.txt"],
        FileTriggerOperation::All,
        vec![
            DialogueLine::new(true, "what the fuck."),
            DialogueLine::new(false, "Okay, Thal, calm down."),
            DialogueLine::new(true, "??? are you an idiot?"),
            DialogueLine::new(false, "No."),
            DialogueLine::new(true, "there is literally an explanation to our amnesia and more, and you're telling me to be calm???"),
            DialogueLine::new(false, "Yes, I did fucking see it. What do you want me to say?"),
            DialogueLine::new(true, "kostya, youre getting on my nerves right now. we need to dig deeper. i want to know who the fuck is behind this."),
            DialogueLine::new(true, "and"),
            DialogueLine::new(true, "who we are really."),
            DialogueLine::new(false, "And how do you think we’ll do that, Ms. “I wanna know the truth”?"),
            DialogueLine::new(true, "connect me back."),
            DialogueLine::new(false, "NO. If you want to get your head blown off in next few days – be my guest and do it yourself. You know the consequences. I’m NOT taking this much risk."),
            DialogueLine::new(true, "fuck you then.").on_complete(
                ScriptedEventTrigger::BeginReboot(RebootSequence::NetworkConnect),
            ),
        ],
    );
}

pub(super) fn build_netconn_file_triggers(file_triggers: &mut FileDialogueTriggers) {
    file_triggers.register_event(
        &["[UNIDENTIFIED]", "omega.png"],
        FileTriggerOperation::All,
        ScriptedEventTrigger::BeginReboot(RebootSequence::ActTrans),
    );
}

pub(super) fn build_act2_dialogues() -> Dialogues {
    Dialogues {
        lines: vec![
            DialogueLine::new(false, "Sending you NetRipper").on_complete(
                ScriptedEventTrigger::ChatTrigger(ChatTriggerType::FileTransfer(
                    NewFileReceiving::NetRipper,
                )),
            ),
            DialogueLine::new(
                false,
                "Once you receive this program, try running it on the folder with your terminal",
            ),
        ],
        index: 0,
    }
}
