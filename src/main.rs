use itertools::Itertools;
use rspotify::{prelude::*, scopes, Token};
use rspotify_model::PlaylistId;
use std::collections::HashSet;

use rand::{seq::SliceRandom, thread_rng};

use futures::StreamExt;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let scopes = scopes!(
        "user-library-read",
        "playlist-read-private",
        "playlist-modify-public",
        "playlist-modify-private",
        "playlist-read-collaborative",
        "user-read-currently-playing"
    );

    let oauth = get_oauth_settings(scopes).expect("Error: Failed to generate oauth settings");
    let creds =
        generate_spotify_credentials().expect("Error: Failed to generate spotify credentials");

    let config = rspotify::Config {
        token_cached: true,
        ..Default::default()
    };

    let spotify_client;
    let token = Token::from_cache(&config.cache_path);
    if let Ok(token) = token {
        spotify_client =
            rspotify::AuthCodeSpotify::from_token_with_config(token, creds, oauth, config);

        refresh_access_token(&spotify_client)
            .await
            .expect("Error: Could not refresh access token");
    } else {
        spotify_client = rspotify::AuthCodeSpotify::with_config(creds, oauth, config);
        authorize_with_spotify(&spotify_client).await;
        refresh_access_token(&spotify_client)
            .await
            .expect("Error: Could not refresh access token");
    }

    // let mut stream = spotify_client.current_user_saved_tracks(None);
    // let mut c = 1;
    // while let Some(item) = stream.next().await {
    //     match item {
    //         Ok(item) => {
    //             println!("{:04}. {}", c, item.track.name);
    //             c += 1;
    //         }
    //         Err(e) => {
    //             eprintln!("Error reading track, are you sure the scope and permissions are correct?\n{:?}", e);
    //         }
    //     }
    // }

    let _user_id = spotify_client.current_user().await;

    //"Playlist - Liked Songs"
    let selected_playlist = "spotify:playlist:7JrIBLVJEfpADiic1MKZy5";

    // let mut playlists;
    // match user_id {
    //     Ok(user_id) => {
    //         playlists = spotify_client.user_playlists(user_id.id);
    //         while let Some(playlist) = &playlists.next().await {
    //             match playlist {
    //                 Ok(playlist) => println!("# {} - {}", playlist.id, playlist.name),
    //                 Err(err) => eprintln!("Error reading playlist\n{}", err),
    //             };
    //         }
    //     }
    //     Err(err) => {
    //         eprintln!("Error: Failed to get user's playlists\n{}", err);
    //     }
    // }
    let selected_playlist = PlaylistId::from_uri(selected_playlist).unwrap();

    // let songs_in_playlist = spotify_client.playlist_items(selected_playlist.clone(), None, None);
    let mut liked_songs_stream = spotify_client.current_user_saved_tracks(None);
    let mut songs = vec![];
    while let Some(liked_song) = &liked_songs_stream.next().await {
        if let Ok(song) = liked_song {
            songs.push(song.to_owned());
        }
    }

    // let mut songs: Vec<_> = songs_in_playlist
    //     .collect::<Vec<_>>()
    //     .await
    //     .into_iter()
    //     .filter_map(|f| f.ok())
    //     .collect();

    println!(
        "Found {} songs in playlist {}",
        songs.len(),
        selected_playlist
    );

    songs.iter().take(10).for_each(|s| {
        let track = &s.track;
        println!("Song: {:?} - {:?}", track.name, track.id);
    });

    songs.shuffle(&mut thread_rng());
    println!("---------------------");

    songs.iter().take(10).for_each(|s| {
        let track = &s.track;
        println!(
            "Song: {:?} - {:?} - {:?}",
            &track.name,
            &track.id,
            &track.id.clone().unwrap().uri()
        );
    });

    let songs: Vec<_> = songs
        .iter()
        .map(|s| {
            let track = &s.track;
            PlayableId::Track(track.id.clone().unwrap())
        })
        .collect();

    println!("selected_playlist: {}", &selected_playlist);
    println!(
        "songs: {:#?}",
        &songs.iter().take(5).map(|s| s.id()).collect::<Vec<_>>()
    );

    // clear list
    let playlist_replace_items = spotify_client
        .playlist_replace_items(selected_playlist.clone(), vec![])
        .await;
    match playlist_replace_items {
        Ok(_) => println!("Replaced songs in playlist"),
        Err(err) => println!(
            "Error: Could not replace songs in playlist\n{:?}\n\n{}",
            err, err
        ),
    }

    // split into chunks of 100
    let chunks: Vec<Vec<_>> = songs
        .into_iter()
        .chunks(100)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect();

    for chunk in chunks {
        let add_items = spotify_client
            .playlist_add_items(selected_playlist.clone(), chunk, Some(0))
            .await;

        match add_items {
            Ok(_) => println!("Added songs to playlist"),
            Err(err) => println!(
                "Error: Could not add songs to playlist\n{:?}\n\n{}",
                err, err
            ),
        }
    }
}

async fn refresh_access_token(spotify_client: &rspotify::AuthCodeSpotify) -> Result<(), ()> {
    match spotify_client.refresh_token().await {
        Ok(_) => {
            println!("Refreshed access token");
            Ok(())
        }
        Err(_) => {
            eprintln!("Error: Could not refresh access token");
            Err(())
        }
    }
}

async fn authorize_with_spotify(spotify_client: &rspotify::AuthCodeSpotify) {
    match spotify_client.get_authorize_url(false) {
        Ok(res) => {
            println!("get_authorize_url: {:?}", res);
            let mut url = String::new();
            std::io::stdin()
                .read_line(&mut url)
                .expect("Failed to read from stdin");
            if !url.is_empty() {
                match spotify_client.parse_response_code(&url) {
                    Some(response_code) => {
                        let token_request = spotify_client.request_token(&response_code).await;
                        match token_request {
                            Ok(_) => {
                                println!("Got token from spotify api")
                            }
                            Err(_) => {
                                eprintln!("Error: Failed to retrieve token from spotify api")
                            }
                        }
                    }
                    None => {
                        eprintln!("Error: Could not parse code from response url");
                    }
                }
            }
        }
        Err(_) => {
            eprintln!("Error getting authorization url!")
        }
    }
}

fn get_oauth_settings(scope: HashSet<String>) -> Option<rspotify::OAuth> {
    rspotify::OAuth::from_env(scope)
}

fn generate_spotify_credentials() -> Option<rspotify::Credentials> {
    let spotify_client_id = match std::env::var("SPOTIFY_CLIENT_ID") {
        Ok(client_id) => client_id,
        Err(_) => {
            eprintln!("Failed to retrieve environment variable SPOTIFY_CLIENT_ID.");
            return None;
        }
    };

    let spotify_client_secret = match std::env::var("SPOTIFY_CLIENT_SECRET") {
        Ok(client_id) => client_id,
        Err(_) => {
            eprintln!("Failed to retrieve environment variable SPOTIFY_CLIENT_SECRET.");
            return None;
        }
    };

    let creds = rspotify::Credentials {
        id: spotify_client_id,
        secret: Some(spotify_client_secret),
    };
    Some(creds)
}
