use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use live_share_worker::e2e_worker_config::e2e_worker_config;
use live_share_worker::share_id::share_id_format;

/// `deploy.json`, republished as-is: the deployment values `cloudflare.config.ts`
/// also reads.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeployConfig {
    worker_host: String,
    d1_database_id: String,
}

#[test]
fn export_web_config() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../web/src/generated/live-share-worker");
    fs::create_dir_all(&out_dir)?;
    fs::write(
        out_dir.join("shareIdFormat.json"),
        serde_json::to_string_pretty(&share_id_format())?,
    )?;
    fs::write(
        out_dir.join("e2eWorkerConfig.json"),
        serde_json::to_string_pretty(&e2e_worker_config())?,
    )?;
    let deploy: DeployConfig = serde_json::from_str(&fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("deploy.json"),
    )?)?;
    fs::write(
        out_dir.join("deployConfig.json"),
        serde_json::to_string_pretty(&deploy)?,
    )?;
    Ok(())
}
