//! Interactions with the OpenWeather HTTP API

use crate::{GeodeticCoords, OWCurrentWeatherResponse, WeatherUnits};

/// Request the current weather from OpenWeather, this is a blocking HTTP request.
pub fn open_weather_request(
    coords: GeodeticCoords,
    units: WeatherUnits,
    api_key: String,
) -> anyhow::Result<OWCurrentWeatherResponse> {
    // TODO (tff): pick a return type for this thing
    let lat_str = coords.lat.to_string();
    let lon_str = coords.lon.to_string();

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={lat_str}&lon={lon_str}&mode=json&units={units}&appid={api_key}"
    );

    // This makes a new Client on each GET, but we're making requests so infrequently this is totally fine.
    let response = reqwest::blocking::get(url)?.text()?;

    // First get the untyped JSON blob so we log it in the event of a parsing failure
    let response_json: serde_json::Value = serde_json::from_str(&response)?;

    // TODO: log here
    // println!("{response_json:#?}");

    let parsed: OWCurrentWeatherResponse = serde_json::from_value(response_json)?;
    Ok(parsed)
}
