#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link<'a> {
    pub link: &'a str,
    pub kind: Kind,
    pub site: Site,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    Track,
    Playlist,
    Album,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Site {
    Spotify,
    Soundclound,
    Bandcamp,
}

mod spotify {
    use regex::Regex;
    use std::sync::LazyLock;

    use super::{Kind, Link};

    static SPOTIFY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"https?:\/\/open\.spotify\.com\/(album|playlist|track)\/([a-zA-Z0-9]+)")
            .unwrap()
    });

    pub fn get_links(text: &str) -> Vec<Link<'_>> {
        SPOTIFY_REGEX
            .captures_iter(text)
            .map(|capture| {
                // We ignore the id for now
                let (full, [kind, _id]) = capture.extract();
                Link {
                    link: full,
                    kind: match kind {
                        "album" => Kind::Album,
                        "playlist" => Kind::Playlist,
                        "track" => Kind::Track,
                        _ => unreachable!("unhandled kind {kind}"),
                    },
                    site: super::Site::Spotify,
                }
            })
            .collect()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn matches(result: &Link<'_>, link: &str, kind: Kind) -> bool {
            result.link == link
                && result.kind == kind
                && result.site == super::super::Site::Spotify
        }

        #[test]
        fn test_basic() {
            let links = get_links("https://open.spotify.com/album/someidhere?si=withthetrackingnonsense lsfadlsl https://open.spotify.com/playlist/anotherid?si=morenonsense ljsfaljksadflj https://open.spotify.com/playlist/myplaylistidhere?si=woo&pi=yea ljksasd https://open.spotify.com/track/finallyatrackid?si=yeapppp");

            assert_eq!(4, links.len());

            matches(
                &links[0],
                "https://open.spotify.com/album/someidhere",
                Kind::Album,
            );
            matches(
                &links[1],
                "https://open.spotify.com/playlist/anotherid",
                Kind::Playlist,
            );
            matches(
                &links[2],
                "https://open.spotify.com/playlist/myplaylistidhere",
                Kind::Playlist,
            );
            matches(
                &links[3],
                "https://open.spotify.com/track/finallytrackid",
                Kind::Track,
            );
        }

        #[test]
        fn test_nothing() {
            let links = get_links("nothing to find here");

            assert_eq!(0, links.len());
        }
    }
}

pub fn get_music_links(text: &str) -> Vec<Link<'_>> {
    let spotify = spotify::get_links(text);

    spotify
}
