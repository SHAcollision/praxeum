use clap::{Parser, ValueEnum};
use crossterm::{
    cursor,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use inquire::{MultiSelect, Select};
use owo_colors::OwoColorize;
use praxeum_core::loader::{default_examples_path, DataFormat, ExerciseLoader, PreflightReport};
use praxeum_core::model::{answer::Answer, exercise::Exercise};
use praxeum_core::{Evaluation, ExerciseEngine, PraxeumError, SessionEvent, SessionTracker};
use rand::seq::SliceRandom;
use std::fmt;
use std::io::{self, stdout, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Turbo CLI demo for the Praxeum exercise engine.
#[derive(Parser, Debug)]
#[command(author, version, about = "Praxeum CLI demo")]
struct Cli {
    /// Path to exercises file (TOML or JSON).
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Input format (default: infer from extension or content).
    #[arg(long, value_enum, default_value_t = FormatArg::Auto)]
    format: FormatArg,

    /// Shuffle exercises before starting.
    #[arg(long, default_value_t = false)]
    shuffle: bool,

    /// Only include exercises whose id starts with the prefix.
    #[arg(long)]
    id_prefix: Option<String>,

    /// Limit how many exercises to play.
    #[arg(long)]
    limit: Option<usize>,

    /// Validate the file and exit without running the interactive flow.
    #[arg(long, default_value_t = false)]
    validate_only: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum FormatArg {
    Auto,
    Json,
    Toml,
}

impl From<FormatArg> for DataFormat {
    fn from(value: FormatArg) -> Self {
        match value {
            FormatArg::Auto => DataFormat::Auto,
            FormatArg::Json => DataFormat::Json,
            FormatArg::Toml => DataFormat::Toml,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let path = cli.file.clone().unwrap_or_else(default_examples_path);

    let format: DataFormat = cli.format.into();
    let preflight = ExerciseLoader::preflight_path(&path, format)?;
    print_preflight(&preflight);

    if !preflight.is_success() {
        if cli.validate_only {
            std::process::exit(1);
        }
        return Err(Box::new(PraxeumError::Validation(
            praxeum_core::validator::ValidationErrorReport::new(
                preflight.source.clone(),
                preflight.issues.clone(),
            ),
        )));
    } else if cli.validate_only {
        return Ok(());
    }

    let mut exercises = ExerciseLoader::load_from_path(&path, format)?;
    apply_filters(&mut exercises, &cli);

    if exercises.is_empty() {
        eprintln!("No exercises after applying filters.");
        return Ok(());
    }

    let engine = ExerciseEngine::new(exercises);
    let mut tracker = SessionTracker::new(engine.len());

    for (idx, exercise) in engine.iter().enumerate() {
        render_banner(engine.len(), &tracker, idx + 1, exercise.title());
        let (eval, elapsed) = run_exercise(&engine, exercise)?;
        let answered_event = tracker.record_answer(exercise, &eval, elapsed.as_millis());

        let SessionEvent::Answered { outcome, metrics } = answered_event else {
            unreachable!();
        };

        render_result_footer(&eval, elapsed, outcome.points_awarded, outcome.streak);
        if idx + 1 < engine.len() {
            pause_for_next()?;
        }

        if metrics.answered >= engine.len() {
            break;
        }
    }

    clear_screen();
    print_summary(&tracker);

    Ok(())
}

fn apply_filters(exercises: &mut Vec<Exercise>, cli: &Cli) {
    if let Some(prefix) = &cli.id_prefix {
        exercises.retain(|ex| ex.id().starts_with(prefix));
    }

    if cli.shuffle {
        exercises.shuffle(&mut rand::thread_rng());
    }

    if let Some(limit) = cli.limit {
        exercises.truncate(limit.min(exercises.len()));
    }
}

fn print_preflight(report: &PreflightReport) {
    println!(
        "Validating {} (format: {:?})",
        report.source.as_deref().unwrap_or("<memory>"),
        report.format
    );

    if report.is_success() {
        println!("{}", "✓ Schema validation passed".green());
        println!("Exercises detected: {}", report.exercise_count);
    } else {
        println!("{}", "Validation issues found:".red().bold());
        for issue in &report.issues {
            println!("  - {}", issue);
        }
    }
    println!();
}

fn clear_screen() {
    let mut out = stdout();
    let _ = out.execute(Clear(ClearType::All));
    let _ = out.execute(cursor::MoveTo(0, 0));
}

fn line() -> String {
    "─".repeat(64)
}

fn render_banner(total: usize, tracker: &SessionTracker, position: usize, title: &str) {
    clear_screen();
    let metrics = tracker.metrics();
    println!("{}", "Praxeum CLI".bold().green());
    println!("{}", "Positional drills for Austrian economics".dimmed());
    println!("{}", line());
    println!(
        "{} {}/{}   {} {}   {} {:.0}%   {} {}",
        "Position".bold(),
        position,
        total,
        "Streak".bold(),
        metrics.current_streak,
        "Avg".bold(),
        tracker.average_score() * 100.0,
        "Points".bold(),
        tracker.points()
    );
    println!("{}", title.bold().cyan());
    println!("{}", line());
    println!();
}

fn render_result_footer(eval: &Evaluation, elapsed: Duration, awarded: u32, streak: usize) {
    let status_icon = if eval.correct {
        format!("{}", "✓".green().bold())
    } else {
        format!("{}", "✗".red().bold())
    };

    let status_text = if eval.correct {
        format!("{}", "On point".green().bold())
    } else {
        format!("{}", "Keep refining".yellow().bold())
    };

    println!("{}", line());
    println!(
        "{} {} ({:.0}% score, {:.1}s)",
        status_icon,
        status_text,
        eval.score * 100.0,
        elapsed.as_secs_f32()
    );
    println!("{}", eval.feedback);
    println!();
    println!(
        "{} +{} pts | streak {}",
        "Momentum".bold().yellow(),
        awarded,
        streak
    );
}

fn pause_for_next() -> io::Result<()> {
    print!("\nPress Enter for the next position...");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(())
}

fn run_exercise(
    engine: &ExerciseEngine,
    exercise: &Exercise,
) -> Result<(Evaluation, Duration), Box<dyn std::error::Error>> {
    println!("{} {}", "ID".bold().cyan(), exercise.id().dimmed());
    println!(
        "{}",
        "Ready? Fast, clean judgments get max points.".dimmed()
    );
    println!();

    let started = Instant::now();
    let eval = match exercise {
        Exercise::Classification {
            prompt,
            categories,
            items,
            ..
        } => {
            println!("{}", prompt.bold());
            println!(
                "{}",
                "Use ↑ ↓ to pick a category, Enter to lock it.".dimmed()
            );
            println!();
            let mut indices = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                println!("{} {}", format!("Item {}:", i + 1).bold(), item.text);
                let idx = select_index(&format!("Category for item {}", i + 1), categories)?;
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
            println!("{}", prompt.bold());
            println!();
            let indices = if *multi_select {
                multi_pick_indices("Pick all correct answers", options)?
            } else {
                vec![select_index("Pick your answer", options)?]
            };
            let answer = Answer::MultipleChoice(indices);
            engine.evaluate(exercise, &answer)?
        }

        Exercise::Scenario {
            description,
            prompt,
            choices,
            ..
        } => {
            println!("{}", description.bold());
            println!();
            println!("{}", prompt);
            println!();
            let choice_labels: Vec<String> = choices
                .iter()
                .enumerate()
                .map(|(i, c)| format!("[{}] {}", i + 1, c.label))
                .collect();
            let idx = select_index("Choose option", &choice_labels)?;
            let answer = Answer::Scenario(idx);
            engine.evaluate(exercise, &answer)?
        }
    };

    let elapsed = started.elapsed();
    println!("\n{}", line());
    Ok((eval, elapsed))
}

#[derive(Clone)]
struct ChoiceItem {
    label: String,
    index: usize,
}

impl fmt::Display for ChoiceItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

fn select_index(prompt: &str, options: &[String]) -> Result<usize, Box<dyn std::error::Error>> {
    let items: Vec<ChoiceItem> = options
        .iter()
        .enumerate()
        .map(|(i, text)| ChoiceItem {
            label: format!("[{}] {}", i + 1, text),
            index: i,
        })
        .collect();

    let picked = Select::new(prompt, items)
        .with_help_message("Use ↑ ↓ to navigate, Enter to confirm")
        .prompt()?;

    Ok(picked.index)
}

fn multi_pick_indices(
    prompt: &str,
    options: &[String],
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let items: Vec<ChoiceItem> = options
        .iter()
        .enumerate()
        .map(|(i, text)| ChoiceItem {
            label: format!("[{}] {}", i + 1, text),
            index: i,
        })
        .collect();

    let picked = MultiSelect::new(prompt, items)
        .with_help_message("Space to toggle, Enter to submit")
        .with_vim_mode(true)
        .prompt()?;

    Ok(picked.into_iter().map(|item| item.index).collect())
}

fn print_summary(tracker: &SessionTracker) {
    let metrics = tracker.metrics();
    println!("\n{}", "Session Summary".bold().green());
    println!("{}", line());
    println!(
        "Exercises: {} | Correct: {} | Avg score: {:.0}%",
        metrics.answered,
        metrics.correct,
        tracker.average_score() * 100.0
    );
    let total_secs = metrics.total_duration_ms as f32 / 1000.0;
    println!(
        "Points: {} | Best streak: {} | Total time: {:.1}s",
        tracker.points(),
        metrics.longest_streak,
        total_secs
    );
}
