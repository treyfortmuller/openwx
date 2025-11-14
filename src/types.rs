use chrono::{DateTime, FixedOffset, Utc};
use serde::Deserialize;
use strum::Display;
use thiserror::Error;

/// Available units for OpenWeather responses
#[derive(Debug, Display)]
#[strum(serialize_all = "lowercase")]
pub enum WeatherUnits {
    /// Standard is the default if the optional "units" parameter is not included in the request
    Standard,
    Imperial,
    Metric,
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

#[derive(Error, Debug, PartialEq)]
pub enum GeodeticCoordsError {
    #[error("provided latitude of `{0}` is out of the valid range [-90, 90]")]
    LatitudeOutOfRange(f32),

    #[error("provided longitude of `{0}` is out of the valid range [-180, 180]")]
    LongitudeOutOfRange(f32),
}

/// OpenWeather response from the current weather API, more details [here](https://openweathermap.org/current).
#[derive(Deserialize, Debug)]
pub struct OWCurrentWeatherResponse {
    pub coord: GeodeticCoords,

    /// NOTE: It is possible to meet more than one weather condition for a requested location.
    /// The first weather condition in the response is primary.
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
    #[serde(deserialize_with = "from_utc_shift")]
    pub timezone: FixedOffset,

    /// City ID
    pub id: u32,

    /// City name
    pub name: String,
}

/// OpenWeather returns the timezone of our query position as a number of seconds shifted from UTC, we want to
/// deserialize that as a DateTime::FixedOffset for the local time.
fn from_utc_shift<'de, D>(deserializer: D) -> Result<FixedOffset, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let tz_shift = i32::deserialize(deserializer)?;

    // I observed a negative value for the timezone shift while sitting in UTC-8, which is the formalism for east_opt
    let fixed_offset = FixedOffset::east_opt(tz_shift)
        .ok_or_else(|| serde::de::Error::custom("invalid timezone shift from UTC"))?;

    Ok(fixed_offset)
}

impl OWCurrentWeatherResponse {
    /// Return the sunrise datetime in the local timezone
    pub fn sunrise_local(&self) -> DateTime<FixedOffset> {
        self.sys.sunrise.with_timezone(&self.timezone)
    }

    /// Return the sunset datetime in the local timezone
    pub fn sunset_local(&self) -> DateTime<FixedOffset> {
        self.sys.sunset.with_timezone(&self.timezone)
    }
}

#[derive(Deserialize, Debug)]
pub struct OWWeather {
    /// Weather condition id, more info on condition IDs and icons [here](https://openweathermap.org/weather-conditions).
    pub id: u32,

    /// Group of weather parameters (Rain, Snow, Clouds etc.)
    pub main: String,

    /// Weather condition within the group. You can get the output in your language, more info [here](https://openweathermap.org/current#list).
    pub description: String,

    /// Weather icon id
    pub icon: String,
}

/// Points on a 16-wind compass rose
#[derive(Debug, Display)]
pub enum CompassPoint {
    North,
    NorthNorthEast,
    NorthEast,
    EastNorthEast,
    East,
    EastSouthEast,
    SouthEast,
    SouthSouthEast,
    South,
    SouthSouthWest,
    SouthWest,
    WestSouthWest,
    West,
    WestNorthWest,
    NorthWest,
    NorthNorthWest,
}

/// Meteorological convention for wind direction is measured in degrees clockwise from true North, and represents
/// the direction _from which_ the wind is coming, thats what the OpenWeather API will respond with.
#[derive(Debug)]
pub struct WindDirection(f32);

#[derive(Error, Debug)]
pub enum WindDirectionError {
    #[error("provided wind direction of `{0}` is outside the valid range [0, 360)")]
    InvalidDirection(f32),
}

impl WindDirection {
    /// Validates the wind direction falls inside the range for compass direction in the meteorological convention.
    pub fn new_checked(deg: f32) -> Result<Self, WindDirectionError> {
        if deg < 0.0 || deg >= 360.0 {
            return Err(WindDirectionError::InvalidDirection(deg));
        }

        Ok(WindDirection(deg))
    }

    /// Returns the compass point from which the wind is blowing
    pub fn compass_point(&self) -> CompassPoint {
        match self.0 {
            0.0..22.5 => CompassPoint::North,
            22.5..45.0 => CompassPoint::NorthNorthEast,
            45.0..67.5 => CompassPoint::NorthEast,
            67.5..90.0 => CompassPoint::EastNorthEast,
            90.0..112.5 => CompassPoint::East,
            112.5..135.0 => CompassPoint::EastSouthEast,
            135.0..157.5 => CompassPoint::SouthEast,
            157.5..180.0 => CompassPoint::SouthSouthEast,
            180.0..202.5 => CompassPoint::South,
            202.5..225.0 => CompassPoint::SouthSouthWest,
            225.0..247.5 => CompassPoint::SouthWest,
            247.5..270.0 => CompassPoint::WestSouthWest,
            270.0..292.5 => CompassPoint::West,
            292.5..315.0 => CompassPoint::WestNorthWest,
            315.0..337.5 => CompassPoint::NorthWest,
            _ => CompassPoint::NorthNorthWest,
        }
    }

    /// Returns the compass point that the wind is blowing to, opposite to the meteorological convention.
    pub fn blowing_towards(&self) -> CompassPoint {
        let tmp = self.0 + 180.0;
        let towards = if tmp >= 360.0 { tmp - 360.0 } else { tmp };

        WindDirection(towards).compass_point()
    }
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
    #[serde(deserialize_with = "from_raw_wind_direction")]
    pub deg: WindDirection,

    /// Wind gust. Unit Default: meter/sec, Metric: meter/sec, Imperial: miles/hour
    /// The docs are not specific about this being optional but I've seen responses without it.
    pub gust: Option<f32>,
}

/// OpenWeather returns sunrise and sunset times as seconds since UNIX epoch expressed in UTC, we convert
/// them to timezone-aware [`chrono::DateTime`]s as part of the deserialization process.
fn from_raw_wind_direction<'de, D>(deserializer: D) -> Result<WindDirection, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = f32::deserialize(deserializer)?;
    let wind_dir = WindDirection::new_checked(raw).map_err(|e| serde::de::Error::custom(e))?;

    Ok(wind_dir)
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
    #[serde(deserialize_with = "from_unix_offset")]
    pub sunrise: DateTime<Utc>,

    /// Sunset time, seconds since UNIX epoch, UTC
    #[serde(deserialize_with = "from_unix_offset")]
    pub sunset: DateTime<Utc>,
}

/// OpenWeather returns sunrise and sunset times as seconds since UNIX epoch expressed in UTC, we convert
/// them to timezone-aware [`chrono::DateTime`]s as part of the deserialization process.
fn from_unix_offset<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let secs_since_unix = u64::deserialize(deserializer)?;

    let date_time = DateTime::from_timestamp(secs_since_unix as i64, 0)
        .ok_or_else(|| serde::de::Error::custom("invalid timestamp"))?;

    Ok(date_time)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geodetic_coords_checked() {
        let bad_lat = GeodeticCoords::new_checked(180.0, 0.0);
        assert_eq!(
            bad_lat.unwrap_err(),
            GeodeticCoordsError::LatitudeOutOfRange(180.0)
        );

        let bad_lon = GeodeticCoords::new_checked(33.0, 190.0);
        assert_eq!(
            bad_lon.unwrap_err(),
            GeodeticCoordsError::LongitudeOutOfRange(190.0)
        );

        let valid_geo = GeodeticCoords::new_checked(33.0, -117.0);
        assert!(valid_geo.is_ok())
    }

    #[test]
    fn parse_open_weather_response() {
        let stringly = r#"
        {
        "coord": {
            "lon": 10.99,
            "lat": 44.34
        },
        "weather": [
            {
            "id": 803,
            "main": "Clouds",
            "description": "broken clouds",
            "icon": "04n"
            }
        ],
        "base": "stations",
        "main": {
            "temp": 281.29,
            "feels_like": 279.63,
            "temp_min": 279.38,
            "temp_max": 281.29,
            "pressure": 1024,
            "humidity": 95,
            "sea_level": 1024,
            "grnd_level": 956
        },
        "visibility": 10000,
        "wind": {
            "speed": 2.69,
            "deg": 202,
            "gust": 3.51
        },
        "clouds": {
            "all": 78
        },
        "dt": 1763077522,
        "sys": {
            "type": 2,
            "id": 2004688,
            "country": "IT",
            "sunrise": 1763100641,
            "sunset": 1763135429
        },
        "timezone": 3600,
        "id": 3163858,
        "name": "Zocca",
        "cod": 200
        }
        "#;

        let res: Result<OWCurrentWeatherResponse, _> = serde_json::from_str(stringly);
        assert!(res.is_ok())
    }
}
