//! Core API for the main chain queries

use plutus::Datum;
#[allow(unused_imports)]
use std::sync::Arc;
use thiserror::Error;

/// Types that will be used by the Cardano follower
pub mod common;

#[derive(Debug, PartialEq, Error)]
pub enum DataSourceError {
	#[error("Bad request: `{0}`.")]
	BadRequest(String),
	#[error("Internal error of data source: `{0}`.")]
	InternalDataSourceError(String),
	#[error("Could not decode {datum:?} to {to:?}, this means that there is an error in Plutus scripts or chain-follower is obsolete.")]
	DatumDecodeError { datum: Datum, to: String },
	#[error("'{0}' not found. Possible causes: main chain follower configuration error, db-sync not synced fully, or data not set on the main chain.")]
	ExpectedDataNotFound(String),
	#[error("Invalid data. {0} Possible cause it an error in Plutus scripts or chain-follower is obsolete.")]
	InvalidData(String),
}

pub type Result<T> = std::result::Result<T, DataSourceError>;
