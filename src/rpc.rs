use std::error::Error;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

// Thank you very much sardonicism-04 for the bulk of this code

pub(crate) fn rpc(file_name: String) -> Result<(), Box<dyn Error>> {
    let mut client = DiscordIpcClient::new("1052633997638905996")?;
    client.connect()?;

    client.set_activity(
        activity::Activity::new()
            .details("Currently Editing")
            .state(&*file_name)
            .assets(
                activity::Assets::new()
                    .large_image("pot"),
            )
            .buttons(vec![activity::Button::new(
                "Homepage",
                "https://github.com/bananaturtlesandwich/stove"
            )]),
    )?;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(99999999));
    }

    // client.close()?;
    // Ok(())
}