use thirtyfour::prelude::*;
use thirtyfour::common::capabilities::firefox::FirefoxPreferences;
use tokio::time::{Duration, timeout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::env;
use ctrlc;

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let exit_requested = Arc::new(AtomicBool::new(false));
    let exit_requested_clone = exit_requested.clone();

    ctrlc::set_handler(move || {
        exit_requested_clone.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    let mut caps = DesiredCapabilities::firefox();
    let mut pref = FirefoxPreferences::new();

    pref.set("media.volume_scale", "0.0")?;
    pref.set("media.hardware-video-decoding.enabled", false)?;
    pref.set("dom.maxHardwareConcurrency", "1")?;
    pref.set("dom.ipc.processCount", "2")?;
    pref.set("dom.ipc.processCount.file", "0")?;
    pref.set("dom.ipc.processCount.privilegedabout", "0")?;
    pref.set("dom.ipc.processCount.privilegedmozilla", "0")?;
    pref.set("dom.ipc.processCount.extension", "0")?;
    pref.set("dom.ipc.processCount.webIsolated", "2")?;

    caps.set_headless()?;
    caps.set_preferences(pref)?;

    let driver = WebDriver::new("http://localhost:4444", caps).await?;

    // it wont let me do it with pref.set() so i have to make it like i had a brain injury
    driver.goto("about:config").await?;
    driver.find(By::Id("warningButton")).await?.click().await?;
    driver.find(By::Id("about-config-search")).await?.send_keys("layout.frame_rate").await?;
    //  thank you firefox
    tokio::time::sleep(Duration::from_secs(1)).await;
    driver.find(By::Css("button[data-l10n-id=\"about-config-pref-edit-button\"")).await?.click().await?;
    driver.find(By::Css("input[type=\"number\"]")).await?.clear().await?;
    driver.find(By::Css("input[type=\"number\"]")).await?.send_keys("1").await?;
    driver.find(By::ClassName("button-save")).await?.click().await?;
    // i actually have no idea if limiting the frame rate like i did above helps at all but i did it anyway

    driver.goto("https://twitch.tv/").await?;
    let args : Vec<String> = env::args().collect();
    if args.len() < 2
    {
        println!("Usage: twitchdrops <auth-token> <channel>");
        driver.quit().await?;
        return Ok(());
    }

    let mut cookie = Cookie::new("auth-token", format!("{}", &args[1]));
    cookie.set_domain("twitch.tv");
    cookie.set_path("/");
    driver.add_cookie(cookie.clone()).await?;

    driver.goto(format!("https://twitch.tv/{}", &args[2])).await?;

    let timeout_duration = Duration::from_secs(30);

    let result = timeout(timeout_duration, async {
        loop {
            match driver.find(By::Css("button[data-a-target=\"user-menu-toggle\"]")).await {
                Ok(button) => {
                    button.click().await.unwrap();
                    break;
                }
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    })
    .await;

    match result {
        Ok(_) => {
            println!("User menu button found!");
        }
        Err(_) => {
            eprintln!("Element not found within the timeout.");
        }
    }

    let original_tab = driver.window().await?;
    let new_tab = driver.new_tab().await?;

    println!("The progress will update every 30 seconds.");

    while !exit_requested.load(Ordering::SeqCst) {
        match driver.find(By::Css("p.CoreText-sc-1txzju1-0.bcMydc")).await {
            Ok(text_element) => {
                let text = text_element.text().await.unwrap();
                print!("\rDrops: {}", text);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                driver.switch_to_window(new_tab.to_owned()).await?;
                tokio::time::sleep(Duration::from_secs(28)).await;
            }
            Err(_) => {
                driver.switch_to_window(original_tab.to_owned()).await?;
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    driver.quit().await?;

    Ok(())
}
