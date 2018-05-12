use event;
use filter::Filter;

pub struct HighFilter {
    limit: u32,
}

impl HighFilter {
    pub fn new(limit: u32) -> Self {
        HighFilter { limit: limit }
    }
}

impl Filter for HighFilter {
    fn process(
        &mut self,
        event: event::Telemetry,
        res: &mut Vec<event::Telemetry>,
    ) -> () {
        if event.value >= self.limit {
            res.push(event);
        }
    }
}
