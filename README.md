<p align="center">
	<img src="/demo.png"/>
</p>

# An experiment in streaming
This is a small tool I run locally to broadcast what media I am consuming.
Fancy way to show how I exactly am I wasting time on youtube.

## How does it work?
It interfaces with macOS's [DistributedNotificationCenter](https://developer.apple.com/documentation/foundation/distributednotificationcenter)
and runs a separate poller which gets data using [nowplaying-cli](https://github.com/kirtan-shah/nowplaying-cli)

## How does it stream?
On an event, we insert a record in a supabase backed postgres db. After that, supabase [realtime api](https://supabase.com/docs/guides/realtime) setup kicks
in and the browser gets the event through sockets.
