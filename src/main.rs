mod misc;

use job_scheduler::{Job, JobScheduler};
use poise::serenity_prelude as serenity;
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;
use std::thread;
use std::time::Duration;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

struct Data {
    secret_store: SecretStore,
}

type Context<'a> = poise::Context<'a, Data, Error>;


async fn send_report<'a>(ctx: &'a Context<'a>) -> Result<(), Error> {
    ctx.defer().await?;

    let mock_data_path = ctx
        .data()
        .secret_store
        .get("MOCK_DATA_PATH")
        .ok_or("Failed to get mock data path")?;

    let (report, kicked_ids) = misc::compile_report(&mock_data_path)?;

    // TODO: Uncomment this when need to actually kick.

    // let guild_id = ctx.guild_id().ok_or("Failed to get guild ID")?;
    // for user_id in kicked_ids {
    //     guild_id
    //         .kick(ctx.serenity_context().http.clone(), user_id)
    //         .await?;
    // }

    ctx.reply(report).await?;

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

    let framework = poise::Framework::builder()
        .options(framework_options)
        .token(token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { secret_store })
            })
        })
        .build()
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    thread::spawn(move || {
        let mut sched = JobScheduler::new();

        // TODO: Change this to 5 am everyday instead of every 10 seconds.
        sched.add(Job::new("1/10 * * * * *".parse().unwrap(), move || {
            // TODO: Fix this.
            // send_report(); 
            println!("Sending report!");
        }));

        loop {
            sched.tick();
            std::thread::sleep(Duration::from_millis(500));
        }
    });

    Ok(framework.into())
}
