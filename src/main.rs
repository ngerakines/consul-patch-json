use anyhow::{anyhow, Error, Result};
use consulrs::{
    api::kv::requests::SetKeyRequest,
    client::{ConsulClient, ConsulClientSettingsBuilder},
    kv,
};
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

    let mut keys: Vec<String> = vec![];
    let mut patches: Vec<(String, Value)> = Vec::new();

    for arg in args.iter() {
        if !arg.contains('=') {
            keys.push(arg.to_string());
        }
    }
    args.retain(|arg| arg.contains('='));

    if keys.len() != 1 {
        return Err(anyhow!("only one key must be provided"));
    }

    for arg in args.iter() {
        let patch = parse_patch(arg)?;
        patches.push(patch);
    }

    for key_name in keys {
        let key_meta = kv::read(&client, &key_name, None).await?;
        println!("key: {:?}", key_meta);

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
                json_value[patch_key] = patch_value.clone();
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

fn parse_patch(patch: &str) -> Result<(String, Value)> {
    let (patch_key, patch_value) = patch
        .trim()
        .split_once('=')
        .ok_or_else(|| anyhow!("invalid patch: {}", patch))?;
    let json_value: Value = serde_json::from_str(patch_value)
        .map_err(|err| Error::msg(err).context(patch.to_string()))?;
    Ok((patch_key.to_string(), json_value))
}
