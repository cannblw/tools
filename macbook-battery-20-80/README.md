# Macbook Battery 20%-80%

## Introduction
Batteries usually maximise their lifespan if they are kept within the 20%-80% charge level.

This tools aims to help making that task easier!

- If your battery level is <= 20%, send a notification, letting you know that you should charge your laptop.
- If your battery level is >= 80%, send a notification, letting you know that you should unplug your laptop.

## Scope

This only works for MacOS because the script invokes terminal commands.

## How to use

- Create a release build
```bash
cargo build --release
```

- Move the release build to some other directory, for example ~/bin

- Update `com.cannblw.macbookbatterychecker.plist` to have the paths you want to use. Always use absolute paths.

- Copy `com.cannblw.macbookbatterychecker.plist` to `~/Library/LaunchAgents/`

- Load the LaunchD manifest:
```bash
launchctl load ~/Library/LaunchAgents/com.cannblw.macbookbatterychecker.plist
```

- Start the program using LaunchD
```bash
launchctl start com.cannblw.macbookbatterychecker
```

- Verify that it's running status
```bash
launchctl list | grep com.cannblw.macbookbatterychecker
```

## How to uninstall
```bash
launchctl stop com.cannblw.macbookbatterychecker
launchctl unload ~/Library/LaunchAgents/com.cannblw.macbookbatterychecker.plist
```

## Future improvements

If the laptop has just finished charging and you unplug it, the battery check will still run every 1 minute even if you're allowing it to discharge.
An improvement would be to do something like:

- If battery is close to 80% and laptop not plugged in => Don't check in a long time

For 20%, we should:

- If battery is close to 20% and laptop is plugged in => Don't check in a long time
