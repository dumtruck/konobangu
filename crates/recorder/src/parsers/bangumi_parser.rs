use crate::parsers::errors::ParseError;

pub fn parse_bangumi_season(season_str: &str) -> Result<i32, ParseError> {
    season_str
        .parse::<i32>()
        .map_err(ParseError::BangumiSeasonError)
}
