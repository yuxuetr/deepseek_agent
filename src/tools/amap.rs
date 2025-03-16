use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct DistrictResponse {
  status: String,
  districts: Vec<District>,
}

#[derive(Debug, Serialize, Deserialize)]
struct District {
  adcode: String,
  name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherResponse {
  pub status: String,
  pub count: String,
  pub info: String,
  pub infocode: String,
  pub forecasts: Vec<Forecast>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Forecast {
  pub city: String,
  pub adcode: String,
  pub province: String,
  pub reporttime: String,
  pub casts: Vec<Cast>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cast {
  pub date: String,
  pub week: String,
  pub dayweather: String,
  pub nightweather: String,
  pub daytemp: String,
  pub nighttemp: String,
  pub daywind: String,
  pub nightwind: String,
  pub daypower: String,
  pub nightpower: String,
  #[serde(rename = "daytemp_float")]
  pub daytemp_float: String,
  #[serde(rename = "nighttemp_float")]
  pub nighttemp_float: String,
}

pub async fn get_weather(location: &str, api_key: &str) -> Result<WeatherResponse, reqwest::Error> {
  // 第一步：获取行政编码
  let district_url = format!(
    "https://restapi.amap.com/v3/config/district?key={}&keywords={}&subdistrict=0&extensions=all",
    api_key, location
  );

  let district_resp: DistrictResponse = reqwest::get(&district_url).await?.json().await?;

  if district_resp.status != "1" || district_resp.districts.is_empty() {
    eprintln!("行政编码查询失败");
    return Err(reqwest::get("http://error").await.unwrap_err());
  }

  let adcode = &district_resp.districts[0].adcode;

  // 第二步：获取天气数据
  let weather_url = format!(
    "https://restapi.amap.com/v3/weather/weatherInfo?key={}&city={}&extensions=all&output=json",
    api_key, adcode
  );

  let weather_resp: WeatherResponse = reqwest::get(&weather_url).await?.json().await?;

  if weather_resp.status != "1" {
    eprintln!("天气查询失败");
    return Err(reqwest::get("http://error").await.unwrap_err());
  }

  Ok(weather_resp)
}
