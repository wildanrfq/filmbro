use std::{collections::HashMap, sync::RwLock};

type DiaryCache = RwLock<HashMap<String, (String, String, Vec<DiaryResult>)>>;
#[derive(Clone, Debug, Default)]
pub struct DiaryResult {
    pub found: bool,
    pub title: String,
    pub rating: String,
    pub rewatched: bool,
    pub liked: bool,
    pub reviewed: bool,
    pub date: String,
    pub url: String,
}

#[derive(Clone, Debug, Default)]
pub struct FilmResult {
    pub found: bool,
    pub title: String,
    pub tagline: String,
    pub synopsis: String,
    pub rating: String,
    pub genre: String,
    pub duration: String,
    pub directors: String,
    pub countries: String,
    pub poster: String,
    pub info: HashMap<String, String>,
    pub url: String,
}

#[derive(Clone, Debug, Default)]
pub struct ProfileResult {
    pub found: bool,
    pub avatar: String,
    pub username: String,
    pub name: String,
    pub bio: String,
    pub followers: String,
    pub favorites: String,
    pub location: String,
    pub films_count: String,
    pub websites: Vec<String>,
    pub url: String,
}

#[derive(Debug, Default)]
pub struct Data {
    pub diary_cache: DiaryCache,
    pub film_cache: RwLock<HashMap<String, FilmResult>>,
    pub poster_cache: RwLock<HashMap<String, (String, Vec<String>)>>,
    pub backdrop_cache: RwLock<HashMap<String, (String, Vec<String>)>>,
    pub profile_cache: RwLock<HashMap<String, ProfileResult>>,
}

#[allow(dead_code)]
pub type Command = poise::Command<Data, Box<dyn std::error::Error + Send + Sync>>;
