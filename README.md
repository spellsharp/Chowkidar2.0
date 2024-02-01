# Chowkidar2.0

Ch0wkidar2.0 is an improved discord bot built for kicking club memebers who haven't send status updates in three consecutive days. The new version has a feature that displays the top 5 streaks. 

# Prerequisities

Before running the bot, ensure you have the following set up:

- Rust: Make sure you have Rust installed on your system. You can install Rust by following the instructions on rustup.rs.
    - For deployment and local runs, you will also require an account on [shuttle.rs](https://www.shuttle.rs/).

- Discord Bot Token: Create a Discord bot on the Discord Developer Portal, and add the token to your Secrets.toml as "DISCORD_TOKEN".

The final Secrets.toml file should resemble:
```
DISCORD_TOKEN=""
```

# Usage

``` /send_report ```

# License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
