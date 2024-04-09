#![warn(missing_docs)]
//! A parsing library for the Europe Elects .csv format, which stores opinion poll data for countries covered by Europe Elects's publications.
//!
//! The [PollTable] struct represents an individual country's opinion poll data.
//! It must be constructed from valid .csv data conforming to Europe Elects' .csv format, which can be found [here](https://europeelects.eu/data/).
//!
//! ```
//! let british_data = PollTable::try_from_path("gb.csv");
//!
//! assert_eq!(british_data.polling_firm(0), "YouGov");
//! assert_eq!(british_data.fieldwork_start(0), NaiveDate::from_ymd_opt(2024, 3, 6).unwrap());
//! assert_eq!(british_data.scope(), Scope::National);
//! assert_eq!(british_data.jurisdiction(), "United Kingdom of Great Britain and Northern Ireland");
//! assert_eq!(british_data.date_range(), 2252);
//! ```
mod errors;
use chrono::NaiveDate;
use csv::ReaderBuilder;
use errors::{PollTableFromStrError, PollTableTryFromPathError, RawPollTableFromStrError};
use serde::{Deserialize, Deserializer};
use std::{collections::HashMap, path::Path, str::FromStr};

#[derive(Copy, Clone, Debug)]
/// The countries, regions and territories for which Europe Elects collects opinion poll data.
pub enum Jurisdiction {
    Albania,
    Andorra,
    Armenia,
    Austria,
    BelgiumBrussels,
    BelgiumFlanders,
    BelgiumWallonia,
    Bulgaria,
    Croatia,
    Cyprus,
    Czechia,
    Denmark,
    Estonia,
    Finland,
    France,
    Georgia,
    Germany,
    Gibraltar,
    Greece,
    Hungary,
    Iceland,
    Ireland,
    Italy,
    Kosovo,
    Latvia,
    Lithuania,
    Luxembourg,
    Malta,
    Moldova,
    Montenegro,
    Netherlands,
    NorthMacedonia,
    Norway,
    Poland,
    Portugal,
    Romania,
    Russia,
    Serbia,
    Slovakia,
    Slovenia,
    Spain,
    Sweden,
    Switzerland,
    Turkiye,
    UKGreatBritain,
    UKNorthernIreland,
    UKNorthernIrelandEuropean,
    UKNorthernIrelandNational,
    Ukraine,
}

fn init_jurisdiction() -> HashMap<String, Jurisdiction> {
    HashMap::from([
        (String::from("al"), Jurisdiction::Albania),
        (String::from("ad"), Jurisdiction::Andorra),
        (String::from("am"), Jurisdiction::Armenia),
        (String::from("at"), Jurisdiction::Austria),
        (String::from("be-bru"), Jurisdiction::BelgiumBrussels),
        (String::from("be-vlg"), Jurisdiction::BelgiumFlanders),
        (String::from("be-wal"), Jurisdiction::BelgiumWallonia),
        (String::from("bg"), Jurisdiction::Bulgaria),
        (String::from("hr"), Jurisdiction::Croatia),
        (String::from("cy"), Jurisdiction::Cyprus),
        (String::from("cz"), Jurisdiction::Czechia),
        (String::from("dk"), Jurisdiction::Denmark),
        (String::from("ee"), Jurisdiction::Estonia),
        (String::from("fi"), Jurisdiction::Finland),
        (String::from("fr"), Jurisdiction::France),
        (String::from("ge"), Jurisdiction::Georgia),
        (String::from("de"), Jurisdiction::Germany),
        (String::from("gi"), Jurisdiction::Gibraltar),
        (String::from("gr"), Jurisdiction::Greece),
        (String::from("hu"), Jurisdiction::Hungary),
        (String::from("is"), Jurisdiction::Iceland),
        (String::from("ie"), Jurisdiction::Ireland),
        (String::from("it"), Jurisdiction::Italy),
        (String::from("xk"), Jurisdiction::Kosovo),
        (String::from("lv"), Jurisdiction::Latvia),
        (String::from("lt"), Jurisdiction::Lithuania),
        (String::from("lu"), Jurisdiction::Luxembourg),
        (String::from("mt"), Jurisdiction::Malta),
        (String::from("md"), Jurisdiction::Moldova),
        (String::from("me"), Jurisdiction::Montenegro),
        (String::from("nl"), Jurisdiction::Netherlands),
        (String::from("mk"), Jurisdiction::NorthMacedonia),
        (String::from("no"), Jurisdiction::Norway),
        (String::from("pl"), Jurisdiction::Poland),
        (String::from("pt"), Jurisdiction::Portugal),
        (String::from("ro"), Jurisdiction::Romania),
        (String::from("ru"), Jurisdiction::Russia),
        (String::from("rs"), Jurisdiction::Serbia),
        (String::from("sk"), Jurisdiction::Slovakia),
        (String::from("si"), Jurisdiction::Slovenia),
        (String::from("es"), Jurisdiction::Spain),
        (String::from("se"), Jurisdiction::Sweden),
        (String::from("ch"), Jurisdiction::Switzerland),
        (String::from("tr"), Jurisdiction::Turkiye),
        (String::from("gb"), Jurisdiction::UKGreatBritain),
        (String::from("gb-nir"), Jurisdiction::UKNorthernIreland),
        (
            String::from("gb-nir-E"),
            Jurisdiction::UKNorthernIrelandEuropean,
        ),
        (
            String::from("gb-nir-N"),
            Jurisdiction::UKNorthernIrelandNational,
        ),
        (String::from("ua"), Jurisdiction::Ukraine),
    ])
}
#[derive(Debug)]
/// Represents one EuropeElects .csv file.
/// It contains metadata about the particular poll file, and the individual opinion polls themselves.
pub struct PollTable {
    polls: Vec<Poll>,
    jurisdiction: Jurisdiction,
}

#[derive(Debug)]
/// Unlike [PollTable], contains no jurisdiction validation and as such can contain arbitary polling data that conforms to the EuropeElects .csv standard.
pub struct RawPollTable {
    polls: Vec<Poll>,
}

/// Each Poll is one line of .csv, and represents all metadata and party results for one opinion poll.
#[derive(Debug, Deserialize)]
pub struct Poll {
    #[serde(rename = "Polling Firm")]
    polling_firm: String,
    #[serde(rename = "Commissioners")]
    commissioners: PollOption<String>,
    #[serde(rename = "Fieldwork Start")]
    fieldwork_start: NaiveDate,
    #[serde(rename = "Fieldwork End")]
    fieldwork_end: NaiveDate,
    #[serde(rename = "Scope")]
    scope: Scope,
    #[serde(rename = "Sample Size")]
    sample_size: PollOption<f32>,
    #[serde(rename = "Sample Size Qualification")]
    sample_size_qualification: PollOption<SampleSizeQualification>,
    #[serde(rename = "Participation")]
    participation: PollOption<Percentage>,
    #[serde(rename = "Precision")]
    precision: PollOption<PercentageOrSeats>,
    #[serde(flatten)]
    party_results: HashMap<String, PollOption<PercentageOrSeats>>,
    #[serde(rename = "Other")]
    other: PollOption<PercentageOrSeats>,
}

impl PollTable {
    pub fn new(polls: Vec<Poll>, jurisdiction: Jurisdiction) -> Self {
        PollTable {
            polls,
            jurisdiction,
        }
    }
    /// Attempts to create a [PollTable] from a .csv file.
    /// The file must be in the Europe Elects format, which is specified at [https://europeelects.eu/data/](https://europeelects.eu/data/).
    /// ```
    /// use europe_elects_csv::*;
    ///
    /// // For French polling data:
    /// let poll_table = PollTable::try_from_path("fr.csv")
    ///
    /// // For Hungarian polling data:
    /// let poll_table = PollTable::try_from_path("hu.csv")
    /// ```
    /// The name of the file must also conform to the ISO 3166 country codes, and of that be a code associated with a territory for which Europe Elects collect opinion poll data.
    /// ```
    /// use europe_elects_csv::*;
    ///
    /// // This would not error, as "gb" is a valid country code.
    /// let poll_table = PollTable::try_from_path("gb.csv");
    ///
    /// // This would not error either , as "gb-nir" is a valid code for the Europe Elects .csv database.
    /// let poll_table = PollTable::try_from_path("gb-nir.csv");
    ///
    /// // This WOULD error, because although "us" is a valid country code,
    /// // it is NOT one covered by Europe Elects.
    /// let poll_table = PollTable::try_from_path("us.csv");
    ///
    /// // This would error too, because "xe" is not a valid country code.
    /// let poll_table = PollTable::try_from_path("xe.csv");
    /// ```
    pub fn try_from_path(path: &str) -> Result<PollTable, PollTableTryFromPathError> {
        let mut rdr = ReaderBuilder::new().from_path(path)?;
        let mut polls: Vec<Poll> = Vec::new();

        let path = Path::new(path);

        let extension = path
            .extension()
            .and_then(|os_str| os_str.to_str())
            .ok_or(PollTableTryFromPathError::InvalidPathError)?;

        if extension != "csv" {
            return Err(PollTableTryFromPathError::NotCsvError);
        }

        let filename = path
            .file_stem()
            .and_then(|os_str| os_str.to_str())
            .ok_or(PollTableTryFromPathError::InvalidPathError)?;

        // Jurisdiction
        let jurisdiction_map = init_jurisdiction();
        let jurisdiction = *jurisdiction_map
            .get(filename)
            .ok_or(PollTableTryFromPathError::InvalidJurisdictionError)?;

        // Polls
        for result in rdr.deserialize() {
            let record: Poll = result?;
            polls.push(record);
        }

        Ok(PollTable {
            polls,
            jurisdiction,
        })
    }

    /// Creates a [PollTable] based on an input &str, which must be formatted exactly as the Europe Elects .csv format.
    /// This does *not* implement [FromStr], because the poll jurisdiction is not contained within .csv data;
    /// in [try_from_path()], it is gathered from the filename, but for [from_str()], it must be specified in the input parameters.
    /// The jurisdiction input parameter must conform to one of the ISO 3166 country codes specified at [https://europeelects.eu/data/](https://europeelects.eu/data/).
    /// ```
    /// use europe_elects_csv::*;
    /// let example = r#"
    ///     Polling Firm,Commissioners,Fieldwork Start,Fieldwork End,Scope,Sample Size,Sample Size Qualification,Participation,Precision,First Party,Second Party,Third Party,Fourth Party,Other
    ///     Epic Polling,The Daily Snail,2024-03-06,2024-03-08,National,2054,Provided,Not Available,1%,30%,40%,25%,5%"#;
    /// let example_poll = PollTable::from_str(example, "de").unwrap();
    /// ```
    pub fn from_str(s: &str, jurisdiction: &str) -> Result<PollTable, PollTableFromStrError> {
        let mut rdr = ReaderBuilder::new().from_reader(s.as_bytes());
        let mut polls: Vec<Poll> = Vec::new();

        // Jurisdiction
        let jurisdiction_map = init_jurisdiction();
        let final_jurisdiction = *jurisdiction_map
            .get(jurisdiction)
            .ok_or(PollTableFromStrError::InvalidJurisdictionError)?;

        // Polls
        for result in rdr.deserialize() {
            let record: Poll = result?;
            polls.push(record);
        }

        Ok(PollTable {
            polls,
            jurisdiction: final_jurisdiction,
        })
    }

    /// Returns all opinion polls as a Vec of [Poll]s, indexed from newest to oldest.
    pub fn polls(&self) -> &Vec<Poll> {
        &self.polls
    }

    /// Returns an Option of an individual opinion poll by its index in the [PollTable].
    pub fn poll_by_index(&self, index: usize) -> Option<&Poll> {
        self.polls.get(index)
    }

    /// Returns the polling firm of the given poll by index.
    /// ```
    /// use europe_elects_csv::*;
    /// let example = r#"
    ///     Polling Firm,Commissioners,Fieldwork Start,Fieldwork End,Scope,Sample Size,Sample Size Qualification,Participation,Precision,First Party,Second Party,Third Party,Fourth Party,Other
    ///     Epic Polling,The Daily Snail,2024-03-06,2024-03-08,National,2054,Provided,Not Available,1%,30%,40%,25%,5%"#;
    /// let example_poll = PollTable::from_str(example, "fr").unwrap();
    ///
    /// assert_eq!(example_poll.polling_firm(0), "Epic Polling")
    /// ```
    pub fn polling_firm(&self, index: usize) -> Option<&String> {
        Some(&self.polls.get(index)?.polling_firm)
    }
    /// Returns the commissioners of the given poll by index, or returns PollOption::NotAvailable if the "Commissioners" field is empty.
    /// ```
    /// use europe_elects_csv::*;
    /// let example = r#"
    ///     Polling Firm,Commissioners,Fieldwork Start,Fieldwork End,Scope,Sample Size,Sample Size Qualification,Participation,Precision,First Party,Second Party,Third Party,Fourth Party,Other
    ///     Epic Polling,The Daily Snail,2024-03-06,2024-03-08,National,2054,Provided,Not Available,1%,30%,40%,25%,5%"#;
    /// let example_poll = PollTable::from_str(example).unwrap();
    ///
    /// assert_eq!(example_poll.commissioners(), PollOption::Some("ExampleCommissioner"))
    /// assert_eq!(poll_with_empty_commissioners.commissioners(), PollOption::NotAvailable)
    /// ```
    pub fn commissioners(&self, index: usize) -> Option<&PollOption<String>> {
        Some(&self.polls.get(index)?.commissioners)
    }

    /// Returns the date of the beginning of the poll's fieldwork using [chrono]'s NaiveDate format.
    /// ```
    /// use europe_elects_csv::PollTable;
    /// let example = r#"
    ///     Polling Firm,Commissioners,Fieldwork Start,Fieldwork End,Scope,Sample Size,Sample Size Qualification,Participation,Precision,First Party,Second Party,Third Party,Fourth Party,Other
    ///     Epic Polling,The Daily Snail,2024-03-06,2024-03-08,National,2054,Provided,Not Available,1%,30%,40%,25%,5%"#;
    /// let example_poll = PollTable::from_str(example).unwrap();
    ///
    /// assert_eq!(example_poll.fieldwork_start(), chrono::NaiveDate::from_ymd_opt(2024, 3, 6).unwrap());
    /// ```
    pub fn fieldwork_start(&self, index: usize) -> &NaiveDate {
        &self.polls[index].fieldwork_start
    }

    /// As with fieldwork_end, but for the end of the poll's fieldwork.
    pub fn fieldwork_end(&self, index: usize) -> &NaiveDate {
        &self.polls[index].fieldwork_end
    }

    /// Returns the scope of the poll, which can either be National for polls for the country's national parliament, or European for polls to the European parliament.
    pub fn scope(&self, index: usize) -> &Scope {
        &self.polls[index].scope
    }

    pub fn sample_size(&self, index: usize) -> &PollOption<f32> {
        &self.polls[index].sample_size
    }

    pub fn sample_size_qualification(&self, index: usize) -> &PollOption<SampleSizeQualification> {
        &self.polls[index].sample_size_qualification
    }

    pub fn participation(&self, index: usize) -> &PollOption<Percentage> {
        &self.polls[index].participation
    }

    pub fn precision(&self, index: usize) -> &PollOption<PercentageOrSeats> {
        &self.polls[index].precision
    }

    pub fn party_results(&self, index: usize) -> &HashMap<String, PollOption<PercentageOrSeats>> {
        &self.polls[index].party_results
    }

    pub fn other(&self, index: usize) -> &PollOption<PercentageOrSeats> {
        &self.polls[index].other
    }

    pub fn jurisdiction(&self) -> &Jurisdiction {
        &self.jurisdiction
    }

    pub fn date_range(&self) -> usize {
        // Date range
        let last_date = &self
            .polls
            .first()
            .expect("Fieldwork End should not be empty")
            .fieldwork_end;
        let first_date = &self
            .polls
            .last()
            .expect("Fieldwork Start should not be empty")
            .fieldwork_start;
        let diff = *last_date - *first_date;
        diff.num_days() as usize
    }
}


impl RawPollTable {
    /// Creates a new RawPollTable from a Vec of [Poll]s.
    pub fn new(polls: Vec<Poll>) -> Self {
        RawPollTable { polls }
    }
}

impl FromStr for RawPollTable {
    type Err = RawPollTableFromStrError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut rdr = ReaderBuilder::new().from_reader(s.as_bytes());
        let mut polls: Vec<Poll> = Vec::new();

        // Polls
        for result in rdr.deserialize() {
            let record: Poll = result?;
            polls.push(record);
        }

        Ok(RawPollTable { polls })
    }
}

impl Poll {
    pub fn new(
        polling_firm: String,
        commissioners: PollOption<String>,
        fieldwork_start: NaiveDate,
        fieldwork_end: NaiveDate,
        scope: Scope,
        sample_size: PollOption<f32>,
        sample_size_qualification: PollOption<SampleSizeQualification>,
        participation: PollOption<Percentage>,
        precision: PollOption<PercentageOrSeats>,
        party_results: HashMap<String, PollOption<PercentageOrSeats>>,
        other: PollOption<PercentageOrSeats>
    ) -> Self {
        Poll {
            polling_firm,
            commissioners,
            fieldwork_start,
            fieldwork_end,
            scope,
            sample_size,
            sample_size_qualification,
            participation,
            precision,
            party_results,
            other,
        }
    }

    pub fn party_results(&self) -> &HashMap<String, PollOption<PercentageOrSeats>> {
        &self.party_results
    }
}

#[derive(Debug, Clone, Copy)]
/// Represents values that are percentages.
pub struct Percentage(f32);

impl Percentage {
    pub fn value(&self) -> f32 {
        self.0
    }
}
#[derive(Debug, Clone, Copy)]
/// Wrapper around an f32 that was parsed from "S%", representing a number of parliamentary seats.
pub struct Seats(f32);

impl Seats {
    pub fn value(&self) -> f32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
/// PollOption is used to represent the optionality of many poll fields. Fields using PollOption<T> may be potentially NotAvailable, mirroring Option<T>.
pub enum PollOption<T> {
    /// Represents that data is not available or not provided by the polling firm.
    NotAvailable,
    /// Wraps values in optional fields.
    Some(T),
}

impl<T> PollOption<T> {
    pub fn poll_unwrap(&self) -> &T {
        match self {
            PollOption::Some(val) => val,
            PollOption::NotAvailable => panic!("Uh oh!")
        }
    }
    pub fn is_ok(&self) -> bool {
        match self {
            PollOption::Some(_) => true,
            PollOption::NotAvailable => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Scope {
    National,
    European,
}

#[derive(Debug, Clone, Copy)]
pub enum SampleSizeQualification {
    Provided,
    EstimatedAssumed,
}

#[derive(Debug, Clone, Copy)]
pub enum PercentageOrSeats {
    Percentage(Percentage),
    Seats(Seats),
}

impl PercentageOrSeats {
    pub fn value(&self) -> f32 {
        match self {
            PercentageOrSeats::Percentage(val) => val.value(),
            PercentageOrSeats::Seats(val) => val.value(),
        }
    }
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
