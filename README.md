# AWS Key Rotator

This program allows you to renew your AWS API key.
MFA code is mandatory because the process has been designed to fit with Sopra Banking Software organisation and security requirements. 
Since you can have only two keys per user, you must have one *inactive* key before run the script. 

## Process
 
Delete disabled key > Create new key > Save new key in local config file > Disable old key

## Configuration

You must insert the `[automation]` configuration in the `~/.aws/config` file:

```
[automation]
profile = central
username = maxime.stevenot@soprabanking.com
mfa_arn = arn:aws:iam::*****
```

## Run it

You must enter your MFA token after 

```console
$ cargo run
```

## TODO List

This project is my lab to learn how to develop in Rust. Feel free to contribute!

- [ ] Write unit tests
- [ ] Improve asynchronous programming
- [ ] Review errors management
