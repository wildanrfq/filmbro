// paginator code originates from poise (poise/builtins/paginate.rs), I modified some part of it.

use poise::serenity_prelude as serenity;
use serenity::ReactionType::Unicode;

#[allow(dead_code)]
pub async fn start_images<U, E>(
    ctx: poise::Context<'_, U, E>,
    title: &str,
    color: serenity::Colour,
    author: u64,
    pages: &[&str],
) -> Result<(), serenity::Error> {
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx.id());
    let next_button_id = format!("{}next", ctx.id());
    let first_button_id = format!("{}first", ctx.id());
    let last_button_id = format!("{}last", ctx.id());

    let mut current_page = 0;
    ctx.send(|b| {
        b.embed(|b| {
            b.title(title)
                .color(color)
                .image(pages[current_page])
                .footer(|f| f.text(format!("Page {}/{}", current_page + 1, pages.len())))
        })
        .components(|b| {
            b.create_action_row(|b| {
                b.create_button(|b| {
                    b.custom_id(&first_button_id)
                        .emoji(Unicode("⏪".to_string()))
                        .disabled(true)
                })
                .create_button(|b| {
                    b.custom_id(&prev_button_id)
                        .emoji(Unicode("◀️".to_string()))
                        .disabled(true)
                })
                .create_button(|b| b.custom_id(&next_button_id).emoji(Unicode("▶️".to_string())))
                .create_button(|b| {
                    b.custom_id(&last_button_id)
                        .emoji(Unicode("⏩".to_string()))
                })
            })
        })
    })
    .await?;

    while let Some(press) = serenity::CollectComponentInteraction::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(3600 * 24))
        .author_id(author)
        .await
    {
        if press.data.custom_id == next_button_id {
            current_page += 1;
            if current_page >= pages.len() {
                current_page = 0;
            }
        } else if press.data.custom_id == prev_button_id {
            current_page = current_page.checked_sub(1).unwrap_or(pages.len() - 1);
        } else if press.data.custom_id == first_button_id {
            current_page = 0;
        } else if press.data.custom_id == last_button_id {
            current_page = pages.len() - 1;
        } else {
            continue;
        }

        press
            .create_interaction_response(ctx, |b| {
                b.kind(serenity::InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|b| {
                        b.embed(|b| {
                            b.title(title)
                                .color(color)
                                .footer(|f| {
                                    f.text(format!("Page {}/{}", current_page + 1, pages.len()))
                                })
                                .image(pages[current_page])
                        })
                        .components(|b| {
                            b.create_action_row(|b| {
                                b.create_button(|b| {
                                    b.custom_id(&first_button_id)
                                        .emoji(Unicode("⏪".to_string()))
                                        .disabled(current_page == 0)
                                })
                                .create_button(|b| {
                                    b.custom_id(&prev_button_id)
                                        .emoji(Unicode("◀️".to_string()))
                                        .disabled(current_page == 0)
                                })
                                .create_button(|b| {
                                    b.custom_id(&next_button_id)
                                        .emoji(Unicode("▶️".to_string()))
                                        .disabled(current_page == pages.len() - 1)
                                })
                                .create_button(|b| {
                                    b.custom_id(&last_button_id)
                                        .emoji(Unicode("⏩".to_string()))
                                        .disabled(current_page == pages.len() - 1)
                                })
                            })
                        })
                    })
            })
            .await?;
    }

    Ok(())
}
