use clap::Parser;
use openwx::{GeodeticCoords, WeatherUnits};

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

    let response =
        openwx::open_weather_request(query_position, WeatherUnits::Imperial, args.api_key)?;

    println!("{response:#?}");

    let local_sunrise = response.sunrise_local().time();
    let local_sunset = response.sunset_local().time();

    println!("Sunrise: {local_sunrise}");
    println!("Sunset: {local_sunset}");
    println!(
        "Wind coming from: {}, blowing towards: {}",
        response.wind.deg.compass_point(),
        response.wind.deg.blowing_towards()
    );

    Ok(())
}
