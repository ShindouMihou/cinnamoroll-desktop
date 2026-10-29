pub(super) const WIDTH: u32 = 284;
pub(super) const TOP: f32 = 94.;
pub(super) const ROW: f32 = 42.;
pub(super) const FOOTER: u32 = 58;

pub(super) fn height(entries: usize) -> u32 {
    TOP as u32 + entries as u32 * ROW as u32 + FOOTER
}

pub(super) fn row(index: usize) -> [f32; 4] {
    [12., TOP + index as f32 * ROW, 260., 38.]
}

pub(super) fn hit(x: f32, y: f32, entries: usize) -> Option<usize> {
    if !(12. ..272.).contains(&x) || y < TOP {
        return None;
    }
    let index = ((y - TOP) / ROW) as usize;
    (index < entries && y < row(index)[1] + 38.).then_some(index)
}

pub(super) fn footer_hit(x: f32, y: f32, height: f32) -> Option<bool> {
    let top = height - FOOTER as f32 + 16.;
    if !(top..top + 30.).contains(&y) {
        return None;
    }
    if (12. ..136.).contains(&x) {
        Some(true)
    } else if (148. ..272.).contains(&x) {
        Some(false)
    } else {
        None
    }
}
