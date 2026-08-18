use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use chrono::{Utc, Duration};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LicenseFile {
    pub key: String,
    pub email: String,
    pub validated_at: chrono::DateTime<Utc>,
}

fn license_path() -> PathBuf {
    let mut p = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    p.push("bnn-code");
    fs::create_dir_all(&p).ok();
    p.push("license.json");
    p
}

pub fn load() -> Option<LicenseFile> {
    let s = fs::read_to_string(license_path()).ok()?;
    serde_json::from_str(&s).ok()
}

pub fn save(f: &LicenseFile) -> anyhow::Result<()> {
    fs::write(license_path(), serde_json::to_string_pretty(f)?)?;
    Ok(())
}

pub fn validate_online(key: &str) -> anyhow::Result<LicenseFile> {
    let client = reqwest::blocking::Client::new();
    let res = client.post("https://api.lemonsqueezy.com/v1/licenses/validate")
       .json(&serde_json::json!({"license_key": key}))
       .send()?
       .json::<serde_json::Value>()?;

    if!res["valid"].as_bool().unwrap_or(false) {
        anyhow::bail!("License ไม่ถูกต้อง");
    }

    Ok(LicenseFile {
        key: key.to_string(),
        email: res["meta"]["customer_email"].as_str().unwrap_or("pro@user.com").to_string(),
        validated_at: Utc::now(),
    })
}

// นี่คือตัวเช็คหลัก
pub fn is_pro() -> bool {
    // ให้ dev ที่ build ด้วย --features pro เทสได้เลยไม่ต้องมี key
    #[cfg(feature = "pro")]
    {
        if std::env::var("BNN_DEV").is_ok() {
            return true;
        }
    }

    let Some(lic) = load() else { return false };
    let age = Utc::now() - lic.validated_at;

    // offline ได้ 7 วัน
    if age < Duration::days(7) {
        return true;
    }
    // เกิน 7 วัน ลองเช็คออนไลน์เงียบๆ
    if let Ok(new_lic) = validate_online(&lic.key) {
        let _ = save(&new_lic);
        return true;
    }
    // grace period 14 วันถ้าไม่มีเน็ต
    age < Duration::days(14)
}

#[macro_export]
macro_rules! require_pro {
    ($feat:expr) => {
        if!crate::utils::license::is_pro() {
            eprintln!("🚫 ฟีเจอร์ '{}' ต้องใช้ bnn-code Pro", $feat);
            eprintln!(" ซื้อที่: https://bnn-code.lemonsqueezy.com/checkout");
            eprintln!(" แล้วรัน: bnn-code license activate YOUR_KEY");
            std::process::exit(1);
        }
    };
}
