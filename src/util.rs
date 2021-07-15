use libc::{rlimit64, setrlimit64, RLIMIT_CPU, SIGXCPU};
use std::io::Write;
use std::{os::unix::prelude::*, process::Command};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

pub fn initialize_tracing() {
    let formatting_layer = BunyanFormattingLayer::new("judge".into(), std::io::stdout);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = Registry::default()
        .with(JsonStorageLayer)
        .with(env_filter)
        .with(formatting_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

pub fn gg() {
    let output = Command::new("nsjail")
        .args(&[
            "-Mo",
            "-Q",
            "--user",
            "256",
            "--group",
            "99999",
            "-R",
            "/bin/",
            "-R",
            "/lib",
            "-R",
            "/lib64/",
            "-R",
            "/usr/",
            "-R",
            "/sbin/",
            "-T",
            "/dev",
            "-R",
            "/dev/urandom",
            "--keep_caps",
            "--skip_setsid",
            "--rlimit_cpu",
            "10",
            "-R",
            "/home/nithin/Git/judge/prepare.sh",
            "-R",
            "/home/nithin/Git/judge/abc.py",
            "--",
            "/bin/bash",
            "/home/nithin/Git/judge/prepare.sh",
        ])
        .output()
        .expect("failed to execute process");

    println!("Output start");
    std::io::stdout().write_all(&output.stdout).unwrap();
    println!("Error start");
    std::io::stderr().write_all(&output.stderr).unwrap();
    println!("Other stuff");

    match output.status.signal() {
        Some(SIGXCPU) => println!("Time limit exceeded"),
        _ => println!("Failed due to unknown reason"),
    }
}
