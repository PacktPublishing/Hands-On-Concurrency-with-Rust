use egress::Egress;
use event;
use quantiles;
use util;

pub struct CKMSEgress {
    error: f64,
    data: util::HashMap<String, quantiles::ckms::CKMS<u32>>,
    new_data_since_last_report: bool,
}

impl Egress for CKMSEgress {
    fn deliver(&mut self, event: event::Telemetry) -> () {
        self.new_data_since_last_report = true;
        let val = event.value;
        let ckms = self.data
            .entry(event.name)
            .or_insert(quantiles::ckms::CKMS::new(self.error));
        ckms.insert(val);
    }

    fn report(&mut self) -> () {
        if self.new_data_since_last_report {
            for (k, v) in &self.data {
                for q in &[0.0, 0.25, 0.5, 0.75, 0.9, 0.99] {
                    println!("[CKMS] {} {}:{}", k, q, v.query(*q).unwrap().1);
                }
            }
            self.new_data_since_last_report = false;
        }
    }
}

impl CKMSEgress {
    pub fn new(error: f64) -> Self {
        CKMSEgress {
            error: error,
            data: Default::default(),
            new_data_since_last_report: false,
        }
    }
}
