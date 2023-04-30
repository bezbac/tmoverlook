use crate::utils::tmutil::{compare, CompareArgs};
use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressState, ProgressStyle};
use log::info;
use std::{fmt::Write, time::Duration};

pub fn run() -> Result<()> {
    info!("Starting analyze process");

    let m = MultiProgress::new();
    let sty = ProgressStyle::with_template("{spinner} [{elapsed_precise}] {wide_msg}")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    // let pb = m.add(ProgressBar::new(0));
    // pb.set_style(sty.clone());
    // pb.set_message("Getting list of backups");
    // pb.enable_steady_tick(Duration::from_millis(100));

    // let backup_paths = list_backups()?;

    // pb.finish();

    let mut commands = vec![];

    // Compare current state (tmutil compare -aX)
    commands.push(CompareArgs::Current);

    // // Compare past backups
    // for (compare_path, compare_to_path) in backup_paths[1..].iter().zip(backup_paths.iter()) {
    //     commands.push(CompareArgs::Backups {
    //         first: compare_path.to_string(),
    //         second: compare_to_path.to_string(),
    //     })
    // }

    for args in commands {
        let pb = m.add(ProgressBar::new(0));

        pb.set_style(sty.clone());
        pb.set_message(format!("Analyzing backup (tmutil {:?})", &args));
        pb.enable_steady_tick(Duration::from_millis(100));

        compare(&args)?;

        pb.finish();
    }

    Ok(())
}
