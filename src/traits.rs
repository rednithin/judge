use std::{
    path::{Path, PathBuf},
    process::Output,
};

use async_trait::async_trait;
// use std::io::Write;
use tokio::{fs, process::Command};
use uuid::Uuid;

#[async_trait]
pub trait LanguageExecutor {
    fn new(uuid: Uuid, code: String) -> Self;
    async fn prepare(&self) -> anyhow::Result<()>;
    async fn execute(&self) -> anyhow::Result<Output>;
    async fn teardown(&self) -> anyhow::Result<()>;
}

pub struct Python {
    file_directory: PathBuf,
    code: String,
}

#[async_trait]
impl LanguageExecutor for Python {
    fn new(uuid: Uuid, code: String) -> Self {
        Self {
            file_directory: Path::new("/tmp").join(uuid.to_string()),
            code,
        }
    }
    async fn prepare(&self) -> anyhow::Result<()> {
        fs::create_dir(&self.file_directory).await?;
        fs::write(
            self.file_directory.join("nsjail.config.proto"),
            format!(
                r#"
name: "python nsjail config"

description: "Python nsjail config"

mode: ONCE
hostname: "python"
log_level: ERROR

time_limit: 5

mount {{
    src: "/home/nithin/Git/judge/tar/judge_root_fs/"
    dst: "."
    is_bind: true
}}

mount {{
    src: {directory:?}
    dst: "/program"
    is_bind: true
}}
        "#,
                directory = self.file_directory
            ),
        )
        .await?;

        fs::write(
            self.file_directory.join("script.sh"),
            format!(
                r#"#!/bin/bash
ulimit -St 4
strace -e 'trace=!all' /usr/bin/python3 /program/code.py
        "#,
            ),
        )
        .await?;

        fs::write(self.file_directory.join("code.py"), &self.code).await?;

        Ok(())
    }

    async fn execute(&self) -> anyhow::Result<Output> {
        let output = Command::new("nsjail")
            .args(&[
                "--config",
                &format!(
                    "{}",
                    self.file_directory
                        .join("nsjail.config.proto")
                        .to_str()
                        .unwrap()
                ),
                "--",
                "/bin/bash",
                "/program/script.sh",
            ])
            .output()
            .await?;

        // println!("Output start");
        // std::io::stdout().write_all(&output.stdout).unwrap();
        // println!("Error start");
        // std::io::stderr().write_all(&output.stderr).unwrap();

        Ok(output)
    }

    async fn teardown(&self) -> anyhow::Result<()> {
        fs::remove_dir_all(&self.file_directory).await?;

        Ok(())
    }
}
