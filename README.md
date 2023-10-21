Very WIP Rust driver for a Raspberry Pi Pico driving these [weird Adafruit LED matrix displays](https://www.adafruit.com/product/2278). Mostly reverse-engineered the protocol for the matrix which is why the code is so janky. I also wanted to write a WiFi driver for the Pico W but didn't end up actually finishing that.

This can display colors though! And I made a cool DVD logo animation:

![GIF of bouncing color-changing DVD logo animation across 2 Adafruit LED matrices](https://doggo.ninja/EmdJPK.gif)

You might find some of this code useful if you're looking to do something similar, especially around figuring out the display protocol!
