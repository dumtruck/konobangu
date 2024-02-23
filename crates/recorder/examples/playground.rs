use eyre::Context;
#[allow(unused_imports)]
use loco_rs::{cli::playground, prelude::*};
use recorder::app::App;

async fn fetch_and_parse_rss_demo () -> eyre::Result<()> {
    let url =
    "https://mikanani.me/RSS/MyBangumi?token=FE9tccsML2nBPUUqpCuJW2uJZydAXCntHJ7RpD9LDP8%3d";

    let res = reqwest::get(url).await?.bytes().await?;
    let channel = rss::Channel::read_from(&res[..])?;
    println!("channel: {:#?}", channel);
    Ok(())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {

    fetch_and_parse_rss_demo().await?;

    // let active_model: articles::ActiveModel = ActiveModel {
    //     title: Set(Some("how to build apps in 3 steps".to_string())),
    //     content: Set(Some("use Loco: https://loco.rs".to_string())),
    //     ..Default::default()
    // };
    // active_model.insert(&ctx.db).await.unwrap();

    // let res = articles::Entity::find().all(&ctx.db).await.unwrap();
    // println!("{:?}", res);
    println!("welcome to playground. edit me at `examples/playground.rs`");

    Ok(())
}
