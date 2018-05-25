//! Utility module, a grab-bag of functionality
use event;
use seahash::SeaHasher;
use std::collections;
use std::hash;
use std::sync::mpsc;

pub type HashMap<K, V> =
    collections::HashMap<K, V, hash::BuildHasherDefault<SeaHasher>>;

pub fn send(chans: &[mpsc::Sender<event::Event>], event: event::Event) {
    if chans.is_empty() {
        return;
    }

    for chan in chans.iter() {
        chan.send(event.clone()).unwrap();
    }
}
