# Deprecated

Strava now supports [per-activity-type default
gear](https://communityhub.strava.com/t5/what-s-new/new-set-default-gear-by-sport/m-p/9728)
so this program is no longer necessary.

## strava-gear-fix

Strava only has the concept of a single default piece of gear that applies to
all activities (or at least all bike rides). But I have a dedicated trainer bike
and I got tired of manually changing my bike on Virtual Rides. I wrote this so I
wouldn't have to do that any more. If Strava had per-activity-type defaults for
gear (i.e. if I could set a bike as the default for Virtual Rides only), I
wouldn't need this, but they don't, so I do.

This program is very specific and unlikely to be useful to you, but if you
have a similar need maybe it can serve as a starting point.

### Requirements

[Rust](https://www.rust-lang.org)

### Setup

[Create an app](https://developers.strava.com/docs/getting-started/#account) on
Strava and get your client id and client secret. Copy `data.example.json` to
`data.json` in the same directory as this project and fill in the client id and
secret and trainer bike id in that file. Once that is done, you can use the
OAuth flow to create tokens:

    cargo run --bin auth

This will start a server on `localhost:8000` and open the Strava authorization
page in your browser (by calling the macOS-specific `open` - change this to
something else if you're on another platform). Once you approve, the OAuth dance
will commence and you'll end up with an access token and refresh token added to
your `data.json` file. From there, running

    cargo run

will do the (very specific) thing of setting the bike to the trainer bike id for
new Virtual Rides. This will save the time of the latest activity found to
`data.json` so it will only look for new activities each time it is run.

Note you can change the level of logs output by using the `RUST_LOG` environment
variable, e.g. `RUST_LOG=(trace|debug|info|warn|error) cargo run`.

### License

BSD 2-clause
