use std::io::{stdin, stdout, Write};

use anyhow::{Context, Result};
use inflector::Inflector;

use crate::aws::key_rotator::AwsKeyRotator;

mod aws;

fn get_answer(prompt: &str) -> Result<String> {
    print!("{}: ", prompt.to_title_case());
    let _r = stdout().flush();

    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .context(format!("Error while reading: '{}'", prompt))?;

    Ok(input.trim_end().to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mfa = get_answer("enter your mfa")?;

    let mut aws_key_rotator = AwsKeyRotator::new(mfa.as_str());
    aws_key_rotator.process().await;

    Ok(())
}
