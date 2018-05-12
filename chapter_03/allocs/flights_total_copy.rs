#[derive(Clone, Copy, PartialEq, Eq)]
enum Project {
    Apollo,
    Gemini,
    Mercury,
}

#[derive(Clone, Copy)]
struct Mission {
    project: Project,
    number: u8,
    duration_days: u8,
}

fn flight() -> Mission {
    Mission {
        project: Project::Apollo,
        number: 8,
        duration_days: 6,
    }
}

fn main() {
    assert_eq!(::std::mem::size_of::<Mission>(), 3);
    let mission = flight();
    if mission.project == Project::Apollo && mission.number == 8 {
        assert_eq!(mission.duration_days, 6);
    }
}
