//! Interactions with the OpenWeather HTTP API

use crate::{GeodeticCoords, OWCurrentWeatherResponse, WeatherUnits};
use log::error;
use thiserror::Error;

/// Errors that occur at the API boundary with OpenWeather
#[derive(Error, Debug)]
pub enum OpenWxError {
    #[error("Failed to parse response")]
    ResponseParseError {
        /// The valid JSON that we failed to parse into strongly-typed data
        input_json: serde_json::Value,

        #[source]
        parse_error: serde_json::Error,
    },

    #[error("the response from open weather is not valid JSON")]
    MalformedResponseError(#[from] serde_json::Error),

    #[error("HTTP GET from OpenWeather failed")]
    HttpGetError(#[from] reqwest::Error),
}

/// Request the current weather from OpenWeather, this is a blocking HTTP request.
pub fn open_weather_request(
    coords: GeodeticCoords,
    units: WeatherUnits,
    api_key: String,
) -> Result<OWCurrentWeatherResponse, OpenWxError> {
    let lat_str = coords.lat.to_string();
    let lon_str = coords.lon.to_string();

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={lat_str}&lon={lon_str}&mode=json&units={units}&appid={api_key}"
    );

    // This makes a new Client on each GET, but we're making requests so infrequently this is totally fine.
    let response_text = reqwest::blocking::get(url)?
        .error_for_status()
        .map_err(OpenWxError::HttpGetError)?
        .text()?;

    // First get the untyped JSON blob so we log it in the event of a parsing failure
    let response_json: serde_json::Value = serde_json::from_str(&response_text)?;

    let parsed: OWCurrentWeatherResponse =
        serde_json::from_value(response_json.clone()).map_err(|err| {
            OpenWxError::ResponseParseError {
                input_json: response_json,
                parse_error: err,
            }
        })?;

    Ok(parsed)
}
