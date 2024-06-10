use futures::stream::Stream;
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Credentials, OAuth};
use std::collections::HashSet;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let scopes = scopes!(
        "playlist-read-private",
        "playlist-read-collaborative",
        "user-read-currently-playing"
    );

    let oauth = get_oauth_settings(scopes).expect("Error: Failed to generate oauth settings");
    let creds =
        generate_spotify_credentials().expect("Error: Failed to generate spotify credentials");

    let config = rspotify::Config {
        ..Default::default()
    };

    let spotify_client = rspotify::AuthCodeSpotify::with_config(creds, oauth, config);

    authorize_with_spotify(&spotify_client).await;

    refresh_access_token(&spotify_client)
        .await
        .expect("Error: Could not refresh access token");

    let stream = spotify_client.current_user_saved_tracks(None);

    // println!("{:?}", playlist);

    // for song in playlist {
    //     println!("{:?}", song);
    // }
    for item in stream {
        println!("* {}", item.unwrap().track.name);
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
    let oauth = rspotify::OAuth::from_env(scope);
    oauth
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
