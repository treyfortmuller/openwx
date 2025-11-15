# openwx

Smallest possible OpenWeather API wrapper library, plus a small test CLI. Synchronous HTTP client from `reqwest` and I only implemented the "current weather" endpoint for OpenWeather's free tier. API responses are fully typed and take care of timezone considerations. 

### OpenWeather API

First signup for the OpenWeather free tier and get an API token [here](https://home.openweathermap.org/users/sign_up). API docs for the "current weather data" API can be found [here](https://openweathermap.org/current).

Note the OpenWeather model isn't updated more frequently than once every 10 minutes so thats a lower bound on API call frequency. The free tier of OpenWeather allows for 1000 calls/day for free.

```txt
curl "https://api.openweathermap.org/data/2.5/weather?lat=44.34&lon=10.99&appid={API key}" | jq
```

Gives us

```json
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
```

The OpenWeather docs have a nice explainer for every field provided in this API response.

### The CLI

```
$ ./openwx --help

Trivial CLI to hit the OpenWeather API for the current weather at a position

Usage: openwx [OPTIONS] --api-key <API_KEY>

Options:
      --lat <LAT>          Latitude of the query position [default: 33.545]
      --lon <LON>          Longitude of the query position [default: -117.771]
  -a, --api-key <API_KEY>  OpenWeather API key
  -h, --help               Print help
  -V, --version            Print version
```
