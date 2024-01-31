mod errors;

use chrono::NaiveDate;
use errors::Error;
use poise::serenity_prelude as serenity;
use serde_json::Value;
use shuttle_poise::ShuttlePoise;
use shuttle_secrets::SecretStore;
use std::fs::File;
use std::io::Read;

struct Data {
    secret_store: SecretStore,
}

type Context<'a> = poise::Context<'a, Data, Error>;

fn get_date() -> NaiveDate {
    let date = format!(
        "{}",
        chrono::Local::now()
            .with_timezone(&chrono_tz::Asia::Kolkata)
            .format("%Y-%m-%d")
    );

    let date = NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d").unwrap();

    return date;
}

#[poise::command(slash_command)]
async fn send_report(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    // TODO: Uncomment this when need to actually kick.
    // let guild_id = ctx.guild_id().unwrap();

    // Making the reply message
    let mut message = String::from("**DAILY REPORT**\n\n");
    let mut kicked = String::new();

    // Using mock data from mock_data.json
    let data_path = ctx
        .data()
        .secret_store
        .get("MOCK_DATA_PATH")
        .expect("Member data path must be set.");
    let mut file = File::open(data_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let json_data: Value = serde_json::from_str(&contents)?;
    let did_not_send = json_data["memberDidNotSend"].as_array().unwrap();
    let did_send = json_data["memberDidSend"].as_array().unwrap();

    let date: NaiveDate = get_date();

    message += &format!("**Did Not Send :scream:**\n");

    for member in did_not_send {
        if let Some(last_status_update) = member["lastStatusUpdate"].as_str() {
            if let Ok(last_update_date) = NaiveDate::parse_from_str(last_status_update, "%Y-%m-%d")
            {
                let days_difference = date.signed_duration_since(last_update_date).num_days();

                message += &format!(
                    "{} - {}D \n",
                    member["fullName"].as_str().unwrap(),
                    days_difference
                );

                if days_difference >= 3 {
                    
                    // TODO: Uncomment this when need to actually kick.
                    // let user_id_str = &member["userID"];
                    // let user_id = user_id_str.as_str().unwrap().parse::<u64>().unwrap();
                    // guild_id
                    //     .kick(ctx.serenity_context().http.clone(), user_id)
                    //     .await?;

                    kicked += &format!("{}\n", member["fullName"].as_str().unwrap());
                }
            }
        }
    }
    message += &format!("\n**Streaks! :fire:**\n");

    // Get the list of top 5 member["streak"] values
    let mut streaks: Vec<&str> = did_send
        .iter()
        .map(|member| member["streak"].as_str().unwrap())
        .collect();
    streaks.sort_by(|a, b| b.parse::<i32>().unwrap().cmp(&a.parse::<i32>().unwrap()));
    streaks.truncate(5);

    // Print members with top 5 streak values
    for member in did_send {
        if streaks.contains(&member["streak"].as_str().unwrap()) {
            message += &format!(
                "{} - {}\n",
                member["fullName"].as_str().unwrap(),
                member["streak"].as_str().unwrap()
            );
        }
    }

    // TODO: Fix this!
    if !kicked.is_empty() {
        kicked = String::from("**Kicked :x: **\n") + &kicked;
    } else {
        kicked = String::from("No one was kicked today!");
    }

    message += &format!("\n{}", kicked);

    ctx.reply(message).await?;

    return Ok(());
}

#[shuttle_runtime::main]
async fn main(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttlePoise<Data, Error> {
    let framework_options = poise::FrameworkOptions {
        commands: vec![send_report()],
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

    Ok(framework.into())
}
