use poise::serenity_prelude as serenity;

mod commands;
mod config;
use commands::cmds;
use commands::utils::structs::Data;

//type Context<'a> = poise::Context<'a, Data, Error>;
type Error = Box<dyn std::error::Error + Send + Sync>;

async fn event_handlers(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _user_data: &Data,
) -> Result<(), Error> {
    match event {
        poise::Event::Ready { data_about_bot } => {
            poise::builtins::register_globally(
                &ctx.http,
                &cmds::all()
            )
            .await?;
            println!("Logged in: {}", data_about_bot.user.tag());
        }
        _ => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let options = poise::FrameworkOptions {
        commands: cmds::all(),
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("mom ".into()),
            ..Default::default()
        },
        event_handler: |ctx, event, framework, user_data| {
            Box::pin(event_handlers(ctx, event, framework, user_data))
        },
        ..Default::default()
    };

    poise::Framework::builder()
        .token(config::DISCORD_TOKEN)
        .options(options)
        .setup(|_ctx, _data_about_bot, _framework| {
            Box::pin(async move {
                Ok(Data {
                    ..Default::default()
                })
            })
        })
        .intents(serenity::GatewayIntents::all())
        .run()
        .await
        .unwrap()
}
