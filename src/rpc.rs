use std::error::Error;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};


pub(crate) fn rpc(file_name : String) -> Result<(), Box<dyn Error>> {
    let mut client = DiscordIpcClient::new("1052633997638905996")?;
    client.connect()?;

    client.set_activity(
        activity::Activity::new()
            .details("Currently Editing")
            .state(&*file_name)
            .assets(
                activity::Assets::new()
                    .large_image("pot"),
            ),
    )?;
    std::thread::sleep(std::time::Duration::from_secs(99999999));

    client.set_activity(
        activity::Activity::new()
            .state("part 2 (test)")
            .details("a placeholder")
            .assets(
                activity::Assets::new()
                    .large_image("small-image")
                    .large_text("a thing"),
            ),
    )?;
    std::thread::sleep(std::time::Duration::from_secs(2));

    client.close()?;
    Ok(())
}