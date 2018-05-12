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
    let gemini_2 = Mission {
        project: Project::Gemini,
        number: 2,
        duration_days: 0,
    };

    let mut missions: [&Mission; 2] = [&gemini_2, &gemini_2];

    let m0 = &mut missions[0];
    let _m1 = &mut missions[1];

    println!(
        "{} {} flew for {} days",
        m0.project, m0.number, m0.duration_days
    );
}
