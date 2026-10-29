use crate::behavior::Activity;

pub(super) fn activity_name(activity: Activity) -> &'static str {
    match activity {
        Activity::CozyNap => "Cozy nap",
        Activity::Fetch => "Fetching a toy",
        Activity::TakeTreat => "Enjoying a treat",
        Activity::SniffTreat => "Sniffing a treat",
        Activity::TypingPeek => "Watching you type",
        Activity::Petting => "Head pats",
        Activity::CheekRub => "Cheek rubs",
        Activity::Curious => "Feeling curious",
        Activity::Excited => "Feeling excited",
        Activity::Stalk => "Ready to pounce",
        _ => activity.label(),
    }
}
