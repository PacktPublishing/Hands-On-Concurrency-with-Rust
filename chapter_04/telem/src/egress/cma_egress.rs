use egress::Egress;
use event;
use util;

struct CMA {
    n: u64,
    cma: f64,
}

pub struct CMAEgress {
    data: util::HashMap<String, CMA>,
    new_data_since_last_report: bool,
}

impl Egress for CMAEgress {
    fn deliver(&mut self, event: event::Telemetry) -> () {
        self.new_data_since_last_report = true;
        let val = event.value;
        let cma = self.data
            .entry(event.name)
            .or_insert(CMA { n: 0, cma: 0.0 });
        cma.n += 1;
        cma.cma = cma.cma + (((val as f64) - cma.cma) / (cma.n as f64));
    }

    fn report(&mut self) -> () {
        if self.new_data_since_last_report {
            for (k, v) in &self.data {
                println!("[CMA] {} {}", k, v.cma);
            }
            self.new_data_since_last_report = false;
        }
    }
}

impl CMAEgress {
    /// TODO
    pub fn new() -> Self {
        CMAEgress {
            data: Default::default(),
            new_data_since_last_report: false,
        }
    }
}
