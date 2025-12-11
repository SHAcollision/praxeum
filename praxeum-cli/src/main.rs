use clap::Parser;
use praxeum_core::loader::{DataFormat, ExerciseLoader};
use praxeum_core::model::{answer::Answer, exercise::Exercise};
use praxeum_core::{Evaluation, ExerciseEngine};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Turbo CLI demo for the Praxeum exercise engine.
#[derive(Parser, Debug)]
#[command(author, version, about = "Praxeum CLI demo")]
struct Cli {
    /// Path to exercises file (TOML or JSON).
    #[arg(short, long)]
    file: Option<PathBuf>,
}

#[derive(Default)]
struct SessionStats {
    total: usize,
    correct: usize,
    streak: usize,
    best_streak: usize,
    total_score: f32,
    total_time: Duration,
    points: u32,
}

impl SessionStats {
    fn register(&mut self, eval: &Evaluation, elapsed: Duration) -> u32 {
        self.total += 1;
        self.total_score += eval.score;
        self.total_time += elapsed;

        if eval.correct {
            self.correct += 1;
            self.streak += 1;
        } else {
            self.streak = 0;
        }

        self.best_streak = self.best_streak.max(self.streak);
        let awarded = award_points(eval, elapsed, self.streak);
        self.points = self.points.saturating_add(awarded);
        awarded
    }

    fn print_summary(&self) {
        println!("\n== Session Summary ==");
        println!(
            "Exercises played: {} | Correct: {} | Avg score: {:.0}%",
            self.total,
            self.correct,
            if self.total == 0 {
                0.0
            } else {
                (self.total_score / self.total as f32) * 100.0
            }
        );
        println!(
            "Points: {} | Best streak: {} | Total time: {:.1}s",
            self.points,
            self.best_streak,
            self.total_time.as_secs_f32()
        );
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let path = cli
        .file
        .unwrap_or_else(praxeum_core::loader::default_examples_path);

    println!("Loading exercises from: {}", path.display());
    let exercises = ExerciseLoader::load_from_path(&path, DataFormat::Auto)?;
    if exercises.is_empty() {
        eprintln!("No exercises loaded.");
        return Ok(());
    }

    let engine = ExerciseEngine::new(exercises);
    let mut stats = SessionStats::default();

    println!(
        "Loaded {} exercises. \u{26a1} Race the clock and chain streaks!\n",
        engine.len()
    );

    for (idx, exercise) in engine.iter().enumerate() {
        println!("== Position {}/{} ==", idx + 1, engine.len());
        let (eval, elapsed) = run_exercise(&engine, exercise)?;
        let awarded = stats.register(&eval, elapsed);
        println!(
            "\n>>> +{} pts | streak {} | time {:.1}s | score {:.0}%\n",
            awarded,
            stats.streak,
            elapsed.as_secs_f32(),
            eval.score * 100.0
        );
        println!("\n---\n");
    }

    stats.print_summary();

    Ok(())
}

fn run_exercise(
    engine: &ExerciseEngine,
    exercise: &Exercise,
) -> Result<(Evaluation, Duration), Box<dyn std::error::Error>> {
    println!("# {} [{}]", exercise.title(), exercise.id());
    println!("Ready? Fast, clean judgments get max points.\n");

    let started = Instant::now();
    let eval = match exercise {
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
            engine.evaluate(exercise, &answer)?
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
            engine.evaluate(exercise, &answer)?
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
            engine.evaluate(exercise, &answer)?
        }
    };

    let elapsed = started.elapsed();
    println!(
        "\nResult: correct = {}, score = {:.2}\n{}",
        eval.correct, eval.score, eval.feedback
    );
    Ok((eval, elapsed))
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

fn award_points(eval: &Evaluation, elapsed: Duration, streak: usize) -> u32 {
    let base = (eval.score * 120.0).round() as u32 + if eval.correct { 50 } else { 15 };
    let speed_bonus = if elapsed.as_secs_f32() < 20.0 {
        15
    } else if elapsed.as_secs_f32() < 40.0 {
        8
    } else {
        0
    };
    let streak_bonus = streak.saturating_sub(1) as u32 * 5;

    base + speed_bonus + streak_bonus
}
