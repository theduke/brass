fn execute(dir: &str, args: &[&str]) {
    let cmd = &args[0];
    let cmd_full = args.join(" ");
    eprintln!("Running '{}'", cmd_full);
    let status = std::process::Command::new(cmd)
        .current_dir(dir)
        .args(&args[1..])
        .spawn()
        .expect(&format!("Could not start command '{}'", cmd_full))
        .wait()
        .expect(&format!("cmd failed: '{}'", cmd_full));

    if !status.success() {
        eprintln!("Command '{}' termainted with a non-0 exit code", cmd_full);
        std::process::exit(1);
    }
    eprintln!("Finished: '{}'", cmd_full);
}

fn todo_build() {
    execute(
        "examples/todo",
        &["cargo", "build", "--target", "wasm32-unknown-unknown"],
    );
    execute(
        "./",
        &[
            "wasm-bindgen",
            "--target",
            "web",
            "--no-typescript",
            // "--reference-types",
            "--out-dir",
            "examples/todo/pkg",
            "./target/wasm32-unknown-unknown/debug/brass_todo.wasm",
        ],
    )
}

fn todo_serve() {
    execute(
        "./",
        &[
            "cargo",
            "watch",
            "-w",
            "examples/todo/src",
            "-w",
            "src",
            "-s",
            "cargo xtask todo-build",
        ],
    );
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let arg_refs: Vec<_> = args.iter().map(|x| x.as_str()).collect();

    match arg_refs.as_slice() {
        &["todo-build"] => {
            todo_build();
        }
        &["todo-serve"] => {
            todo_serve();
        }
        _ => {
            eprint!("Unknown arguments");
            std::process::exit(1);
        }
    }
}
