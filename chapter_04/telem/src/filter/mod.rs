use event;
use std::sync::mpsc;
use util;

mod high_filter;
mod low_filter;

pub use self::high_filter::*;
pub use self::low_filter::*;

pub trait Filter {
    fn process(
        &mut self,
        event: event::Telemetry,
        res: &mut Vec<event::Telemetry>,
    ) -> ();

    fn run(
        &mut self,
        recv: mpsc::Receiver<event::Event>,
        chans: Vec<mpsc::Sender<event::Event>>,
    ) {
        let mut telems = Vec::with_capacity(64);
        for event in recv.into_iter() {
            match event {
                event::Event::Flush => util::send(&chans, event::Event::Flush),
                event::Event::Telemetry(telem) => {
                    self.process(telem, &mut telems);
                    for telem in telems.drain(..) {
                        util::send(&chans, event::Event::Telemetry(telem))
                    }
                }
            }
        }
    }
}
