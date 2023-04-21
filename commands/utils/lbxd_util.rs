#![allow(unused_variables, dead_code)]

use crate::commands::utils::structs::*;

use html_escape::decode_html_entities as decode_html;
use rand::Rng;
use regex::Regex;
use reqwest::{blocking::get as reqget, header::HeaderValue};
use scraper::{Html, Selector};

use std::collections::HashMap;

pub trait HeaderValueExt {
    fn to_string(&self) -> String;
}

impl HeaderValueExt for HeaderValue {
    fn to_string(&self) -> String {
        self.to_str().unwrap_or_default().to_string()
    }
}

fn format_number(n: &str) -> String {
    let number = n.replace(",", "").parse::<f64>().unwrap();
    match number.abs() {
        n if n >= 1_000_000.0 => format!("{:.2}m", number / 1_000_000.0),
        n if n >= 100_000.0 && n < 1_000_000.0 => format!("{:.1}k", number / 1_000.0),
        n if n >= 1_000.0 => format!("{:.1}k", number / 1_000.0),
        _ => format!("{:.0}", number),
    }
}

fn selector(selector: impl Into<String>) -> scraper::Selector {
    Selector::parse(&selector.into()).unwrap()
}

fn starrize(rating: f32) -> String {
    if rating == 0.0 {
        return String::new();
    }

    let clean_rating = (rating * 2.0).round() / 2.0;

    let rounded = clean_rating.floor() as usize;
    let mut star_string = "<:lbstar:1061604009783341117>".repeat(rounded);

    if clean_rating.fract() != 0.0 {
        star_string += "<:lbhstar:1061603475949096991>";
    }

    star_string
}

fn convert_duration(minutes: u32) -> String {
    let hours = minutes / 60;
    let minutes = minutes % 60;
    match (hours, minutes) {
        (0, m) => format!("{}m", m),
        (h, 0) => format!("{}h", h),
        (h, m) => format!("{}h {}m", h, m),
    }
}

fn format_bio(text: &str) -> String {
    let hyperlink_regex = build_regex(r#"<a.*href="(?P<link>.*?)".*>(?P<title>.*?)</a>"#);
    decode_html(
        hyperlink_regex
            .replace_all(text, "[$title]($link)")
            .into_owned()
            .replace("<p>", "")
            .replace("</p>", "\n")
            .replace("<br>", "\n")
            .replace("<b>", "**")
            .replace("</b>", "**")
            .replace("<i>", "*")
            .replace("</i>", "*")
            .trim(),
    )
    .to_string()
}

fn build_regex(pat: &str) -> Regex {
    Regex::new(pat).unwrap()
}
pub fn get_diary(
    username: String,
) -> Result<(String, String, Vec<DiaryResult>), Box<dyn std::error::Error>> {
    const BASE_URL: &str = "https://letterboxd.com";
    let search_diary = reqget(format!("{}/{}/films/diary", BASE_URL, username))?.text()?;

    if search_diary.contains("Sorry, we can’t find the page you’ve requested.") {
        return Ok((
            String::new(),
            String::new(),
            vec![DiaryResult {
                found: false,
                ..Default::default()
            }],
        ));
    }

    if search_diary.contains("No diary entries") {
        return Ok((
            String::new(),
            String::new(),
            vec![DiaryResult {
                found: true,
                title: "Not found".to_string(),
                ..Default::default()
            }],
        ));
    }

    let sd_html = Html::parse_document(&search_diary);
    let tbody_selector = selector("tbody");
    let entries_selector = selector("tr");
    let entries = sd_html
        .select(&tbody_selector)
        .next()
        .unwrap()
        .select(&entries_selector);
    let mut diaries_vec: Vec<DiaryResult> = vec![];
    let avatar_selector = selector(r#"img[width="24"]"#);
    let avatar_raw = sd_html
        .select(&avatar_selector)
        .next()
        .unwrap()
        .value()
        .attr("src")
        .unwrap();
    let avatar = if avatar_raw.contains("static") {
        String::new()
    } else {
        avatar_raw.replace("0-48-0-48", "0-220-0-220").to_string()
    };
    let display_name_selector = selector(r#"meta[property="og:title"]"#);
    let display_name = sd_html
        .select(&display_name_selector)
        .next()
        .unwrap()
        .value()
        .attr("content")
        .unwrap()
        .split("’s")
        .collect::<Vec<_>>()[0]
        .to_string();
    for entry in entries.take(5) {
        let info_selector = selector(r#"a[class="edit-review-button has-icon icon-16 icon-edit"]"#);
        let data = entry.select(&info_selector).next().unwrap().value();
        let title = format!(
            "{} ({})",
            data.attr("data-film-name").unwrap(),
            data.attr("data-film-year").unwrap()
        );
        let url = format!(
            "{}{}",
            BASE_URL,
            data.attr("data-film-poster")
                .unwrap()
                .replace("/image-150/", "")
        );
        let date_raw = data.attr("data-viewing-date-str").unwrap().to_string();
        let date = if date_raw.contains("2023") {
            date_raw
                .replace(" 2023", "")
                .split(" ")
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            let dates = date_raw.split(" ").collect::<Vec<_>>();
            format!("{} {}, {}", dates[1], dates[0], dates[2])
        };
        let rating = starrize(data.attr("data-rating").unwrap().parse::<f32>().unwrap() / 2.0);
        let reviewed = !data
            .attr("data-review-text")
            .unwrap()
            .to_string()
            .is_empty();
        let rewatched: bool = data.attr("data-rewatch").unwrap().trim().parse().unwrap();
        let liked_selector =
            selector(r#"span[class="has-icon icon-16 large-liked icon-liked hide-for-owner"]"#);
        let liked = entry.select(&liked_selector).next().is_some();
        diaries_vec.push(DiaryResult {
            found: true,
            title: title,
            rating: rating,
            date: date,
            rewatched: rewatched,
            liked: liked,
            reviewed: reviewed,
            url: url,
        });
    }
    Ok((avatar, display_name, diaries_vec))
}

pub fn get_film(title: &str) -> Result<FilmResult, Box<dyn std::error::Error>> {
    const BASE_URL: &str = "https://letterboxd.com";
    let title_regex = build_regex(
        r#"([^[:ascii:][:alnum:]'\s]|^)([[:ascii:][:alnum:]'\s\u{4e00}-\u{9fff}]*)([^[:ascii:][:alnum:]'\s]|$)"#,
    );
    let new_title = title_regex
        .replace_all(&title.to_lowercase(), "$2")
        .to_string();
    let search_film = reqget(BASE_URL.to_string() + "/search/films/" + &new_title + "/?adult")?;
    let sf_ul = selector("ul.results");
    let sf_li = selector("li");
    let sf_div = selector("div");
    let sf_html = Html::parse_document(&search_film.text()?);
    let sf_ul2 = sf_html.select(&sf_ul).next().unwrap();
    let film_url = sf_ul2
        .select(&sf_li)
        .next()
        .unwrap()
        .select(&sf_div)
        .next()
        .unwrap()
        .value()
        .attr("data-target-link")
        .unwrap();
    let film = reqget(BASE_URL.to_string() + film_url)?.text()?;
    let info_film = reqget(BASE_URL.to_string() + film_url + "/reviews")?.text()?;
    let html_film = Html::parse_document(&film);
    let title_selector = selector(r#"meta[property="og:title"]"#);
    let title = html_film
        .select(&title_selector)
        .next()
        .unwrap()
        .value()
        .attr("content")
        .unwrap();
    let syn_selector = selector(r#"meta[name="description"]"#);
    let synopsis_raw: String;
    let syn = html_film.select(&syn_selector).next();
    synopsis_raw = if syn.is_some() {
        syn.unwrap().value().attr("content").unwrap().to_string()
    } else {
        String::new()
    };
    let synopsis = if synopsis_raw.len() > 100 {
        format!("{}...", &synopsis_raw[..100]).to_string()
    } else {
        synopsis_raw.to_string()
    };
    let tag_selector = selector("h4.tagline");
    let tagline_check = html_film.select(&tag_selector).next();
    let tagline = if let Some(tag) = tagline_check {
        tag.inner_html()
    } else {
        "".to_string()
    };
    let poster_pattern = build_regex(r#""image":"([^\s"']+)"#);
    let poster = if poster_pattern.captures(&film).is_some() {
        poster_pattern
            .captures(&film)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
    } else {
        ""
    };
    let rating_selector = selector(r#"meta[name="twitter:data2"]"#);
    let rating_point: f32 = if html_film.select(&rating_selector).next().is_some() {
        html_film
            .select(&rating_selector)
            .next()
            .unwrap()
            .value()
            .attr("content")
            .unwrap()
            .split(" out")
            .next()
            .unwrap()
            .parse()
            .unwrap()
    } else {
        0.0
    };

    let rating = format!(
        "{} {}{}",
        starrize(rating_point),
        rating_point,
        ["", ".0"][(rating_point.fract() == 0.0) as usize]
    );
    let directors_selector = selector(r#"meta[name="twitter:data1"]"#);
    let directors = html_film
        .select(&directors_selector)
        .next()
        .unwrap()
        .value()
        .attr("content")
        .unwrap()
        .to_string();
    let countries_pattern = build_regex(r#"/films/country/.*/" class=".*">(.*)</a>"#);
    let countries_raw = countries_pattern.captures(&film);
    let countries = if countries_raw.is_some() {
        countries_raw
            .unwrap()
            .get(0)
            .map(|m| m.as_str())
            .unwrap_or_default()
            .split(r#"text-slug">"#)
            .filter_map(|c| {
                if c.contains("</a>") {
                    Some(c.split("</a>").next().unwrap_or_default().to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        "".to_string()
    };
    let duration_selector = selector(r#"p[class="text-link text-footer"]"#);
    let duration_regex = build_regex(r#"(\d+)&nbsp;mins &nbsp;"#);
    let duration_raw = html_film
        .select(&duration_selector)
        .next()
        .unwrap()
        .inner_html();
    let duration_str_raw = duration_regex.captures(&duration_raw);
    let duration_str = if duration_str_raw.is_some() {
        duration_str_raw.unwrap().get(1).unwrap().as_str()
    } else {
        "0"
    };
    let duration = convert_duration(duration_str.parse::<u32>().unwrap());
    let genre_regex = build_regex(r#""genre":[\[](.*)"[\]]"#);
    let genre_raw = genre_regex.captures(&film);
    let genre = if genre_raw.is_some() {
        genre_raw
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .replace(r#"""#, "")
            .replace(",", ", ")
    } else {
        "".to_string()
    };
    let info_regex = build_regex(r#"title="(.*)&nbsp;(people|likes|reviews)"#);
    let mut info: HashMap<String, String> = HashMap::new();
    for i in info_regex.captures_iter(&info_film) {
        info.insert(i[2].to_string(), format_number(&i[1]));
    }
    let result = FilmResult {
        found: true,
        title: title.to_string(),
        tagline: tagline.to_string(),
        synopsis: synopsis.to_string(),
        rating: rating.to_string(),
        duration: duration.to_string(),
        directors: directors,
        countries: countries,
        genre: genre,
        info: info,
        poster: poster.to_string(),
        url: BASE_URL.to_string() + film_url,
    };
    Ok(result)
}

pub fn get_profile(username: &str) -> Result<ProfileResult, Box<dyn std::error::Error>> {
    const BASE_URL: &str = "https://letterboxd.com";
    let profile_url = format!("{}/{}", BASE_URL, username);
    let search_profile = reqget(&profile_url)?.text()?;

    if search_profile.contains("Sorry, we can’t find the page you’ve requested.") {
        return Ok(ProfileResult {
            found: false,
            ..Default::default()
        });
    }

    let sp_html = Html::parse_document(&search_profile);
    let username_selector = selector(r#"div[data-profile="true"]"#);
    let actual_username = sp_html
        .select(&username_selector)
        .next()
        .unwrap()
        .value()
        .attr("data-username")
        .unwrap()
        .to_string();
    let name_selector = selector(r#"meta[property="og:title"]"#);
    let name = sp_html
        .select(&name_selector)
        .next()
        .unwrap()
        .value()
        .attr("content")
        .unwrap()
        .split("’s profile")
        .collect::<Vec<_>>()[0]
        .to_string();
    let location_selector = selector(r#"div[class="metadatum -has-label js-metadatum"]"#);
    let location = sp_html
        .select(&location_selector)
        .next()
        .map(|elem| {
            format!(
                "📍 ***{}***",
                elem.text().collect::<Vec<_>>().join("").trim().to_string()
            )
        })
        .unwrap_or_else(String::new);
    let links_selector = selector(r#"a[class="metadatum -has-label js-metadatum"]"#);
    let websites = sp_html
        .select(&links_selector)
        .map(|link| link.value().attr("href").unwrap().to_string())
        .collect::<Vec<_>>();
    let favorites_section_selector = selector(r#"section[id="favourites"]"#);
    let favorites_section = sp_html.select(&favorites_section_selector).next().unwrap();
    let favorites_selector = selector("div");
    let favorites_links = favorites_section.select(&favorites_selector);
    let mut favorites_link = vec![];
    let description_selector = selector(r#"meta[name="description"]"#);
    let description_raw = sp_html
        .select(&description_selector)
        .next()
        .unwrap()
        .value()
        .attr("content")
        .unwrap();
    let mut favorites = String::new();
    if description_raw.contains("Favorites: ") {
        let description = description_raw.split("Favorites: ").collect::<Vec<_>>()[1];
        let favorite_titles = if description.contains("Bio: ") {
            description.split(". Bio: ").collect::<Vec<_>>()[0]
                .split(", ")
                .collect::<Vec<_>>()
        } else {
            let new_description = &description[..description.len() - 1];
            new_description.split(", ").collect::<Vec<_>>()
        };
        for favorite in favorites_links {
            favorites_link
                .push(BASE_URL.to_owned() + favorite.value().attr("data-film-slug").unwrap());
        }
        for (link, title) in favorites_link.iter().zip(favorite_titles.iter()) {
            favorites.push_str(&format!("• [{}]({})\n", title, link));
        }
        favorites.pop();
    }
    let avatar_selector = selector(r#"img[width="110"]"#);
    let avatar_raw = sp_html
        .select(&avatar_selector)
        .next()
        .unwrap()
        .value()
        .attr("src")
        .unwrap();
    let avatar = if avatar_raw.contains("static") {
        String::new()
    } else {
        avatar_raw.to_string()
    };
    let bio_selector = selector(r#"section[id="person-bio"]"#);
    let bio_raw = sp_html.select(&bio_selector).next();
    let bio = if bio_raw.is_some() {
        let div_bio_selector = selector(r#"div[class="collapsed-text"]"#);
        let div_bio = bio_raw.unwrap().select(&div_bio_selector).next();
        if div_bio.is_some() {
            format_bio(&div_bio.unwrap().inner_html())
        } else {
            let medium_bio_selector = selector(r#"div[class="collapsible-text body-text -small"]"#);
            let medium_bio = bio_raw.unwrap().select(&medium_bio_selector).next();
            format_bio(&medium_bio.unwrap().inner_html())
        }
    } else {
        let short_div_bio_selector =
            Selector::parse(r#"div[class="collapsible-text body-text -small js-bio-content"]"#)
                .unwrap();
        let check_short_bio = sp_html.select(&short_div_bio_selector).next();
        if check_short_bio.is_some() {
            format_bio(&check_short_bio.unwrap().inner_html())
        } else {
            String::new()
        }
    };
    let data_selector = selector(r#"h4[class="profile-statistic statistic"]"#);
    let mut films = sp_html.select(&data_selector);
    let mut films_count = String::new();
    for film in films.by_ref().take(2) {
        if films_count.is_empty() {
            films_count.push_str(&format!(
                "{} films logged, ",
                film.text().collect::<Vec<_>>()[0]
            ))
        } else {
            films_count.push_str(&format!(
                "{} this year.",
                film.text().collect::<Vec<_>>()[0]
            ))
        }
    }
    let followers = films.last().unwrap().text().collect::<Vec<_>>()[0].to_string();
    Ok(ProfileResult {
        found: true,
        avatar: avatar,
        bio: bio,
        username: actual_username,
        name: name,
        followers: followers,
        favorites: favorites,
        location: location,
        films_count: films_count,
        websites: websites,
        url: profile_url,
    })
}

fn generate_lbxd_link() -> String {
    let mut rng = rand::thread_rng();
    let x: Vec<char> = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
        .chars()
        .collect();
    format!(
        "https://boxd.it/{}",
        (0..6)
            .map(|_| x[rng.gen_range(0..x.len())])
            .collect::<String>()
    )
}

pub fn get_roulette() -> Result<FilmResult, Box<dyn std::error::Error>> {
    let mut url = generate_lbxd_link();
    let mut hd;
    let res = loop {
        let c = url.clone();
        if url[url.len() - 4..].contains("/") {
            url = generate_lbxd_link()
        }
        let cc = c.clone();
        dbg!(&url[url.len() - 4..]);
        if let Ok(res) = reqget(c) {
            hd = res.headers().clone();
            if ["Film", "LogEntry"].contains(
                &hd.get("x-letterboxd-type")
                    .map(|h| h.to_string())
                    .unwrap_or_else(|| "X".to_string())
                    .as_str(),
            ) {
                break res;
            } else {
                url.pop();
            }
        }
    };
    let res_html = Html::parse_document(&res.text()?.clone());
    let header = hd.get("x-letterboxd-type").unwrap().to_str().unwrap();
    let title = if header == "Film" {
        let title_selector = selector(r#"meta[property="og:title"]"#);
        res_html
            .select(&title_selector)
            .next()
            .unwrap()
            .value()
            .attr("content")
            .unwrap()
    } else {
        let title_selector = selector(r#"meta[property="og:title"]"#);
        let rating = res_html
            .select(&title_selector)
            .next()
            .unwrap()
            .value()
            .attr("content")
            .unwrap();
        if rating.contains("entry for") {
            rating.split("entry for ").collect::<Vec<_>>()[1]
        } else {
            rating.split("review of ").collect::<Vec<_>>()[1]
        }
    };
    get_film(&title)
}
