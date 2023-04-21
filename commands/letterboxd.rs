use crate::commands::utils::{lbxd_util, structs};

use poise::serenity_prelude as serenity;
use serenity::{
    model::{id::EmojiId, misc::EmojiIdentifier},
    ReactionType::Unicode,
};
use tokio::{runtime::Handle, task::spawn_blocking};

type Context<'a> = poise::Context<'a, structs::Data, Error>;
type Error = Box<dyn std::error::Error + Send + Sync>;

async fn sleep(secs: u64) {
    tokio::time::sleep(tokio::time::Duration::from_secs(secs)).await;
}

/// Base Letterboxd commands.
#[poise::command(
    slash_command,
    rename = "letterboxd",
    subcommands("diary", "film", "profile", "roulette")
)]
pub async fn base(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Get the recent diary entries from a Letterboxd profile.
#[poise::command(slash_command)]
pub async fn diary(
    ctx: Context<'_>,
    #[description = "The profile username."] username: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let color = ctx
        .author_member()
        .await
        .unwrap()
        .colour(&ctx.serenity_context().cache)
        .unwrap();
    let cache = &ctx.data().diary_cache;
    let diaries = if cache.read().unwrap().contains_key(&username) {
        cache.read().unwrap().get(&username).cloned().unwrap()
    } else {
        let username_clone = username.clone();
        let handle = Handle::current();
        let diaries = spawn_blocking(move || lbxd_util::get_diary(username_clone).unwrap())
            .await
            .unwrap();
        drop(handle);
        ctx.data()
            .diary_cache
            .write()
            .unwrap()
            .insert(username.clone(), diaries);
        ctx.data()
            .diary_cache
            .read()
            .unwrap()
            .get(&username)
            .cloned()
            .unwrap()
    };
    if !diaries.0.is_empty() {
        let mut description = String::new();
        for diary in diaries.2 {
            let rewatched = if diary.rewatched {
                "🔄 ".to_string()
            } else {
                String::new()
            };
            let liked = if diary.liked {
                "❤️ ".to_string()
            } else {
                String::new()
            };
            let reviewed = if diary.reviewed {
                "💬".to_string()
            } else {
                String::new()
            };
            let rating = if !diary.rating.is_empty() {
                format!("{} ", diary.rating)
            } else {
                String::new()
            };
            description.push_str(
                format!(
                    "[**{}**]({})\n{} {}{}{}{}\n",
                    diary.title, diary.url, diary.date, rating, rewatched, liked, reviewed,
                )
                .as_str(),
            )
        }
        ctx.send(|m| {
            m.embed(|e| {
                e.title(format!("{}'s Recent Diary Entries", diaries.1))
                    .description(description)
                    .url(format!("https://letterboxd.com/{}/films/diary", username))
                    .color(color)
                    .thumbnail(diaries.0)
            })
        })
        .await?;
    } else {
        if diaries.2[0].title.is_empty() {
            let error_message = ctx.say(format!(
                "Couldn't find `{}` user.\nMake sure to provide your Letterboxd **username**, not the link.",
                username
            )).await?;
            sleep(5).await;
            error_message.delete(ctx).await?;
        } else {
            let error_message = ctx
                .say(format!("`{}` doesn't have any recent diaries.", username))
                .await?;
            sleep(5).await;
            error_message.delete(ctx).await?;
        }
    }
    Ok(())
}

/// Get a film information based off Letterboxd.
#[poise::command(slash_command)]
pub async fn film(
    ctx: Context<'_>,
    #[description = "The film title."] title: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let cache = &ctx.data().film_cache;
    let film_info = if cache.read().unwrap().contains_key(&title) {
        cache.read().unwrap().get(&title).cloned().unwrap()
    } else {
        let title_clone = title.clone();
        let handle = Handle::current();
        let film_info = spawn_blocking(move || lbxd_util::get_film(&title_clone).unwrap())
            .await
            .unwrap();
        drop(handle);
        ctx.data()
            .film_cache
            .write()
            .unwrap()
            .insert(title.clone(), film_info);
        ctx.data()
            .film_cache
            .read()
            .unwrap()
            .get(&title)
            .cloned()
            .unwrap()
    };
    if film_info.found {
        let color = ctx
            .author_member()
            .await
            .unwrap()
            .colour(&ctx.serenity_context().cache)
            .unwrap();
        let tagline = if !film_info.tagline.is_empty() {
            format!("**{}**\n", film_info.tagline)
        } else {
            String::new()
        };
        let rating = if film_info.rating != " 0.0".to_string() {
            format!("{}\n", film_info.rating)
        } else {
            "".to_string()
        };
        let plural_check = vec!["", "s"][(film_info.directors.split(", ").count() > 1) as usize];
        let country_check = if !film_info.countries.is_empty() {
            "|"
        } else {
            ""
        };
        let duration = if film_info.duration != "0m".to_string() {
            format!("{}\n", film_info.duration)
        } else {
            "".to_string()
        };
        ctx.send(|m| {
                m.embed(|e| {
                    e.title(film_info.title)
                    .description(
                        format!(
                            "{}{}\n\n{}Director{}: {}\n{} {} {}\n{}\u{1f440} {} | ❤️ {} | \u{1f4ac} {}",
                            tagline,
                            film_info.synopsis,
                            rating,
                            plural_check,
                            film_info.directors,
                            film_info.countries,
                            country_check,
                            film_info.genre,
                            duration,
                            film_info.info["people"],
                            film_info.info["likes"],
                            film_info.info.get("reviews").unwrap_or(&"0".to_string())
                        )
                    )
                    .url(film_info.url)
                    .color(color)
                    .thumbnail(film_info.poster)
                })
        })
        .await?;
    } else {
        let error_message = ctx.say(format!("Couldn't find `{}` film.", title)).await?;
        sleep(5).await;
        error_message.delete(ctx).await?;
    }
    Ok(())
}

/// Get a Letterboxd profile information.
#[poise::command(slash_command)]
pub async fn profile(
    ctx: Context<'_>,
    #[description = "The profile username."] username: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let cache = &ctx.data().profile_cache;
    let user = if cache.read().unwrap().contains_key(&username) {
        cache.read().unwrap().get(&username).cloned().unwrap()
    } else {
        let username_clone = username.clone();
        let handle = Handle::current();
        let user = spawn_blocking(move || lbxd_util::get_profile(&username_clone).unwrap())
            .await
            .unwrap();
        drop(handle);
        ctx.data()
            .profile_cache
            .write()
            .unwrap()
            .insert(username.clone(), user);
        ctx.data()
            .profile_cache
            .read()
            .unwrap()
            .get(&username)
            .cloned()
            .unwrap() //&ctx.data().profile_cache.get(&username).unwrap()
    };
    if user.found {
        let mut description = String::new();
        if !user.location.is_empty() || !user.bio.is_empty() {
            description.push_str(&format!(
                "{}\n{}\n———————————————\n",
                user.location, user.bio
            ));
        }
        if !user.favorites.is_empty() {
            description.push_str(&format!("{}\n", user.favorites));
        }
        let color = ctx
            .author_member()
            .await
            .unwrap()
            .colour(&ctx.serenity_context().cache)
            .unwrap();
        ctx.send(|m| {
                if user.websites.len() != 0 {
                    m.components(|c| {
                        c.create_action_row(|ar| {
                            if user.websites.len() == 1 {
                                if !user.websites[0].contains("twitter") {
                                    ar.create_button(|b| {
                                    b.style(serenity::ButtonStyle::Link).label("Website").url(&user.websites[0]).emoji(Unicode("🌐".to_string()))
                                })
                                } else {
                                    ar.create_button(|b| {
                                    b.style(serenity::ButtonStyle::Link).label("Twitter").url(&user.websites[0]).emoji(EmojiIdentifier { animated: false , id: EmojiId(1083962148633456670), name: "twt".to_string()})
                                })
                                }
                            } else {
                                ar.create_button(|b| {
                                    b.style(serenity::ButtonStyle::Link).label("Website").url(&user.websites[0]).emoji(Unicode("🌐".to_string()))
                                });
                                ar.create_button(|b| {
                                    b.style(serenity::ButtonStyle::Link).label("Twitter").url(&user.websites[1]).emoji(EmojiIdentifier { animated: false , id: EmojiId(1083962148633456670), name: "twt".to_string()})
                                })
                            }
                        })
                    });
                }
                m.embed(|e| {
                    if !user.avatar.is_empty() {
                        e.thumbnail(user.avatar);
                    }
                    e.author(|a| a
                        .icon_url("https://cdn.discordapp.com/emojis/710193146843365457.webp?size=96&quality=lossless")
                        .name(user.username)
                    )
                    .title(user.name)
                    .description(description)
                    .color(color)
                    .url(user.url)
                    .footer(|f| f.text(format!("{} follower{}, {}", user.followers, ["", "s"][(user.followers.replace(",", "").parse::<i32>().unwrap() > 1) as usize], user.films_count)))
                })}).await?;
    } else {
        let error_message = ctx.say(format!(
                "Couldn't find `{}` user.\nMake sure to provide your Letterboxd **username**, not the link.",
                username
            )).await?;
        sleep(5).await;
        error_message.delete(ctx).await?;
    }
    Ok(())
}

/// A roulette to get a random film off Letterboxd.
#[poise::command(slash_command)]
pub async fn roulette(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let wait = ctx.say("Please wait...").await?;
    let handle = Handle::current();
    let film_info = spawn_blocking(move || {
        lbxd_util::get_roulette().unwrap_or_else(|_| {
            lbxd_util::get_roulette().unwrap_or_else(|_| lbxd_util::get_roulette().unwrap())
        })
    })
    .await
    .unwrap();
    drop(handle);
    wait.edit(ctx, |m| m.content("Fetching information..."))
        .await?;
    let color = ctx
        .author_member()
        .await
        .unwrap()
        .colour(&ctx.serenity_context().cache)
        .unwrap();
    let tagline = if !film_info.tagline.is_empty() {
        format!("**{}**\n", film_info.tagline)
    } else {
        String::new()
    };
    let rating = if film_info.rating != " 0.0".to_string() {
        format!("{}\n", film_info.rating)
    } else {
        "".to_string()
    };
    let plural_check = vec!["", "s"][(film_info.directors.split(", ").count() > 1) as usize];
    let country_check = if !film_info.countries.is_empty() {
        "|"
    } else {
        ""
    };
    let duration = if film_info.duration != "0m".to_string() {
        format!("{}\n", film_info.duration)
    } else {
        "".to_string()
    };
    wait.edit(ctx, |m| {
            m.content("")
            .embed(|e| {
                e.title(film_info.title)
                .description(
                    format!(
                        "{}{}\n\n{}Director{}: {}\n{} {} {}\n{}\u{1f440} {} | ❤️ {} | \u{1f4ac} {}",
                        tagline,
                        film_info.synopsis,
                        rating,
                        plural_check,
                        film_info.directors,
                        film_info.countries,
                        country_check,
                        film_info.genre,
                        duration,
                        film_info.info["people"],
                        film_info.info["likes"],
                        film_info.info.get("reviews").unwrap_or(&"0".to_string())
                    )
                )
                .url(film_info.url)
                .color(color)
                .thumbnail(film_info.poster)
            })
    })
    .await?;
    Ok(())
}
