#[derive(Clone, Copy)]
pub struct CozyReturn {
    pub due: f64,
    pub home: (f32, f32),
}

pub struct PawOffer {
    pub until: f64,
    contact_since: Option<f64>,
}

#[derive(Debug, PartialEq)]
pub enum Response {
    Waiting,
    Accepted,
    Expired,
}

impl PawOffer {
    pub fn new(now: f64) -> Self {
        Self {
            until: now + 10.,
            contact_since: None,
        }
    }

    pub fn sample(&mut self, point: (f32, f32), now: f64) -> Response {
        if now >= self.until {
            return Response::Expired;
        }
        if (point.0 - 205.).hypot(point.1 - 113.) > 22. {
            self.contact_since = None;
            return Response::Waiting;
        }
        let entered = *self.contact_since.get_or_insert(now);
        if now - entered >= 0.8 {
            Response::Accepted
        } else {
            Response::Waiting
        }
    }

    pub fn hold(&mut self, elapsed: f64) {
        self.until += elapsed;
        if let Some(since) = self.contact_since.as_mut() {
            *since += elapsed;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raised_paw_needs_a_deliberate_hold_and_leaving_restarts_it() {
        let mut offer = PawOffer::new(0.);
        assert_eq!(offer.sample((205., 113.), 1.), Response::Waiting);
        assert_eq!(offer.sample((100., 150.), 1.7), Response::Waiting);
        assert_eq!(offer.sample((205., 113.), 2.), Response::Waiting);
        assert_eq!(offer.sample((205., 113.), 2.81), Response::Accepted);
    }

    #[test]
    fn offers_expire_without_requiring_a_response_and_menu_holds_shift_deadlines() {
        let mut offer = PawOffer::new(0.);
        offer.hold(20.);
        assert_eq!(offer.sample((0., 0.), 29.), Response::Waiting);
        assert_eq!(offer.sample((205., 113.), 30.), Response::Expired);
    }
}
