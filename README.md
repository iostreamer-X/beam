<p align="center">
	<img src="/demo.png" width="200px"/>
</p>
<p align="center">
	<a href="https://iostreamer.me">Demo</a>
</p>

# An experiment in streaming
This is a small tool I run locally to broadcast what media I am consuming.
Fancy way to show exactly how am I wasting time on youtube.

## How does it work?
It interfaces with macOS's [DistributedNotificationCenter](https://developer.apple.com/documentation/foundation/distributednotificationcenter)
and runs a separate poller which gets data using [nowplaying-cli](https://github.com/kirtan-shah/nowplaying-cli)

This lets me broadcast more than just apple music events. If I am watching something on browser that also gets picked up.
So if I am spending too much time watching [r/Art](https://www.reddit.com/r/Art/comments/udb3p8/uluru_blackhle_rise_me_pixel_art_2022/) you
will see it on my blog!

## How does it stream?
On an event, we insert a record in a supabase backed postgres db. After that, supabase [realtime api](https://supabase.com/docs/guides/realtime) setup kicks
in and the browser gets the event through sockets.
