use ed25519_dalek::Keypair;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

fn main() {
    let num_threads = num_cpus::get();

    println!("bruteforcing with {num_threads} threads");

    let mut to_threads = Vec::new();
    let (to_controller, from_threads) = sync_channel(64);

    for _ in 0..num_threads {
        let (to_thread, from_controller) = sync_channel(64);
        to_threads.push(to_thread);

        let to_controller = to_controller.clone();
        std::thread::spawn(|| {
            run_thread(from_controller, to_controller);
        });
    }

    run_controller(to_threads, from_threads);
}

fn run_controller(to_threads: Vec<SyncSender<u64>>, from_threads: Receiver<Keypair>) {
    let mut rng = rand::thread_rng();
    let mut largest = Keypair::generate(&mut rng);
    let mut largest_value = u64::from_be_bytes(largest.public.as_bytes()[0..8].try_into().unwrap());

    for sender in &to_threads {
        sender.send(largest_value).unwrap();
    }

    while let Ok(keypair) = from_threads.recv() {
        let value = u64::from_be_bytes(keypair.public.as_bytes()[0..8].try_into().unwrap());

        if value > largest_value {
            largest_value = value;

            for sender in &to_threads {
                sender.send(largest_value).unwrap();
            }

            largest = keypair;
            println!(
                "{} <-> {}",
                hex::encode(&largest.public.as_bytes()),
                hex::encode(&largest.secret.as_bytes())
            );
        }
    }
}

fn run_thread(from_controller: Receiver<u64>, to_controller: SyncSender<Keypair>) {
    let mut rng = rand::thread_rng();
    let mut largest_value = from_controller.recv().unwrap();

    loop {
        for _ in 0..1024 {
            let pair = Keypair::generate(&mut rng);
            let public_key = &pair.public;

            let value = u64::from_be_bytes(public_key.as_bytes()[0..8].try_into().unwrap());

            if value > largest_value {
                to_controller.send(pair).unwrap();
                largest_value = value;
            }
        }

        if let Ok(largest_found) = from_controller.try_recv() {
            largest_value = largest_found;
        }
    }
}
