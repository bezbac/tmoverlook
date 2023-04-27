use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressState, ProgressStyle};
use log::info;
use std::{fmt::Write, process::Command, time::Duration};

pub fn run() -> Result<()> {
    info!("Starting analyze process");

    let m = MultiProgress::new();
    let sty = ProgressStyle::with_template("{spinner} [{elapsed_precise}] {wide_msg}")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    let pb = m.add(ProgressBar::new(0));
    pb.set_style(sty.clone());
    pb.set_message("Getting list of backups");
    pb.enable_steady_tick(Duration::from_millis(100));

    let backups = Command::new("tmutil")
        .arg("listbackups")
        .output()
        .expect("Failed to execute tmutil");

    pb.finish();

    let backups = String::from_utf8(backups.stdout)?;

    let backup_paths = backups
        .split("\n")
        .filter(|p| !p.trim().is_empty())
        .collect::<Vec<&str>>();

    let mut commands = vec![];

    // Compare current state (tmutil compare -aX)
    commands.push(vec![]);

    // Compare past backups
    for (compare_path, compare_to_path) in backup_paths[1..].iter().zip(backup_paths.iter()) {
        commands.push(vec![compare_path, compare_to_path])
    }

    for args in commands {
        let pb = m.add(ProgressBar::new(0));

        let mut final_args = vec!["compare", "-aX"];
        final_args.extend(args);

        pb.set_style(sty.clone());
        pb.set_message(format!(
            "Analyzing backup (tmutil {:?})",
            final_args.join(" ")
        ));
        pb.enable_steady_tick(Duration::from_millis(100));

        let backups = Command::new("tmutil")
            .args(&final_args)
            .output()
            .expect("Failed to execute tmutil");

        pb.finish();
    }

    Ok(())
}
