use anyhow::Context;
use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    time::Duration,
};

fn main() {
    if let Err(e) = run_main() {
        eprintln!("error running edbrute: {e}");
    }
}

fn run_main() -> anyhow::Result<()> {
    let num_threads = num_cpus::get();

    println!("bruteforcing with {num_threads} threads");

    let spinner = setup_spinner();

    let mut to_threads = Vec::new();
    let (to_controller, from_threads) = sync_channel(64);

    for _ in 0..num_threads {
        let (to_thread, from_controller) = sync_channel(64);
        to_threads.push(to_thread);

        let spinner = spinner.clone();
        let to_controller = to_controller.clone();
        std::thread::spawn(move || {
            run_worker(spinner, from_controller, to_controller);
        });

        std::thread::sleep(Duration::from_millis(100));
    }

    run_controller(spinner, to_threads, from_threads)
        .context("unable to start controller thread")?;

    Ok(())
}

fn run_controller(
    spinner: ProgressBar,
    to_threads: Vec<SyncSender<u64>>,
    from_threads: Receiver<Keypair>,
) -> anyhow::Result<()> {
    let (mut checkpoint_file, saved_largest_keypair) =
        checkpoint_with_largest_keypair("checkpoint.log")
            .context("unable to create checkpoint file")?;

    let mut largest_keypair =
        saved_largest_keypair.unwrap_or_else(|| Keypair::generate(&mut rand::thread_rng()));

    let mut largest_value = public_key_to_u64(&largest_keypair);
    for sender in &to_threads {
        sender.send(largest_value).unwrap();
    }

    let public_pretty = pretty_print_public(&largest_keypair);
    spinner.set_message(public_pretty);

    while let Ok(keypair) = from_threads.recv() {
        let value = public_key_to_u64(&keypair);

        if value > largest_value {
            largest_value = value;

            for sender in &to_threads {
                sender.send(largest_value).unwrap();
            }

            largest_keypair = keypair;

            writeln!(checkpoint_file, "{}", serialize_keypair(&largest_keypair))
                .context("unable to save keypair to checkpoint file")?;
            checkpoint_file.flush()?;

            let printed_keypair = pretty_print_public(&largest_keypair);
            spinner.println(format!(
                "[{}] {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                &printed_keypair
            ));
            spinner.set_message(printed_keypair);
        }
    }

    Ok(())
}

fn run_worker(
    spinner: ProgressBar,
    from_controller: Receiver<u64>,
    to_controller: SyncSender<Keypair>,
) {
    let mut rng = rand::thread_rng();
    let mut largest_value = from_controller.recv().unwrap();

    let iteration_delta = u16::MAX as usize;
    loop {
        for _ in 0..iteration_delta {
            let pair = Keypair::generate(&mut rng);
            let value = public_key_to_u64(&pair);

            if value > largest_value {
                to_controller.send(pair).unwrap();
                largest_value = value;
            }
        }

        spinner.inc(iteration_delta as u64);
        while let Ok(largest_found) = from_controller.try_recv() {
            largest_value = largest_found;
        }
    }
}

fn pretty_print_public(keypair: &Keypair) -> String {
    hex::encode(keypair.public)
}

fn serialize_keypair(keypair: &Keypair) -> String {
    format!(
        "{},{}",
        hex::encode(keypair.public.as_bytes()),
        hex::encode(keypair.secret.as_bytes())
    )
}

fn public_key_to_u64(keypair: &Keypair) -> u64 {
    u64::from_be_bytes(keypair.public.as_bytes()[0..8].try_into().unwrap())
}

fn checkpoint_with_largest_keypair(
    path: impl AsRef<Path>,
) -> anyhow::Result<(File, Option<Keypair>)> {
    let checkpoint_file = std::fs::File::options()
        .create(true)
        .read(true)
        .append(true)
        .open(path)
        .context("unable to open checkpoint file")?;

    let reader = BufReader::new(&checkpoint_file);

    let mut keypairs = Vec::new();
    for line in reader.lines().flatten() {
        let (public_hex, secret_hex) = line.split_once(',').context("malformed keypair line")?;
        let (public_bytes, secret_bytes) = (hex::decode(public_hex)?, hex::decode(secret_hex)?);
        let (public, secret) = (
            PublicKey::from_bytes(&public_bytes)?,
            SecretKey::from_bytes(&secret_bytes)?,
        );

        keypairs.push(Keypair { public, secret })
    }

    let starting_key = keypairs.into_iter().max_by_key(public_key_to_u64);
    Ok((checkpoint_file, starting_key))
}

fn setup_spinner() -> ProgressBar {
    let spinner = indicatif::ProgressBar::new_spinner();

    spinner.enable_steady_tick(Duration::from_millis(150));
    spinner.set_style(
        ProgressStyle::with_template(
            "\n{spinner} [{elapsed_precise}] {smoothed_per_sec}, {human_pos} total.\n  largest: {msg}",
        )
        .unwrap()
        .with_key(
            "smoothed_per_sec",
            |s: &ProgressState, w: &mut dyn std::fmt::Write| match (
                s.pos(),
                s.elapsed().as_millis(),
            ) {
                (pos, elapsed_ms) if elapsed_ms > 0 => {
                    write!(w, "{:.2} keys/s", pos as f64 * 1000_f64 / elapsed_ms as f64).unwrap()
                }
                _ => write!(w, "-").unwrap(),
            },
        ),
    );

    spinner
}
