use event;
use filter::Filter;

pub struct LowFilter {
    limit: u32,
}

impl LowFilter {
    pub fn new(limit: u32) -> Self {
        LowFilter { limit: limit }
    }
}

impl Filter for LowFilter {
    fn process(
        &mut self,
        event: event::Telemetry,
        res: &mut Vec<event::Telemetry>,
    ) -> () {
        if event.value <= self.limit {
            res.push(event);
        }
    }
}
