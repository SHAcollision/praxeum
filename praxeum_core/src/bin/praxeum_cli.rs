use clap::Parser;
use praxeum_core::{
    loader::{default_examples_path, DataFormat, ExerciseLoader},
    Answer, Exercise, ExerciseEngine,
};
use std::io::{self, Write};
use std::path::PathBuf;

/// Simple CLI demo for the Praxeum exercise engine.
#[derive(Parser, Debug)]
#[command(author, version, about = "Praxeum CLI demo")]
struct Cli {
    /// Path to exercises file (TOML or JSON).
    #[arg(short, long)]
    file: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let path = cli.file.unwrap_or_else(default_examples_path);

    println!("Loading exercises from: {}", path.display());
    let exercises = ExerciseLoader::load_from_path(&path, DataFormat::Auto)?;
    if exercises.is_empty() {
        eprintln!("No exercises loaded.");
        return Ok(());
    }

    let engine = ExerciseEngine::new(exercises);

    println!("Loaded {} exercises.\n", engine.len());

    for exercise in engine.iter() {
        run_exercise(&engine, exercise)?;
        println!("\n---\n");
    }

    Ok(())
}

fn run_exercise(
    engine: &ExerciseEngine,
    exercise: &Exercise,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("# {} [{}]", exercise.title(), exercise.id());

    match exercise {
        Exercise::Classification {
            prompt,
            categories,
            items,
            ..
        } => {
            println!("{}", prompt);
            println!();
            for (i, cat) in categories.iter().enumerate() {
                println!("  [{}] {}", i, cat);
            }
            println!();
            let mut indices = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                println!("Item {}: {}", i, item.text);
                let idx = prompt_usize("Choose category index: ")?;
                indices.push(idx);
            }
            let answer = Answer::Classification(indices);
            let eval = engine.evaluate(exercise, &answer)?;
            println!(
                "\nResult: correct = {}, score = {:.2}",
                eval.correct, eval.score
            );
            println!("{}", eval.feedback);
        }

        Exercise::MultipleChoice {
            prompt,
            options,
            multi_select,
            ..
        } => {
            println!("{}", prompt);
            println!();
            for (i, opt) in options.iter().enumerate() {
                println!("  [{}] {}", i, opt);
            }
            println!();
            if *multi_select {
                println!("Enter one or more indices separated by commas:");
            } else {
                println!("Enter a single index:");
            }
            let line = prompt_line("Your choice(s): ")?;
            let indices = parse_indices(&line);
            let answer = Answer::MultipleChoice(indices);
            let eval = engine.evaluate(exercise, &answer)?;
            println!(
                "\nResult: correct = {}, score = {:.2}",
                eval.correct, eval.score
            );
            println!("{}", eval.feedback);
        }

        Exercise::Scenario {
            description,
            prompt,
            choices,
            ..
        } => {
            println!("{}", description);
            println!();
            println!("{}", prompt);
            println!();
            for (i, choice) in choices.iter().enumerate() {
                println!("  [{}] {}", i, choice.label);
            }
            let idx = prompt_usize("\nChoose option index: ")?;
            let answer = Answer::Scenario(idx);
            let eval = engine.evaluate(exercise, &answer)?;
            println!(
                "\nResult: correct = {}, score = {:.2}",
                eval.correct, eval.score
            );
            println!("{}", eval.feedback);
        }
    }

    Ok(())
}

fn prompt_line(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

fn prompt_usize(prompt: &str) -> io::Result<usize> {
    loop {
        let line = prompt_line(prompt)?;
        match line.parse::<usize>() {
            Ok(n) => return Ok(n),
            Err(_) => {
                println!("Invalid integer, try again.");
            }
        }
    }
}

fn parse_indices(input: &str) -> Vec<usize> {
    input
        .split([',', ' ', ';'])
        .filter_map(|s| s.trim().parse::<usize>().ok())
        .collect()
}
