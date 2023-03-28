#![allow(dead_code)]
use crate::commands::utils::{tmdb_util, paginator, structs};
//use poise::serenity_prelude as serenity;

type Context<'a> = poise::Context<'a, structs::Data, Error>;
type Error = Box<dyn std::error::Error + Send + Sync>;

async fn sleep(secs: u64) {
    tokio::time::sleep(tokio::time::Duration::from_secs(secs)).await;
}

/// Base film commands.
#[poise::command(slash_command, rename = "film", subcommands("backdrops", "posters"))]
pub async fn base(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Get a film's backdrops.
#[poise::command(slash_command)]
pub async fn backdrops(
    ctx: Context<'_>,
    #[description = "The film title."] title: String,
    #[description = "The release year of the film."]
    #[min = 1900]
    #[max = 2023]
    year: Option<i32>,
) -> Result<(), Error> {
    ctx.defer().await?;
    let color = ctx
        .author_member()
        .await
        .unwrap()
        .colour(&ctx.serenity_context().cache)
        .unwrap();
    let cache = &ctx.data().backdrop_cache;
    let backdrops = if cache.read().unwrap().contains_key(&title) {
        cache.read().unwrap().get(&title).cloned().unwrap()
    } else {
        let title_clone = title.clone();
        let year = year.unwrap_or(0);
        let handle = tokio::runtime::Handle::current();
        let backdrops = tokio::task::spawn_blocking(move || {
            tmdb_util::get_images(title_clone, year, "backdrops").unwrap()
        })
        .await
        .unwrap();
        drop(handle);
        ctx.data()
            .backdrop_cache
            .write()
            .unwrap()
            .insert(title.clone(), backdrops);
        ctx.data()
            .backdrop_cache
            .read()
            .unwrap()
            .get(&title)
            .cloned()
            .unwrap()
    };
    if !backdrops.0.is_empty() {
        paginator::start_images(
            ctx,
            &backdrops.0,
            color,
            *ctx.author().id.as_u64(),
            &backdrops.1.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
        )
        .await?;
    } else {
        let error_message = ctx.say(format!("Couldn't find `{}` film.", title)).await?;
        sleep(5).await;
        error_message.delete(ctx).await?;
    }
    Ok(())
}

/// Get a film's posters. This only shows US posters.
#[poise::command(slash_command)]
pub async fn posters(
    ctx: Context<'_>,
    #[description = "The film title."] title: String,
    #[description = "The release year of the film."]
    #[min = 1900]
    #[max = 2023]
    year: Option<i32>,
) -> Result<(), Error> {
    ctx.defer().await?;
    let color = ctx
        .author_member()
        .await
        .unwrap()
        .colour(&ctx.serenity_context().cache)
        .unwrap();
    let cache = &ctx.data().poster_cache;
    let posters = if cache.read().unwrap().contains_key(&title) {
        cache.read().unwrap().get(&title).cloned().unwrap()
    } else {
        let title_clone = title.clone();
        let year = year.unwrap_or(0);
        let handle = tokio::runtime::Handle::current();
        let posters = tokio::task::spawn_blocking(move || {
            tmdb_util::get_images(title_clone, year, "posters").unwrap()
        })
        .await
        .unwrap();
        drop(handle);
        ctx.data()
            .poster_cache
            .write()
            .unwrap()
            .insert(title.clone(), posters);
        ctx.data()
            .poster_cache
            .read()
            .unwrap()
            .get(&title)
            .cloned()
            .unwrap()
    };
    if !posters.0.is_empty() {
        paginator::start_images(
            ctx,
            &posters.0,
            color,
            *ctx.author().id.as_u64(),
            &posters.1.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
        )
        .await?;
    } else {
        let error_message = ctx.say(format!("Couldn't find `{}` film.", title)).await?;
        sleep(5).await;
        error_message.delete(ctx).await?;
    }
    Ok(())
}
