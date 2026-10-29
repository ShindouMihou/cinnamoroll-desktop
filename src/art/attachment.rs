use crate::behavior::Activity;
use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Clone, Copy, Debug, Deserialize)]
pub(super) struct Attachments {
    pub hat: [f32; 4],
    pub bow: [f32; 4],
    pub scarf: [f32; 4],
    pub bag: [f32; 4],
}

type Rigs = BTreeMap<String, [Attachments; 8]>;

pub(super) fn pose(
    activity: Activity,
    index: usize,
    facing: f32,
) -> Result<Option<Attachments>, String> {
    static RIGS: OnceLock<Result<Rigs, String>> = OnceLock::new();
    let rigs = RIGS.get_or_init(|| {
        serde_json::from_slice(include_bytes!("../../assets/strips/attachments.json"))
            .map_err(|error| format!("Invalid accessory attachment data: {error}"))
    });
    let rigs = rigs.as_ref().map_err(Clone::clone)?;
    let Some(strip) = super::strip_catalog::find(activity) else {
        return Ok(None);
    };
    let frames = rigs
        .get(strip.name)
        .ok_or_else(|| format!("Missing accessory attachments for {}", strip.name))?;
    let source = super::strips::source_index(activity, index.min(7));
    let mut pose = frames[source];
    for anchor in [&mut pose.hat, &mut pose.bow, &mut pose.scarf, &mut pose.bag] {
        if activity == Activity::Land {
            anchor[1] = 197. + (anchor[1] - 197.) * (0.94 + index as f32 / 7. * 0.06);
        }
        if facing < 0. && super::strip_catalog::directional(activity) {
            anchor[0] = 320. - anchor[0];
            anchor[2] = -anchor[2];
        }
    }
    Ok(Some(pose))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_strip_has_finite_attachments_and_profile_poses_mirror_together() {
        for activity in Activity::ALL {
            if super::super::strip_catalog::find(activity).is_none() {
                continue;
            }
            for frame in 0..8 {
                let right = pose(activity, frame, 1.).unwrap().unwrap();
                let left = pose(activity, frame, -1.).unwrap().unwrap();
                for (r, l) in [right.hat, right.bow, right.scarf, right.bag]
                    .into_iter()
                    .zip([left.hat, left.bow, left.scarf, left.bag])
                {
                    assert!(r.iter().all(|v| v.is_finite()));
                    assert!((0.5..=1.5).contains(&r[3]));
                    if super::super::strip_catalog::directional(activity) {
                        assert!((r[0] + l[0] - 320.).abs() < 0.01);
                        assert_eq!(r[1], l[1]);
                        assert_eq!(r[2], -l[2]);
                    }
                }
            }
        }
    }
    #[test]
    fn blink_does_not_detach_outfits_and_body_does_not_follow_head_roll() {
        let open = pose(Activity::Idle, 0, 1.).unwrap().unwrap();
        let closed = pose(Activity::Idle, 4, 1.).unwrap().unwrap();
        assert!((closed.hat[0] - open.hat[0]).abs() < 3.);
        assert!((closed.hat[1] - open.hat[1]).abs() < 3.);
        assert!((closed.bag[0] - open.bag[0]).abs() < 3.);
        let tilted = pose(Activity::EarWiggle, 4, 1.).unwrap().unwrap();
        assert!(tilted.hat[2].abs() > 5.);
        assert_eq!(tilted.bag[2], 0.);
    }
}
