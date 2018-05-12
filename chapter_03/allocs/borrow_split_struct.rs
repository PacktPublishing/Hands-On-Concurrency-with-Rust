use std::fmt;

enum Project {
    Apollo,
    Gemini,
    Mercury,
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Project::Apollo => write!(f, "Apollo"),
            Project::Mercury => write!(f, "Mercury"),
            Project::Gemini => write!(f, "Gemini"),
        }
    }
}

struct Mission {
    project: Project,
    number: u8,
    duration_days: u8,
}

fn main() {
    let mut mission = Mission {
        project: Project::Gemini,
        number: 2,
        duration_days: 0,
    };
    let proj: &Project = &mission.project;
    let num: &mut u8 = &mut mission.number;
    let dur: &mut u8 = &mut mission.duration_days;

    *num = 12;
    *dur = 3;

    println!("{} {} flew for {} days", proj, num, dur);
}
