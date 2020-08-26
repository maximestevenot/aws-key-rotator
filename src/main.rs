use std::io::{stdin, stdout, Write};
use std::str::FromStr;
use anyhow::{Context, Result, anyhow};
use inflector::Inflector;

use crate::aws::key_rotator::AwsKeyRotator;

mod aws;

fn get_answer(prompt: &str) -> Result<u32> {
    print!("{}: ", prompt.to_title_case());
    let _r = stdout().flush();

    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .context(format!("Error while reading: '{}'", prompt))?;

    let parsed_input = input.trim_end();
    if parsed_input.len() == 6 {
        Ok(u32::from_str(parsed_input)?)
    } else {
        Err(anyhow!("Input should be 6 character long"))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mfa = get_answer("enter your mfa")?;

    let mut aws_key_rotator = AwsKeyRotator::new(&mfa.to_string());
    aws_key_rotator.process().await;

    Ok(())
}
