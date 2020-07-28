use std::path::PathBuf;

use dirs::home_dir;
use ini::Ini;
use rusoto_iam::AccessKey;

fn aws_credentials_file_path() -> PathBuf {
    let mut filename = home_dir().unwrap();
    filename.push(".aws");
    filename.push("credentials");
    filename
}

fn get_aws_key_from_file(aws_account: &str) -> AccessKey {
    let conf = Ini::load_from_file(aws_credentials_file_path().as_path()).unwrap();
    let section = conf.section(Some(aws_account)).unwrap();
    AccessKey {
        access_key_id: String::from(section.get("aws_access_key_id").unwrap()),
        secret_access_key: String::from(section.get("aws_secret_access_key").unwrap()),
        create_date: None,
        status: String::default(),
        user_name: String::default(),
    }
}

fn main() {
    let old_key = get_aws_key_from_file("central");
    println!("{:?}", old_key)
}

