mod misc;

use job_scheduler::{Job, JobScheduler};
use poise::serenity_prelude::{self as serenity, ChannelId};
use serenity::{
    http::Http,
    model::id::{GuildId, UserId},
};
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;
use std::{sync::Arc, thread, time::Duration};
use tokio::runtime::Runtime;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

struct Data {
    secret_store: SecretStore,
}

async fn probate_member(http: &Http, guild_id: GuildId, user_id: UserId) -> Result<(), Error> {
    let mut member = guild_id.member(http, user_id).await?;

    // Remove all existing roles
    member.remove_roles(http, &[]).await?;

    // Add the "Probation" role
    let guild = guild_id.to_partial_guild(&http).await?;
    let probation_role = guild
        .roles
        .values()
        .find(|role| role.name == "Probation")
        .ok_or("Role 'Probation' not found")?;
    member.add_role(http, probation_role).await?;

    println!("Probating user: {}", user_id);
    Ok(())
}

async fn send_report(
    secret_store: SecretStore,
    http: Arc<Http>,
    channel_id: ChannelId,
    guild_id: GuildId,
) -> Result<(), Error> {
    let mock_data_path = secret_store
        .get("MOCK_DATA_PATH")
        .ok_or("Failed to get mock data path")?;

    let (report, kicked_ids) = misc::compile_report(&mock_data_path)?;

    // TODO: Uncomment this when need to actually kick.
    for user_id_u64 in kicked_ids {
        let user_id = UserId(user_id_u64);
        match probate_member(&http, guild_id, user_id).await {
            Ok(_) => println!("Probated user: {}", user_id),
            Err(why) => println!("Error kicking user: {:?}", why),
        }
    }

    match channel_id.say(&http, report).await {
        Ok(_) => println!("Message sent"),
        Err(why) => println!("Error sending message: {:?}", why),
    }
    println!("Reached");
    Ok(())
}

#[shuttle_runtime::main]
async fn main(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttlePoise<Data, Error> {
    let framework_options = poise::FrameworkOptions {
        commands: vec![],
        ..Default::default()
    };

    let token = secret_store
        .get("DISCORD_TOKEN")
        .expect("Discord Token must be set");

    let token_clone = token.clone();

    let secret_store_clone = secret_store.clone();
    let channel_id = ChannelId(
        secret_store
            .get("CHANNEL_ID")
            .expect("Channel ID must be set")
            .parse()
            .expect("Invalid Channel ID"),
    );

    let guild_id = GuildId(
        secret_store
            .get("GUILD_ID")
            .expect("Guild ID must be set")
            .parse()
            .expect("Invalid Guild ID"),
    );

    let framework = poise::Framework::builder()
        .options(framework_options)
        .token(token_clone)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    secret_store: secret_store_clone,
                })
            })
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    thread::spawn(move || {
        let mut sched = JobScheduler::new();
        let http = Arc::new(Http::new(&token));
        let rt = Runtime::new().unwrap();

        // TODO: Change the expression to "0 0 5 * * *" for calling the function everyday at 5 AM.
        sched.add(Job::new("* * * * * *".parse().unwrap(), move || {
            let secret_store = secret_store.clone();
            let http = http.clone();
            rt.block_on(async move {
                let _ = send_report(secret_store, http, channel_id, guild_id).await;
            });
        }));
        loop {
            sched.tick();
            std::thread::sleep(Duration::from_millis(500));
        }
    });

    Ok(framework.into())
}
