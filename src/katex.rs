use std::env;
use std::fs;
use std::process::Command;
use uuid::Uuid;

pub fn render(src: String, is_display_mode: bool) -> String {
    let mut path = env::temp_dir();
    path.push(Uuid::new_v4().to_string());
    fs::create_dir(path.as_path()).expect("failed to create temporary dir");
    path.push("input.txt");
    let mut input_file = fs::File::create(path.as_path()).expect("failed to create file");
    use std::io::prelude::*;
    writeln!(input_file, "{}", src).expect("failed to write input src");
    let input_filepath = path.clone();
    path.pop();

    let mut command = Command::new("npx");
    command.arg("katex");
    if is_display_mode {
        command.arg("--display-mode");
    }
    command.arg("--input").arg(input_filepath);
    let output = command.output().expect("failed to execute `npx katex`");
    fs::remove_dir_all(path).expect("failed to remove temporary file");

    String::from_utf8(output.stdout).expect("failed to recognize stdout as utf8 string")
}
