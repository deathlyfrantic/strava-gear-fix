## strava-gear-fix

Strava only has the concept of a single default piece of gear that applies to
all activities (or at least all bike rides). But I have a dedicated trainer bike
and I got tired of manually changing my bike on Virtual Rides. I wrote this
little script so I wouldn't have to any more. If Strava had per-activity-type
defaults (i.e. if I could set a bike as the default for Virtual Rides only) for
gear I wouldn't need this, but they don't, so I do.

This script is very specific and unlikely to be useful to you, but if you
have a similar need maybe it can serve as a starting point.

### Requirements

[Deno](https://deno.land)

### Setup

[Create an app](https://developers.strava.com/docs/getting-started/#account) on
Strava and get your client id and client secret. Copy `data.example.json` to
`data.json` in the same directory as this project and fill in the client id and
secret in that file. Once that is done, you can use the OAuth flow to create
tokens:

    deno --allow-all auth.ts

This will start a server on `localhost:8000` and open the Strava authorization
page in your browser (by calling the macOS-specific `open` - change this to
something else if you're on another platform). Once you approve, the OAuth dance
will commence and you'll end up with an access token and refresh token added to
your `data.json` file. From there:

    deno --allow-all main.ts

Will do the (very specific) thing of setting the bike to the trainer bike id for
new Virtual Rides. (Make sure you add a `trainerBikeId` entry to `data.json` for
this to work.) This will save the time of the latest activity found to
`data.json` so it will only look for new activities each time it is run.

Note you can enable debug logging by including `DEBUG=true` (or any non-empty
string) to the above commands.

### License

BSD 2-clause
