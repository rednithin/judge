use libc::{rlimit64, setrlimit64, RLIMIT_CPU, SIGXCPU};
use std::io::Write;
use std::{os::unix::prelude::*, process::Command};

trait Isolate {
    fn isolate(&mut self) -> &mut Self;
}

impl Isolate for Command {
    fn isolate(&mut self) -> &mut Self {
        unsafe {
            self.pre_exec(move || {
                setrlimit64(
                    RLIMIT_CPU,
                    &rlimit64 {
                        rlim_cur: 5,
                        rlim_max: 10,
                    },
                );
                Ok(())
            })
        }
    }
}

fn main() {
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
        .isolate()
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
