use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TmdbListItemDto {
    pub id: i64,
    pub name: String,
    pub adult: bool,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub media_type: String,
    pub original_language: String,
    pub original_name: String,
    pub overview: String,
    pub genre_ids: Vec<i64>,
    pub popularity: f64,
    pub first_air_date: String,
    pub origin_country: Option<Vec<String>>,
    pub vote_average: f32,
    pub vote_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TmdbListPageDto {
    pub id: i64,
    pub page: u32,
    pub sort_by: Option<String>,
    pub total_pages: u32,
    pub total_results: u32,
    pub name: String,
    pub results: Vec<TmdbListItemDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TmdbGenresObjDto {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TmdbEpisodeAirDto {
    pub id: i64,
    pub name: String,
    pub overview: String,
    pub vote_average: f32,
    pub vote_count: i32,
    pub air_date: String,
    pub episode_number: i32,
    pub episode_type: String,
    pub production_code: String,
    pub runtime: Option<i32>,
    pub season_number: i32,
    pub show_id: i64,
    pub still_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TmdbSeasonDto {
    pub air_date: String,
    pub episode_count: i32,
    pub id: i64,
    pub name: String,
    pub overview: String,
    pub poster_path: Option<String>,
    pub season_number: i32,
    pub vote_average: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TmdbSpokenLanguageDto {
    pub iso_639_1: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TmdbTvSeriesDetailDto {
    pub adult: bool,
    pub id: i64,
    pub name: String,
    pub backdrop_path: Option<String>,
    pub episode_run_time: Option<Vec<i32>>,
    pub genres: Vec<TmdbGenresObjDto>,
    pub first_air_date: Option<String>,
    pub home_page: Option<String>,
    pub in_production: bool,
    pub languages: Vec<String>,
    pub last_air_date: Option<String>,
    pub last_episode_to_air: Option<TmdbEpisodeAirDto>,
    pub next_episode_to_air: Option<TmdbEpisodeAirDto>,
    pub number_of_episodes: i32,
    pub number_of_seasons: i32,
    pub origin_country: Vec<String>,
    pub original_language: String,
    pub original_name: String,
    pub overview: String,
    pub popularity: f32,
    pub poster_path: Option<String>,
    pub seasons: Vec<TmdbSeasonDto>,
    pub spoken_languages: Vec<TmdbSpokenLanguageDto>,
    pub status: String,
    pub tagline: String,
    pub vote_average: f32,
    pub vote_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TmdbMovieDetailDto {
    pub adult: bool,
    pub backdrop_path: Option<String>,
    pub homepage: Option<String>,
    pub id: i64,
    pub budget: i64,
    pub imdb_id: Option<String>,
    pub original_language: String,
    pub original_title: String,
    pub overview: String,
    pub popularity: f32,
    pub poster_path: Option<String>,
    pub release_date: String,
    pub revenue: i32,
    pub runtime: Option<i32>,
    pub spoken_languages: Vec<TmdbSpokenLanguageDto>,
    pub status: String,
    pub tagline: String,
    pub title: String,
    pub video: bool,
    pub vote_average: f32,
    pub vote_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TmdbSearchMultiItemDto {
    pub adult: bool,
    pub backdrop_path: Option<String>,
    pub id: i64,
    pub name: String,
    pub original_language: String,
    pub original_name: String,
    pub overview: String,
    pub poster_path: Option<String>,
    pub media_type: String,
    pub genre_ids: Vec<i64>,
    pub popularity: f32,
    pub first_air_date: Option<String>,
    pub vote_average: f32,
    pub vote_count: i32,
    pub origin_country: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TmdbMediaDetailDto {
    Tv(TmdbTvSeriesDetailDto),
    Movie(TmdbMovieDetailDto),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TmdbSearchMultiPageDto {
    pub total_results: u32,
    pub total_pages: u32,
    pub page: u32,
    pub results: Vec<TmdbSearchMultiItemDto>,
}