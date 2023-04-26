#![allow(dead_code)]

use crate::config;
use poise::serenity_prelude::json;

pub fn get_images(
    film: String,
    year: i32,
    choice: &str,
) -> Result<(String, Vec<String>), Box<dyn std::error::Error>> {
    let year = if year != 0 {
        format!("&year={}", year)
    } else {
        String::new()
    };
    let data: json::Value = reqwest::blocking::get(format!("https://api.themoviedb.org/3/search/movie?api_key={}&language=en-US&query={}&page=1&include_adult=false{}", config::TMDB_API_TOKEN, film, year))?.json()?;
    let results = &data["results"].as_array().unwrap();
    if !results.is_empty() {
        let film_id = &results[0]["id"];
        let title = format!(
            "{} ({})",
            &results[0]["original_title"].as_str().unwrap(),
            &results[0]["release_date"].as_str().unwrap()[..4]
        );
        let posters_check = if choice == "posters" {
            "&language=en-US&include_image_language=en"
        } else {
            ""
        };
        let images: json::Value = reqwest::blocking::get(format!(
            "https://api.themoviedb.org/3/movie/{}/images?api_key={}{}",
            film_id,
            config::TMDB_API_TOKEN,
            posters_check
        ))?
        .json()?;
        let posters = &images[choice]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| {
                format!(
                    "https://www.themoviedb.org/t/p/original{}",
                    p["file_path"].as_str().unwrap()
                )
            })
            .collect::<Vec<_>>()
            .to_vec();
        Ok((title, posters.to_vec()))
    } else {
        Ok((String::new(), vec![]))
    }
}
