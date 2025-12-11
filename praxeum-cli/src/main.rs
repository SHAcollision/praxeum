use clap::Parser;
use crossterm::{
    cursor,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use inquire::{MultiSelect, Select};
use owo_colors::OwoColorize;
use praxeum_core::loader::{DataFormat, ExerciseLoader};
use praxeum_core::model::{answer::Answer, exercise::Exercise};
use praxeum_core::{Evaluation, ExerciseEngine};
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
    fn avg_score(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.total_score / self.total as f32) * 100.0
        }
    }

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
        println!("\n{}", "Session Summary".bold().green());
        println!("{}", line());
        println!(
            "Exercises: {} | Correct: {} | Avg score: {:.0}%",
            self.total,
            self.correct,
            self.avg_score()
        );
        println!(
            "Points: {} | Best streak: {} | Total time: {:.1}s",
            self.points,
            self.best_streak,
            self.total_time.as_secs_f32()
        );
    }
}

fn clear_screen() {
    let mut out = stdout();
    let _ = out.execute(Clear(ClearType::All));
    let _ = out.execute(cursor::MoveTo(0, 0));
}

fn line() -> String {
    "─".repeat(64)
}

fn render_banner(total: usize, stats: &SessionStats, position: usize, title: &str) {
    clear_screen();
    println!("{}", "Praxeum CLI".bold().green());
    println!("{}", "Positional drills for Austrian economics".dimmed());
    println!("{}", line());
    println!(
        "{} {}/{}   {} {}   {} {:.0}%   {} {}",
        "Position".bold(),
        position,
        total,
        "Streak".bold(),
        stats.streak,
        "Avg".bold(),
        stats.avg_score(),
        "Points".bold(),
        stats.points
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

    for (idx, exercise) in engine.iter().enumerate() {
        render_banner(engine.len(), &stats, idx + 1, exercise.title());
        let (eval, elapsed) = run_exercise(&engine, exercise)?;
        let awarded = stats.register(&eval, elapsed);
        render_result_footer(&eval, elapsed, awarded, stats.streak);
        if idx + 1 < engine.len() {
            pause_for_next()?;
        }
    }

    clear_screen();
    stats.print_summary();

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
