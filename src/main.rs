use clap::Parser;
use serde::Deserialize;
use std::fmt;
use thiserror::Error;

/// Trivial CLI to hit the OpenWeather API for the current weather at a position
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Latitude of the query position
    #[arg(long, default_value_t = 33.545)]
    lat: f32,

    /// Longitude of the query position
    #[arg(long, default_value_t = -117.771)]
    lon: f32,

    /// OpenWeather API key
    #[arg(short, long)]
    api_key: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let query_position = GeodeticCoords::new_checked(args.lat, args.lon)?;

    let response = open_weather_request(query_position, WeatherUnits::Imperial, args.api_key)?;

    println!("{response:#?}");

    Ok(())
}

/// Available units for OpenWeather responses
pub enum WeatherUnits {
    /// Standard is the default if the optional "units" parameter is not included in the request
    Standard,
    Imperial,
    Metric,
}

impl fmt::Display for WeatherUnits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WeatherUnits::Standard => write!(f, "standard"),
            WeatherUnits::Imperial => write!(f, "imperial"),
            WeatherUnits::Metric => write!(f, "metric"),
        }
    }
}

/// Geodetic coordinates, latitude and longitude
#[derive(Deserialize, Debug)]
pub struct GeodeticCoords {
    /// Latitude of the location
    pub lat: f32,

    /// Longitude of the location
    pub lon: f32,
}

impl GeodeticCoords {
    /// Creates a new [`GeodeticCoords`] and fails if the provided points are invalid.
    pub fn new_checked(lat: f32, lon: f32) -> Result<GeodeticCoords, GeodeticCoordsError> {
        if lat < -90.0 || lat > 90.0 {
            return Err(GeodeticCoordsError::LatitudeOutOfRange(lat));
        }

        if lon < -180.0 || lon > 180.0 {
            return Err(GeodeticCoordsError::LongitudeOutOfRange(lon));
        }

        Ok(GeodeticCoords { lat, lon })
    }
}

#[derive(Error, Debug)]
pub enum GeodeticCoordsError {
    #[error("provided latitude of `{0}` is out of the valid range [-90, 90]")]
    LatitudeOutOfRange(f32),

    #[error("provided longitude of `{0}` is out of the valid range [-180, 180]")]
    LongitudeOutOfRange(f32),
}

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

/// OpenWeather response from the current weather API, more details [here](https://openweathermap.org/current).
#[derive(Deserialize, Debug)]
pub struct OWCurrentWeatherResponse {
    pub coord: GeodeticCoords,

    // For some reason the response includes a list of these OWWeather objects
    pub weather: Vec<OWWeather>,

    pub main: OWMain,

    /// Visibility, meter. The maximum value of the visibility is 10 km
    pub visibility: f32,

    pub wind: OWWind,

    pub clouds: OWClouds,

    pub rain: Option<OWRain>,

    pub snow: Option<OWSnow>,

    /// Time of data calculation, UNIX time in seconds, UTC
    pub dt: u64,

    pub sys: OWSys,

    /// Shift in seconds from UTC
    pub timezone: i64,

    /// City ID
    pub id: u32,

    /// City name
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct OWWeather {
    /// Weather condition id
    pub id: u32,

    /// Group of weather parameters (Rain, Snow, Clouds etc.)
    pub main: String,

    /// Weather condition within the group. You can get the output in your language, more info [here](https://openweathermap.org/current#list).
    pub description: String,

    /// Weather icon id
    pub icon: String,
}

#[derive(Deserialize, Debug)]
pub struct OWMain {
    /// Temperature. Unit Default: Kelvin, Metric: Celsius, Imperial: Fahrenheit
    pub temp: f32,

    /// Temperature. This temperature parameter accounts for the human perception of weather. Unit Default: Kelvin, Metric: Celsius, Imperial: Fahrenheit
    pub feels_like: f32,

    /// Atmospheric pressure on the sea level, hPa
    pub pressure: f32,

    /// Humidity, %
    pub humidity: f32,

    /// Minimum temperature at the moment. This is minimal currently observed temperature (within large megalopolises and urban areas). Please find more info here. Unit Default: Kelvin, Metric: Celsius, Imperial: Fahrenheit
    pub temp_min: f32,

    /// Maximum temperature at the moment. This is maximal currently observed temperature (within large megalopolises and urban areas). Please find more info here. Unit Default: Kelvin, Metric: Celsius, Imperial: Fahrenheit
    pub temp_max: f32,

    /// Atmospheric pressure on the sea level, hPa
    pub sea_level: f32,

    /// Atmospheric pressure on the ground level, hPa
    pub grnd_level: f32,
}

#[derive(Deserialize, Debug)]
pub struct OWWind {
    /// Wind speed. Unit Default: meter/sec, Metric: meter/sec, Imperial: miles/hour
    pub speed: f32,

    /// Wind direction, degrees (meteorological)
    pub deg: f32,

    /// Wind gust. Unit Default: meter/sec, Metric: meter/sec, Imperial: miles/hour
    /// The docs are not specific about this being optional but I've seen responses without it.
    pub gust: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct OWClouds {
    /// Cloudiness %
    pub all: f32,
}

#[derive(Deserialize, Debug)]
pub struct OWRain {
    /// Precipitation, mm/h. Please note that only mm/h as units of measurement are available for this parameter
    pub r#_1h: f32,
}

#[derive(Deserialize, Debug)]
pub struct OWSnow {
    /// Precipitation, mm/h. Please note that only mm/h as units of measurement are available for this parameter
    pub r#_1h: f32,
}

#[derive(Deserialize, Debug)]
pub struct OWSys {
    /// Country code (GB, JP etc.)
    pub country: String,

    /// Sunrise time, seconds since UNIX epoch, UTC
    pub sunrise: u64,

    /// Sunset time, seconds since UNIX epoch, UTC
    pub sunset: u64,
}
