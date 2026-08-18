use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TotpAccount {
    label: String,
    issuer: String,
    account: String,
    secret: String,
    digits: u32,
    period: u64,
}

impl TotpAccount {
    pub fn new(
        label: impl Into<String>,
        issuer: impl Into<String>,
        account: impl Into<String>,
        secret: impl Into<String>,
    ) -> Self {
        let secret = secret.into();
        Self {
            label: label.into(),
            issuer: issuer.into(),
            account: account.into(),
            secret,
            digits: 6,
            period: 30,
        }
    }

    pub fn set_digits(&mut self, digits: u32) {
        self.digits = digits;
    }

    pub fn set_period(&mut self, period: u64) {
        self.period = period;
    }

    pub fn generate_code(&self) -> Result<String> {
        let secret_bytes = base32::decode(&self.secret)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        let time_step = now / self.period;

        let mut hmac = hmac::HMAC::new(hmac::SHA1, &secret_bytes);
        hmac.update(&time_step.to_be_bytes());
        let result = hmac.finalize();
        let code_bytes = result.into_bytes();

        let offset = (code_bytes[code_bytes.len() - 1] & 0x0f) as usize;
        let binary =
            ((code_bytes[offset] & 0x7f) as u32) << 24
                | ((code_bytes[offset + 1] & 0xff) as u32) << 16
                | ((code_bytes[offset + 2] & 0xff) as u32) << 8
                | (code_bytes[offset + 3] & 0xff) as u32;

        let code = binary % 10u32.pow(self.digits);
        Ok(format!("{:06}", code))
    }

    pub fn verify_code(&self, code: &str) -> Result<bool> {
        let generated = self.generate_code()?;
        Ok(generated == code)
    }

    pub fn to_uri(&self) -> String {
        let mut uri = format!("otpauth://totp/{}:{}?", self.label, self.account);
        uri.push_str(&format!("secret={}", self.secret));
        uri.push_str(&format!("&digits={}", self.digits));
        uri.push_str(&format!("&period={}", self.period));
        if !self.issuer.is_empty() {
            uri.push_str(&format!("&issuer={}", self.issuer));
        }
        uri
    }
}