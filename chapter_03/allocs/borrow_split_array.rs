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
    let mut missions: [Mission; 2] = [
        Mission {
            project: Project::Gemini,
            number: 2,
            duration_days: 0,
        },
        Mission {
            project: Project::Gemini,
            number: 12,
            duration_days: 2,
        },
    ];

    let gemini_2 = &mut missions[0];
    let _gemini_12 = &mut missions[1];

    println!(
        "{} {} flew for {} days",
        gemini_2.project, gemini_2.number, gemini_2.duration_days
    );
}
