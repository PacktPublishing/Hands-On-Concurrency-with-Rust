use std::cell::Cell;

enum Project {
    Apollo,
    Gemini,
    Mercury,
}

struct Mission {
    project: Project,
    number: u8,
    duration_days: Cell<u8>,
}

fn main() {
    let mission = Mission {
        project: Project::Mercury,
        number: 7,
        duration_days: Cell::new(255),
    };

    mission.duration_days.set(0);
    assert_eq!(0, mission.duration_days.get());
}
