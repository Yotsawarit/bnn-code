use anyhow::Result;
use std::path::Path;
use tokenizers::Tokenizer as HfTokenizer;

pub struct Tokenizer {
    tokenizer: HfTokenizer,
}

pub struct Encoding {
    pub ids: Vec<i64>,
    pub attention_mask: Vec<i64>,
}

impl Encoding {
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

impl Tokenizer {
    pub fn new(tokenizer_path: &Path) -> Result<Self> {
        let tokenizer = HfTokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Tokenizer load error from {:?}: {:?}", tokenizer_path, e))?;
        Ok(Self { tokenizer })
    }

    pub fn encode(&self, text: &str, max_length: usize) -> Result<Encoding> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!("Tokenizer encode error: {:?}", e))?;

        let ids: Vec<i64> = encoding
            .get_ids()
            .iter()
            .map(|&x| x as i64)
            .take(max_length)
            .collect();
        let attention_mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&x| x as i64)
            .take(max_length)
            .collect();

        Ok(Encoding { ids, attention_mask })
    }

    pub fn decode(&self, ids: &[i64]) -> Result<String> {
        let ids_u32: Vec<u32> = ids.iter().map(|&x| x as u32).collect();
        let text = self
            .tokenizer
            .decode(&ids_u32, true)
            .map_err(|e| anyhow::anyhow!("Tokenizer decode error: {:?}", e))?;
        Ok(text)
    }
}
