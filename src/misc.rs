use chrono::{Datelike, NaiveDate};
use serde_json::Value;
use std::fs::File;
use std::io::Read;
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

fn days_to_weeks(days: i64) -> String {
    let week = days / 7;
    return format!("{}W+", week);
}

fn days_to_months(days: i64) -> String {
    let month = days / 30;
    return format!("{}M+", month);
}

fn days_to_years(days: i64) -> String {
    let year = days / 365;
    return format!("{}Y+", year);
}

pub fn process_json(
    data_path: &str,
) -> Result<(Vec<Value>, Vec<Value>), Error> {
    let mut file = File::open(data_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let json_data: Value = match serde_json::from_str(&contents) {
        Ok(data) => data,
        Err(err) => return Err(Box::new(err)),
    };

    let did_not_send = match json_data["memberDidNotSend"].as_array() {
        Some(array) => array.to_vec(),
        None => return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid JSON format: memberDidNotSend is missing or not an array",
        ))),
    };

    let did_send = match json_data["memberDidSend"].as_array() {
        Some(array) => array.to_vec(),
        None => return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid JSON format: memberDidSend is missing or not an array",
        ))),
    };

    Ok((did_not_send, did_send))
}

pub fn compile_report(mock_data_path: &str) -> Result<(String, Vec<u64>), Error> {
    
    let (did_not_send, did_send) = process_json(mock_data_path)?;


    let mut report = String::from("**DAILY REPORT**\n\n");
    let mut kicked_ids: Vec<u64> = Vec::new();

    let mut kicked = String::new();
    let date: NaiveDate = get_date();

    let mut first_years: Vec<String> = Vec::new();
    let mut second_years: Vec<String> = Vec::new();
    let mut third_years: Vec<String> = Vec::new();
    let mut fourth_years: Vec<String> = Vec::new();

    let mut kick_index = 0;
    report += &format!("**Did Not Send :scream:**\n\n");
    for member in did_not_send {
        if let Some(last_status_update) = member["lastStatusUpdate"].as_str() {
            if let Ok(last_update_date) = NaiveDate::parse_from_str(last_status_update, "%Y-%m-%d")
            {
                let days_difference = date.signed_duration_since(last_update_date).num_days();

                let mut time_period_not_sent = format!("{}D", days_difference);

                if days_difference > 365 {
                    time_period_not_sent = days_to_years(days_difference);
                } else if days_difference > 30 {
                    time_period_not_sent = days_to_months(days_difference);
                } else if days_difference > 7 {
                    time_period_not_sent = days_to_weeks(days_difference);
                }

                let admission_year = member["admissionYear"]
                    .as_str()
                    .unwrap()
                    .parse::<i32>()
                    .unwrap();

                if date.year() - admission_year == 1 {
                    first_years.push(format!(
                        "{} - {} \n",
                        member["fullName"].as_str().unwrap(),
                        time_period_not_sent
                    ));
                }

                if date.year() - admission_year == 2 {
                    second_years.push(format!(
                        "{} - {} \n",
                        member["fullName"].as_str().unwrap(),
                        time_period_not_sent
                    ));
                }

                if date.year() - admission_year == 3 {
                    third_years.push(format!(
                        "{} - {} \n",
                        member["fullName"].as_str().unwrap(),
                        time_period_not_sent
                    ));
                }

                if date.year() - admission_year == 4 {
                    fourth_years.push(format!(
                        "{} - {} \n",
                        member["fullName"].as_str().unwrap(),
                        time_period_not_sent
                    ));   
                }

                if days_difference >= 3 {
                    kick_index += 1;
                    let user_id_str = &member["userID"];
                    let user_id = user_id_str.as_str().unwrap().parse::<u64>().unwrap();
                    kicked_ids.push(user_id); // Add the kicked user's ID to the vector
                    kicked += &format!("{}. {}\n", kick_index, member["fullName"].as_str().unwrap());
                }
            }
        }
    }
    if !first_years.is_empty() {
        report += &format!("First Years\n");
        for (index, member) in first_years.iter().enumerate() {
            report += &format!("{}. {}\n", index + 1, member);
        }
    }

    if !second_years.is_empty() {
        report += &format!("Second Years\n");
        for (index, member) in second_years.iter().enumerate() {
            report += &format!("{}. {}\n", index + 1, member);
        }
    }

    if !third_years.is_empty() {
        report += &format!("Third Years\n");
        for (index, member) in third_years.iter().enumerate() {
            report += &format!("{}. {}\n", index + 1, member);
        }
    }

    if !fourth_years.is_empty() {
        report += &format!("Fourth Years\n");
        for (index, member) in fourth_years.iter().enumerate() {
            report += &format!("{}. {}\n", index + 1, member);
        }
    }

    report += &format!("**Streaks! :fire:**\n");

    // Get members with top 5 streak values
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

    // Add list of people who got kicked to the report.
    if !kicked.is_empty() {
        kicked = String::from("**Kicked :x: **\n") + &kicked;
    } else {
        kicked = String::from("No one was kicked today!");
    }
    println!("{}", kicked);

    report += &format!("\n{}", kicked);
    Ok((report, kicked_ids))
}