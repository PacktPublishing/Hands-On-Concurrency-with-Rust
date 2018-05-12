#[derive(Clone, Copy)]
enum Project {
    Apollo,
    Mercury,
    Gemini,
}

#[derive(Clone, Copy)]
struct Mission {
    project: Project,
    number: u8,
}

fn main() {
    let m = Mission {
        project: Project::Apollo,
        number: 17,
    };
}
