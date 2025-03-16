use dotenv::dotenv;
use reqwest;
use std::env;

use reqwest::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct WeatherError {
  message: String,
}

// 行政区域响应结构体
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

// 天气响应结构体
#[derive(Debug, Serialize, Deserialize)]
struct WeatherResponse {
  status: String,
  count: String,
  info: String,
  infocode: String,
  forecasts: Vec<Forecast>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Forecast {
  city: String,
  adcode: String,
  province: String,
  reporttime: String,
  casts: Vec<Cast>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Cast {
  date: String,
  week: String,
  dayweather: String,
  nightweather: String,
  daytemp: String,
  nighttemp: String,
  daywind: String,
  nightwind: String,
  daypower: String,
  nightpower: String,
  #[serde(rename = "daytemp_float")]
  daytemp_float: String,
  #[serde(rename = "nighttemp_float")]
  nighttemp_float: String,
}

async fn get_weather(location: &str, api_key: &str) -> Result<WeatherResponse, Error> {
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

#[tokio::main]
async fn main() {
  dotenv().ok();
  let api_key = env::var("AMAP_API_KEY").unwrap();
  let location = "广州";

  match get_weather(location, api_key.as_str()).await {
    Ok(weather_data) => {
      println!("{}", serde_json::to_string_pretty(&weather_data).unwrap());
    }
    Err(e) => {
      eprintln!("请求发生错误：{}", e);
    }
  }
}
