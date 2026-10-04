use hang::moq_net;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::time::Duration;

use crate::track_format::{Container, TrackFormat, WireContainer, audio_config, video_config};

enum Media {
    Video(moq_mux::container::Producer<WireContainer, hang::catalog::VideoConfig>),
    Audio(moq_mux::container::Producer<WireContainer, hang::catalog::AudioConfig>),
}

struct LiveTrack {
    container: Container,
    media: Media,
}

impl LiveTrack {
    fn set(&mut self, format: TrackFormat) -> Result<(), UpdateTrackError> {
        let container = self.container.into();
        match (&mut self.media, format) {
            (Media::Video(p), TrackFormat::Video(f)) => p.set(video_config(f, container)),
            (Media::Audio(p), TrackFormat::Audio(f)) => p.set(audio_config(f, container)),
            (_, _) => return Err(UpdateTrackError::KindMismatch),
        }
        .map_err(UpdateTrackError::Catalog)
    }

    fn write(&mut self, frame: moq_mux::container::Frame) -> Result<(), moq_mux::Error> {
        match &mut self.media {
            Media::Video(p) => p.write(frame),
            Media::Audio(p) => p.write(frame),
        }
    }

    fn finish(&mut self) -> Result<(), moq_mux::Error> {
        match &mut self.media {
            Media::Video(p) => p.finish(),
            Media::Audio(p) => p.finish(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum CreateError {
    #[error("create_broadcast({path}) failed: {source}")]
    Broadcast {
        path: String,
        source: moq_net::Error,
    },
    #[error("catalog::Producer::new failed: {0}")]
    Catalog(moq_net::Error),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum AddTrackError {
    #[error("track already exists")]
    AlreadyExists,
    #[error("create_track failed: {0}")]
    CreateTrack(moq_net::Error),
    #[error("media_producer failed: {0}")]
    MediaProducer(moq_mux::Error),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum UpdateTrackError {
    #[error("unknown track")]
    UnknownTrack,
    #[error("cannot change a track's media kind in place")]
    KindMismatch,
    #[error("catalog update failed: {0}")]
    Catalog(moq_mux::Error),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum WriteFrameError {
    #[error("unknown track")]
    UnknownTrack,
    #[error("missing keyframe")]
    MissingKeyframe,
    #[error("writing frame failed: {0}")]
    Write(moq_mux::Error),
}

pub(crate) struct Producer {
    broadcast: moq_net::broadcast::Producer,
    catalog: moq_mux::catalog::Producer,
    tracks: HashMap<String, LiveTrack>,
}

impl Producer {
    pub(crate) fn new(session: &crate::session::Handle, path: String) -> Result<Self, CreateError> {
        let mut broadcast = session
            .publish
            .publish(&path, moq_net::origin::Route::default())
            .map_err(|source| CreateError::Broadcast { path, source })?;

        let catalog =
            moq_mux::catalog::Producer::new(&mut broadcast, moq_mux::catalog::Config::default())
                .map_err(CreateError::Catalog)?;

        Ok(Self {
            broadcast,
            catalog,
            tracks: HashMap::new(),
        })
    }

    pub(crate) fn add_track(
        &mut self,
        track: String,
        format: TrackFormat,
        container: Container,
        priority: u8,
        latency: Duration,
    ) -> Result<(), AddTrackError> {
        let entry = match self.tracks.entry(track) {
            Entry::Occupied(_) => return Err(AddTrackError::AlreadyExists),
            Entry::Vacant(entry) => entry,
        };

        let track = self
            .broadcast
            .create_track(
                entry.key().as_str(),
                moq_net::track::Info::default().with_priority(priority),
            )
            .map_err(AddTrackError::CreateTrack)?;

        let media = match format {
            TrackFormat::Video(format) => {
                let config = video_config(format, container.into());
                let wire =
                    WireContainer::try_from(&config).map_err(AddTrackError::MediaProducer)?;
                self.catalog
                    .video(track, wire, config)
                    .map(|p| Media::Video(p.with_buffer(latency)))
            }
            TrackFormat::Audio(format) => {
                let config = audio_config(format, container.into());
                let wire =
                    WireContainer::try_from(&config).map_err(AddTrackError::MediaProducer)?;
                self.catalog
                    .audio(track, wire, config)
                    .map(|p| Media::Audio(p.with_buffer(latency)))
            }
        }
        .map_err(AddTrackError::MediaProducer)?;

        entry.insert(LiveTrack { container, media });
        Ok(())
    }

    pub(crate) fn update_track(
        &mut self,
        track: &str,
        format: TrackFormat,
    ) -> Result<(), UpdateTrackError> {
        self.tracks
            .get_mut(track)
            .ok_or(UpdateTrackError::UnknownTrack)?
            .set(format)
    }

    pub(crate) fn write_frame(
        &mut self,
        track: &str,
        frame: moq_mux::container::Frame,
    ) -> Result<(), WriteFrameError> {
        self.tracks
            .get_mut(track)
            .ok_or(WriteFrameError::UnknownTrack)?
            .write(frame)
            .map_err(|e| match e {
                moq_mux::Error::MissingKeyframe(moq_mux::container::MissingKeyframe) => {
                    WriteFrameError::MissingKeyframe
                }
                e => WriteFrameError::Write(e),
            })
    }

    pub(crate) fn remove_track(&mut self, track: &str) {
        if let Some(mut live) = self.tracks.remove(track) {
            let _ = live.finish();
        }
    }

    pub(crate) fn finish(&mut self) {
        for live in self.tracks.values_mut() {
            let _ = live.finish();
        }
        let _ = self.catalog.finish();
    }

    pub(crate) fn abort(&mut self) {
        self.tracks.clear();
        self.broadcast.clone().close();
    }
}
