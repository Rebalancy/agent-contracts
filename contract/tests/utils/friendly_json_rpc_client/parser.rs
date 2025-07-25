use std::error::Error;

use shade_agent_contract::types::{ActivityLog, ChainId};

pub trait ParseResult: Sized {
    fn parse(result_str: String) -> Result<Self, Box<dyn Error>>;
}

impl ParseResult for Vec<u8> {
    fn parse(result_str: String) -> Result<Self, Box<dyn Error>> {
        let result_bytes: Self = result_str
            .trim_matches(|c| c == '[' || c == ']') // Eliminar corchetes
            .split(',') // Dividir por comas
            .map(|s| s.trim().parse::<u8>().unwrap()) // Convertir cada parte a u8
            .collect();
        Ok(result_bytes)
    }
}

impl ParseResult for u128 {
    fn parse(result_str: String) -> Result<Self, Box<dyn Error>> {
        let value = result_str.trim_matches('"').parse::<Self>()?;
        Ok(value)
    }
}

impl ParseResult for String {
    fn parse(result_str: String) -> Result<Self, Box<dyn Error>> {
        Ok(result_str)
    }
}

impl ParseResult for Vec<Vec<u8>> {
    fn parse(result_str: String) -> Result<Self, Box<dyn Error>> {
        let cleaned = result_str.trim_matches(|c| c == '[' || c == ']');

        // Split into inner arrays
        let inner_strs = cleaned
            .split("],[")
            .map(|s| s.trim_matches(|c| c == '[' || c == ']'));

        let mut result = Vec::new();

        for inner in inner_strs {
            let vec = inner
                .split(',')
                .map(|s| s.trim().parse::<u8>())
                .collect::<Result<Vec<u8>, _>>()?;
            result.push(vec);
        }

        Ok(result)
    }
}

impl ParseResult for Vec<ActivityLog> {
    fn parse(result_str: String) -> Result<Self, Box<dyn Error>> {
        let logs: Vec<ActivityLog> = serde_json::from_str(&result_str)?;
        Ok(logs)
    }
}

impl ParseResult for Vec<(ChainId, u128)> {
    fn parse(result_str: String) -> Result<Self, Box<dyn Error>> {
        let raw: Vec<(ChainId, u64)> = serde_json::from_str(&result_str)?;
        Ok(raw.into_iter().map(|(id, amt)| (id, amt as u128)).collect())
    }
}
