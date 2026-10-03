# Local hot-plug patch

Upstream Bevy 0.19.1, bevy_gilrs. Original dual Apache-2.0/MIT
licensing is retained. Bug 14: ignore disconnect/button/axis events whose
Gilrs ID has no entity mapping yet, instead of panicking. WGI discovery and
connection callbacks can race during hot-plug. Normal mapped events and
reconnection behavior are unchanged. Keep until upstream guards these paths.
