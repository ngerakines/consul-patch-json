use anyhow::{anyhow, Error, Result};
use consulrs::{
    api::kv::requests::SetKeyRequest,
    client::{ConsulClient, ConsulClientSettingsBuilder},
    kv,
};
use json_patch::{merge, patch, Patch};
use serde_json::Value;
use std::{env, str};

#[tokio::main]
async fn main() -> Result<()> {
    let client = ConsulClient::new(ConsulClientSettingsBuilder::default().build().unwrap())?;

    let mut args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!(
            "Usage: {} <path> <key>=<value> [... <key>=<value>]",
            args[0]
        );
        return Ok(());
    }
    args.remove(0);

    let dry_run = if args.contains(&"--dry-run".to_string()) {
        args.retain(|arg| arg != "--dry-run");
        true
    } else {
        false
    };

    let json_patch = if args.contains(&"--json-patch".to_string()) {
        args.retain(|arg| arg != "--json-patch");
        true
    } else {
        false
    };

    let mut keys: Vec<String> = vec![];
    let mut patches: Vec<(Option<String>, Value)> = Vec::new();

    for arg in args.iter() {
        if arg.contains('=') || arg == "--" {
            continue;
        }
        keys.push(arg.to_string());
    }
    args.retain(|arg| arg.contains('=') || arg == "--");

    if keys.is_empty() {
        return Err(anyhow!("one or more keys must be specified"));
    }

    let read_stdin =
        args.contains(&"--".to_string()) || args.iter().any(|arg| arg.ends_with("=--"));
    let stdin_value = if read_stdin {
        Some(stdin_input()?)
    } else {
        None
    };

    for arg in args.iter() {
        let patch = parse_patch(arg, stdin_value.clone())?;
        patches.push(patch);
    }

    for key_name in keys {
        let key_meta = kv::read(&client, &key_name, None).await?;

        if key_meta.response.len() != 1 {
            return Err(anyhow!("consul response must contain exactly one key"));
        }

        for key in key_meta.response {
            let encoded_value: Vec<u8> = key.value.unwrap().try_into()?;
            let decoded_value = str::from_utf8(&encoded_value)?;

            let mut json_value: Value = serde_json::from_str(decoded_value)?;

            if !json_value.is_object() {
                return Err(anyhow!("only object values are supported"));
            }

            for (patch_key, patch_value) in patches.iter() {
                if patch_key.is_some() {
                    json_value[patch_key.clone().unwrap()] = patch_value.clone();
                } else if json_patch {
                    let p: Patch = serde_json::from_value(patch_value.clone())?;
                    patch(&mut json_value, &p)?;
                } else {
                    merge(&mut json_value, patch_value);
                }
            }

            if dry_run {
                println!("{}", serde_json::to_string_pretty(&json_value)?);
                continue;
            }
            kv::set_json(
                &client,
                &key_name,
                &json_value,
                Some(SetKeyRequest::builder().cas(key.modify_index)),
            )
            .await?;
        }
    }

    Ok(())
}

fn parse_patch(patch: &str, stdin_value: Option<Value>) -> Result<(Option<String>, Value)> {
    if patch == "--" {
        return Ok((None, stdin_value.unwrap()));
    }
    let (patch_key, patch_value) = patch
        .trim()
        .split_once('=')
        .ok_or_else(|| anyhow!("invalid patch: {}", patch))?;
    if patch_value == "--" {
        return Ok((Some(patch_key.to_string()), stdin_value.unwrap()));
    }
    let json_value: Value = serde_json::from_str(patch_value)
        .map_err(|err| Error::msg(err).context(patch.to_string()))?;
    Ok((Some(patch_key.to_string()), json_value))
}

use std::io::{stdin, BufRead};

fn stdin_input() -> Result<Value> {
    let mut lines = Vec::new();
    let input = stdin();
    let mut stream = input.lock();

    let mut line = String::new();

    while let Ok(n) = stream.read_line(&mut line) {
        if n == 0 {
            break;
        }

        lines.push(line);
        line = String::new();
    }

    let joined_lines = lines.join("");

    serde_json::from_str::<Value>(&joined_lines)
        .map_err(|err| Error::msg(err).context(format!("'{}'", joined_lines)))
}
