use crate::error::Error;
use crate::result::Result;
use spectre_consensus_core::constants::SOMPI_PER_SPECTRE;
use std::fmt::Display;

pub fn try_parse_required_nonzero_spectre_as_sompi_u64<S: ToString + Display>(spectre_amount: Option<S>) -> Result<u64> {
    if let Some(spectre_amount) = spectre_amount {
        let sompi_amount = spectre_amount
            .to_string()
            .parse::<f64>()
            .map_err(|_| Error::custom(format!("Supplied Spectre amount is not valid: '{spectre_amount}'")))?
            * SOMPI_PER_SPECTRE as f64;
        if sompi_amount < 0.0 {
            Err(Error::custom("Supplied Spectre amount is not valid: '{spectre_amount}'"))
        } else {
            let sompi_amount = sompi_amount as u64;
            if sompi_amount == 0 {
                Err(Error::custom("Supplied required spectre amount must not be a zero: '{spectre_amount}'"))
            } else {
                Ok(sompi_amount)
            }
        }
    } else {
        Err(Error::custom("Missing Spectre amount"))
    }
}

pub fn try_parse_required_spectre_as_sompi_u64<S: ToString + Display>(spectre_amount: Option<S>) -> Result<u64> {
    if let Some(spectre_amount) = spectre_amount {
        let sompi_amount = spectre_amount
            .to_string()
            .parse::<f64>()
            .map_err(|_| Error::custom(format!("Supplied Spectre amount is not valid: '{spectre_amount}'")))?
            * SOMPI_PER_SPECTRE as f64;
        if sompi_amount < 0.0 {
            Err(Error::custom("Supplied Spectre amount is not valid: '{spectre_amount}'"))
        } else {
            Ok(sompi_amount as u64)
        }
    } else {
        Err(Error::custom("Missing Spectre amount"))
    }
}

pub fn try_parse_optional_spectre_as_sompi_i64<S: ToString + Display>(spectre_amount: Option<S>) -> Result<Option<i64>> {
    if let Some(spectre_amount) = spectre_amount {
        let sompi_amount = spectre_amount
            .to_string()
            .parse::<f64>()
            .map_err(|_e| Error::custom(format!("Supplied Spectre amount is not valid: '{spectre_amount}'")))?
            * SOMPI_PER_SPECTRE as f64;
        if sompi_amount < 0.0 {
            Err(Error::custom("Supplied Spectre amount is not valid: '{spectre_amount}'"))
        } else {
            Ok(Some(sompi_amount as i64))
        }
    } else {
        Ok(None)
    }
}
