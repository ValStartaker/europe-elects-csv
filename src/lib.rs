mod errors;
use chrono::NaiveDate;
use csv::ReaderBuilder;
use errors::{GetDateRangeError, GetJurisdictionError, PollTableTryFromPathError};
use jurisdiction::Jurisdiction;
use serde::{Deserialize, Deserializer};
use std::{collections::HashMap, path::Path, str::FromStr};

type DateRange = u32;

#[derive(Debug)]
pub struct PollTable {
    pub polls: Vec<Poll>,
    pub jurisdiction: Jurisdiction,
    pub date_range: DateRange,
}

#[derive(Debug, Deserialize)]
pub struct Poll {
    #[serde(rename = "Polling Firm")]
    pub polling_firm: String,
    #[serde(rename = "Commissioners")]
    pub commissioners: PollOption<String>,
    #[serde(rename = "Fieldwork Start")]
    pub fieldwork_start: NaiveDate,
    #[serde(rename = "Fieldwork End")]
    pub fieldwork_end: NaiveDate,
    #[serde(rename = "Scope")]
    pub scope: Scope,
    #[serde(rename = "Sample Size")]
    pub sample_size: PollOption<f32>,
    #[serde(rename = "Sample Size Qualification")]
    pub sample_size_qualification: PollOption<SampleSizeQualification>,
    #[serde(rename = "Participation")]
    pub participation: PollOption<Percentage>,
    #[serde(rename = "Precision")]
    pub precision: PollOption<PercentageOrSeats>,
    #[serde(flatten)]
    pub party_names: HashMap<String, PollOption<PercentageOrSeats>>,
    #[serde(rename = "Other")]
    pub other: PollOption<PercentageOrSeats>,
}

impl PollTable {
    pub fn get_jurisdiction_from_path(path: &str) -> Result<Jurisdiction, GetJurisdictionError> {
        let path = Path::new(path);
        path.extension().ok_or(GetJurisdictionError::NotCsvError)?;
        let filename = path
            .file_stem()
            .and_then(|os_str| os_str.to_str())
            .ok_or(GetJurisdictionError::InvalidPathError)?
            .chars()
            .map(|c| c.to_uppercase().to_string())
            .collect::<String>();

        let jurisdiction = Jurisdiction::from_str(&filename)?;
        Ok(jurisdiction)
    }

    pub fn get_date_range_from_fieldwork_start(self) -> DateRange {
        let last_date = self
            .polls
            .first()
            .expect("Fieldwork Start should not be empty")
            .fieldwork_start;
        let first_date = self
            .polls
            .last()
            .expect("Fieldwork Start should not be empty")
            .fieldwork_start;
        let diff = last_date - first_date;
        diff.num_days() as u32
    }

    pub fn get_date_range_from_fieldwork_end(self) -> DateRange {
        let last_date = self
            .polls
            .first()
            .expect("Fieldwork End should not be empty")
            .fieldwork_end;
        let first_date = self
            .polls
            .last()
            .expect("Fieldwork End should not be empty")
            .fieldwork_end;
        let diff = last_date - first_date;
        diff.num_days() as u32
    }

    pub fn get_date_range(polls: &[Poll]) -> DateRange {
        let last_date = polls
            .first()
            .expect("Fieldwork End should not be empty")
            .fieldwork_end;
        let first_date = polls
            .last()
            .expect("Fieldwork Start should not be empty")
            .fieldwork_start;
        let diff = last_date - first_date;
        diff.num_days() as u32
    }

    pub fn try_from_path(path: &str) -> Result<PollTable, PollTableTryFromPathError> {
        let mut rdr = ReaderBuilder::new().from_path(path)?;
        let mut polls: Vec<Poll> = Vec::new();
        let jurisdiction = Self::get_jurisdiction_from_path(path)?;
        for result in rdr.deserialize() {
            let record: Poll = result?;
            polls.push(record);
        }
        let date_range = Self::get_date_range(&polls);
        Ok(PollTable {
            polls,
            jurisdiction,
            date_range,
        })
    }
}

#[derive(Debug)]
pub struct Percentage(f32);
#[derive(Debug)]
pub struct Seats(f32);

#[derive(Debug)]
pub enum PollOption<T> {
    NotAvailable,
    Some(T),
}

#[derive(Debug)]
pub enum Scope {
    National,
    European,
}

#[derive(Debug)]
pub enum SampleSizeQualification {
    Provided,
    EstimatedAssumed,
}

#[derive(Debug)]
pub enum PercentageOrSeats {
    Percentage(Percentage),
    Seats(Seats),
}

impl<'de> Deserialize<'de> for PollOption<String> {
    fn deserialize<D>(deserializer: D) -> Result<PollOption<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val: String = Deserialize::deserialize(deserializer)?;
        match val.as_str() {
            "Not Available" | "N/A" => Ok(PollOption::NotAvailable),
            _ => Ok(PollOption::Some(val.to_string())),
        }
    }
}

impl<'de> Deserialize<'de> for Scope {
    fn deserialize<D>(deserializer: D) -> Result<Scope, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val: String = Deserialize::deserialize(deserializer)?;
        match val.as_str() {
            "National" => Ok(Scope::National),
            "European" => Ok(Scope::European),
            _ => Err(serde::de::Error::custom("Failed to parse Scope")),
        }
    }
}

impl<'de> Deserialize<'de> for PollOption<f32> {
    fn deserialize<D>(deserializer: D) -> Result<PollOption<f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val: String = Deserialize::deserialize(deserializer)?;
        if let Ok(val) = val.parse::<f32>() {
            Ok(PollOption::Some(val))
        } else {
            match val.as_str() {
                "Not Available" | "N/A" => Ok(PollOption::NotAvailable),
                _ => Err(serde::de::Error::custom("Failed to parse PollOption<f32>")),
            }
        }
    }
}

impl<'de> Deserialize<'de> for PollOption<SampleSizeQualification> {
    fn deserialize<D>(deserializer: D) -> Result<PollOption<SampleSizeQualification>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val: String = Deserialize::deserialize(deserializer)?;
        match val.as_str() {
            "Provided" => Ok(PollOption::Some(SampleSizeQualification::Provided)),
            "Estimated/Assumed" => Ok(PollOption::Some(SampleSizeQualification::EstimatedAssumed)),
            "Not Available" | "N/A" => Ok(PollOption::NotAvailable),
            _ => Err(serde::de::Error::custom(
                "Failed to parse SampleSizeQualification",
            )),
        }
    }
}

impl<'de> Deserialize<'de> for PollOption<Percentage> {
    fn deserialize<D>(deserializer: D) -> Result<PollOption<Percentage>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val: String = Deserialize::deserialize(deserializer)?;

        if val.contains('%') {
            let val = val
                .trim_end_matches('%')
                .parse::<f32>()
                .expect("Should be able to parse percentage as f32");
            Ok(PollOption::Some(Percentage(val)))
        } else if val.contains("Not Available") || val.contains("N/A") {
            Ok(PollOption::NotAvailable)
        } else {
            Err(serde::de::Error::custom(
                "Failed to parse PollOption<Percentage>",
            ))
        }
    }
}

impl<'de> Deserialize<'de> for PollOption<PercentageOrSeats> {
    fn deserialize<D>(deserializer: D) -> Result<PollOption<PercentageOrSeats>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val: String = Deserialize::deserialize(deserializer)?;

        match val.as_str() {
            "Not Available" | "N/A" => Ok(PollOption::NotAvailable),
            _ => {
                if val.contains('%') {
                    let val = val
                        .trim_end_matches('%')
                        .parse::<f32>()
                        .expect("Should be able to parse percentage as f32");
                    Ok(PollOption::Some(PercentageOrSeats::Percentage(Percentage(
                        val,
                    ))))
                } else {
                    match val.parse::<f32>() {
                        Ok(val) => Ok(PollOption::Some(PercentageOrSeats::Seats(Seats(val)))),
                        Err(_) => Err(serde::de::Error::custom("Seats could not be parsed as f32")),
                    }
                }
            }
        }
    }
}
