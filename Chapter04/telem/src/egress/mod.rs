use event;
use std::sync::mpsc;

mod cma_egress;
mod ckms_egress;

pub use self::ckms_egress::*;
pub use self::cma_egress::*;

pub trait Egress {
    fn deliver(&mut self, event: event::Telemetry) -> ();

    fn report(&mut self) -> ();

    fn run(&mut self, recv: mpsc::Receiver<event::Event>) {
        for event in recv.into_iter() {
            match event {
                event::Event::Telemetry(telem) => self.deliver(telem),
                event::Event::Flush => self.report(),
            }
        }
    }
}
