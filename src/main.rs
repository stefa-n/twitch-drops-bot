use thirtyfour::prelude::*;
use thirtyfour::common::capabilities::firefox::FirefoxPreferences;
use tokio::time::{Duration, timeout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::env;
use ctrlc;


async fn init_driver(args: Vec<String>) -> thirtyfour::WebDriver
{
    let mut caps = DesiredCapabilities::firefox();
    let mut pref = FirefoxPreferences::new();

    let debug = args.len() > 2;

    pref.set("media.hardware-video-decoding.enabled", false).unwrap();
    pref.set("dom.maxHardwareConcurrency", "1").unwrap();
    pref.set("dom.ipc.processCount", "2").unwrap();
    pref.set("dom.ipc.processCount.file", "0").unwrap();
    pref.set("dom.ipc.processCount.privilegedabout", "0").unwrap();
    pref.set("dom.ipc.processCount.privilegedmozilla", "0").unwrap();
    pref.set("dom.ipc.processCount.extension", "0").unwrap();
    pref.set("dom.ipc.processCount.webIsolated", "2").unwrap();

    if !debug
    {
        pref.set("media.volume_scale", "0.0").unwrap();
        caps.set_headless().unwrap();
    }
    caps.set_preferences(pref).unwrap();

    println!("Preferences set up!");

    let driver = WebDriver::new("http://localhost:4444", caps).await.unwrap();
    
    if !debug
    {
        // it wont let me do it with pref.set() so i have to make it like i had a brain injury
        driver.goto("about:config").await.unwrap();
        driver.find(By::Id("warningButton")).await.unwrap().click().await.unwrap();
        driver.find(By::Id("about-config-search")).await.unwrap().send_keys("layout.frame_rate").await.unwrap();
        //  thank you firefox
        tokio::time::sleep(Duration::from_secs(1)).await;
        driver.find(By::Css("button[data-l10n-id=\"about-config-pref-edit-button\"")).await.unwrap().click().await.unwrap();
        driver.find(By::Css("input[type=\"number\"]")).await.unwrap().clear().await.unwrap();
        driver.find(By::Css("input[type=\"number\"]")).await.unwrap().send_keys("1").await.unwrap();
        driver.find(By::ClassName("button-save")).await.unwrap().click().await.unwrap();
        // i actually have no idea if limiting the frame rate like i did above helps at all but i did it anyway

        println!("Set frame rate to 1!");
    }

    driver.goto("https://twitch.tv/").await.unwrap();
    if args.len() < 2
    {
        println!("Usage: twitchdrops <auth-token> <channel>");
        driver.quit().await.unwrap();
        std::process::abort();
    }

    let mut cookie = Cookie::new("auth-token", format!("{}", &args[1]));
    cookie.set_domain("twitch.tv");
    cookie.set_path("/");
    driver.add_cookie(cookie.clone()).await.unwrap();

    driver.goto(format!("https://twitch.tv/{}", &args[2])).await.unwrap();

    return driver;
}

async fn load_twitch_stream(driver: thirtyfour::WebDriver)
{
    let exit_requested = Arc::new(AtomicBool::new(false));
    let exit_requested_clone = exit_requested.clone();
    ctrlc::set_handler(move || {
        exit_requested_clone.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    println!("Waiting for content classification button");
    let result = timeout(Duration::from_secs(10), async {
        loop {
            match driver.find(By::Css("button[data-a-target=\"content-classification-gate-overlay-start-watching-button\"]")).await {
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
            println!("Content classification button found!");
        }
        Err(_) => {
            eprintln!("Element not found within the timeout.");
        }
    }

    let result = timeout(Duration::from_secs(3), async {
        loop {
            match driver.find(By::Css("button[data-a-target=\"player-settings-button\"]")).await {
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
            println!("Player settings button found!");
        }
        Err(_) => {
            eprintln!("Element not found within the timeout.");
        }
    }

    let result = timeout(Duration::from_secs(3), async {
        loop {
            match driver.find(By::Css("button[data-a-target=\"player-settings-menu-item-quality\"]")).await {
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
            println!("Quality button found!");
        }
        Err(_) => {
            eprintln!("Element not found within the timeout.");
        }
    }

    let result = timeout(Duration::from_secs(3), async {
        let mut last_matching_element = None;

        loop {
            let buttons = driver.find_all(By::Css("div[class=\"Layout-sc-1xcs6mc-0 ScRadioLayout-sc-1pxozg3-1 gyuRLA kGFJBn tw-radio\"]")).await;

            match buttons {
                Ok(buttons) => {
                    if buttons.is_empty() {
                        break;
                    }

                    last_matching_element = Some(buttons.last().unwrap().clone());

                    if let Err(_) = last_matching_element.clone().unwrap().click().await {
                        eprintln!("Failed to click the element.");
                    }
                }
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }

        last_matching_element
    })
    .await;

    match result {
        Ok(_) => {
            println!("Set quality to lowest.");
        }
        Err(_) => {
            eprintln!("Element not found within the timeout.");
        }
    }

    let result = timeout(Duration::from_secs(30), async {
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

    
    let original_tab = driver.window().await.unwrap();
    let new_tab = driver.new_tab().await.unwrap();

    while !exit_requested.load(Ordering::SeqCst) {
        match driver.find(By::Css("p.CoreText-sc-1txzju1-0.bcMydc")).await {
            Ok(text_element) => {
                let text = text_element.text().await.unwrap();
                print!("\rDrops: {}", text);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                driver.switch_to_window(new_tab.to_owned()).await.unwrap();
                tokio::time::sleep(Duration::from_secs(28)).await;
            }
            Err(_) => {
                driver.switch_to_window(original_tab.to_owned()).await.unwrap();
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

#[tokio::main]
async fn main() {
    let args : Vec<String> = env::args().collect();
    let driver = init_driver(args).await;

    println!("The progress will update every 30 seconds.");
    load_twitch_stream(driver.to_owned()).await;

    driver.quit().await.unwrap();
}
