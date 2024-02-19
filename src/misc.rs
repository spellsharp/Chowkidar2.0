use chrono::{Datelike, NaiveDate};
use serde_json::Value;
use std::fs::File;
use std::io::{self, Read};
use std::marker::{Send, Sync};

pub type Error = Box<dyn std::error::Error + Send + Sync>;

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
fn read_json_file(data_path: &str) -> Result<String, Error> {
    let mut file = File::open(data_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn extract_array_from_json(json_data: &Value, key: &str) -> Result<Vec<Value>, Error> {
    match json_data[key].as_array() {
        Some(array) => Ok(array.to_vec()),
        None => Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid JSON format: {} is missing or not an array", key),
        ))),
    }
}

fn process_json_data(contents: &str) -> Result<(Vec<Value>, Vec<Value>), Error> {
    let json_data: Value = serde_json::from_str(contents)?;
    let did_not_send = extract_array_from_json(&json_data, "memberDidNotSend")?;
    let did_send = extract_array_from_json(&json_data, "memberDidSend")?;
    Ok((did_not_send, did_send))
}

fn days_to_time_period(days: i64) -> String {
    if days >= 365 {
        days_to_years(days)
    } else if days >= 30 {
        days_to_months(days)
    } else if days >= 7 {
        days_to_weeks(days)
    } else {
        format!("{}D", days)
    }
}

fn days_to_weeks(days: i64) -> String {
    let week = days / 7;
    format!("{}W+", week)
}

fn days_to_months(days: i64) -> String {
    let month = days / 30;
    format!("{}M+", month)
}

fn days_to_years(days: i64) -> String {
    let year = days / 365;
    format!("{}Y+", year)
}

fn generate_report_content(
    did_not_send: &[Value],
    did_send: &[Value],
    date: NaiveDate,
) -> (String, Vec<u64>) {
    let mut report = String::from("**DAILY REPORT**\n\n");
    let mut kicked_ids: Vec<u64> = Vec::new();
    let mut years: Vec<Vec<String>> = vec![Vec::new(); 4];
    let mut kick_index = 0;

    // Processing members who did not send
    report += "**Did Not Send :scream:**\n\n";
    for member in did_not_send {
        if let Some(last_status_update) = member["lastStatusUpdate"].as_str() {
            if let Ok(last_update_date) = NaiveDate::parse_from_str(last_status_update, "%Y-%m-%d")
            {
                let days_difference = date.signed_duration_since(last_update_date).num_days();
                let time_period_not_sent = days_to_time_period(days_difference);

                let admission_year = member["admissionYear"]
                    .as_str()
                    .unwrap()
                    .parse::<i32>()
                    .unwrap();

                if let Some(year) = years.get_mut((date.year() - admission_year) as usize) {
                    year.push(format!(
                        "{} - {} \n",
                        member["fullName"].as_str().unwrap(),
                        time_period_not_sent
                    ));
                }

                if days_difference >= 3 {
                    kick_index += 1;
                    let user_id_str = &member["userID"];
                    let user_id = user_id_str.as_str().unwrap().parse::<u64>().unwrap();
                    kicked_ids.push(user_id);
                }
            }
        }
    }

    // Generating report content for each year
    for (index, members) in years.iter().enumerate() {
        if !members.is_empty() {
            let title = match index {
                1 => "**First Year Batch**",
                2 => "**Second Year Batch**",
                3 => "**Third Year Batch**",
                4 => "**Fourth Year Batch**",
                _ => "",
            };
            report += &format!("{}\n", title);
            for (index, member) in members.iter().enumerate() {
                report += &format!("{}. {}\n", index + 1, member);
            }
        }
    }

    // Generating streaks
    report += "**Streaks! :fire:**\n";
    let mut streaks: Vec<(&str, &str)> = did_send
        .iter()
        .map(|member| {
            (
                member["fullName"].as_str().unwrap(),
                member["streak"].as_str().unwrap(),
            )
        })
        .collect();
    streaks.sort_by(|a, b| {
        b.1.parse::<i32>()
            .unwrap()
            .cmp(&a.1.parse::<i32>().unwrap())
    });
    streaks.truncate(5);
    let mut i = 1;
    for (name, streak) in streaks {
        report += &format!("{}. {} - {}\n", i, name, streak);
        i += 1;
    }

    (report, kicked_ids)
}

pub fn compile_report(mock_data_path: &str) -> Result<(String, Vec<u64>), Error> {
    let contents = read_json_file(mock_data_path)?;
    let (did_not_send, did_send) = process_json_data(&contents)?;
    let date = get_date();
    let (report_content, kicked_ids) = generate_report_content(&did_not_send, &did_send, date);
    let kicked = if !kicked_ids.is_empty() {
        format!(
            "**Kicked :x: **\n{}",
            kicked_ids
                .iter()
                .enumerate()
                .fold(String::new(), |mut acc, (index, id)| {
                    let member = did_not_send
                        .iter()
                        .find(|m| m["userID"].as_str() == Some(id.to_string().as_str()));
                    if let Some(member) = member {
                        let full_name = member["fullName"].as_str().unwrap();
                        acc.push_str(&format!("{}. {}\n", index + 1, full_name));
                    }
                    acc
                })
        )
    } else {
        String::from("No one was kicked today!")
    };
    println!("{}", kicked);
    Ok((format!("{}\n{}", report_content, kicked), kicked_ids))
}
