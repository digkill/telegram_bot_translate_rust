# Telegram Translation Bot

![Bot Logo](./assets/images/cat-maid-cat-maid-translation.gif)

Welcome to the Telegram Translation Bot project! This bot helps you translate messages in real-time within your Telegram chats. It's designed to be easy to use and supports multiple languages.

## Features

- **Real-time Translation**: Translates messages as you send them.
- **Multiple Languages**: Supports over 100 languages.
- **Easy Setup**: Simple and straightforward installation process.

## Install

```Bash
cp .env.cample .env

cargo run
```

All that remains is to build the application and run it in daemon mode on the server

1. Build your application using the command:
   `cargo build --release`

2. Create a unit file for systemd. For example, let's create a file
   `/etc/systemd/system/translate_bot.service`

```Bash
[Unit]
Description=Translate Bot
After=network.target

[Service]
ExecStart=/path/to/your/application
Restart=always
User=yourusername
Group=yourgroupname
Environment=/path/to/your/.env
WorkingDirectory=/path/to/your/application/directory

[Install]
WantedBy=multi-user.target
```

4. Reload your systemd configuration to let it know about the new service:
   `sudo systemctl daemon-reload`

5. Start the service:
   `sudo systemctl start translate_bot`

6. Make sure the service is up and running:
   `sudo systemctl status translate_bot`

7.To have your service automatically start when the system starts, run the command:
`sudo systemctl enable rust_service`

Your Rust application will run as a system service managed by systemd. You can control it using standard systemd commands such as start, stop, restart, and status.

### Resolved problem with Ubuntu:
```Bash
export OPENSSL_DIR=/usr
export OPENSSL_INCLUDE_DIR=/usr/include/openssl
export OPENSSL_LIB_DIR='/usr/lib/x86_64-linux-gnu'
cargo build --release
```