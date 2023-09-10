# twitch-drops-bot
Twitch Drops bot made in Rust (code may suck, I am still learning Rust); currently only works for Firefox

# Installation
Either compile or download the .exe from the Releases page. No additional setup required!

# Usage
You need to set up [Firefox](https://www.mozilla.org/ro/firefox/new/) and [Geckodriver](https://github.com/mozilla/geckodriver/releases).

Run Geckodriver on port 4444 (it runs on that port by default) and then open a CMD in the directory you downloaded twitch-drops-bot.exe; in Windows 11, you can do that by right-clicking in File Explorer and selecting "Open in Terminal".

After you open the Terminal, run "twitch-drops-bot authkey channel-to-watch", where authkey is your authentification cookie and channel to watch is the channel to watch, which has an active Drops campaign.

Close the program by CTRL+C, NOT by closing the CMD window.

# Errors
## How do I get my Authentification Key?
Open Twitch in a web browser of your choice (prefferably Firefox) and open Dev Tools (CTRL+SHIFT+I or F12). In the Dev Tools, go to the Storage tab, select twitch.tv under the Cookies category and get the value of "auth-token".

## No connection could be made because the target machine actively refused it.
Make sure you have Geckodriver running on port 4444 (default port).

## Closing the program with CTRL+C takes a while
That is not intended, but a bug. It takes up to 30 seconds, just wait
