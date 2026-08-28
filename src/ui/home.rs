//! The Home page.

use egui::{CornerRadius, Rect, Sense, Vec2, pos2, vec2};

use crate::api::models::{Playlist, pick_image};
use crate::app::App;
use crate::model::{Action, Loadable, Page};
use crate::theme::{self, Icon};

use super::widgets;

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    ui.add_space(4.0);
    theme::text(ui, crate::util::greeting(), theme::bold(26.0), palette.text);
    ui.add_space(8.0);
    quick_access(app, ui);
    ui.add_space(10.0);

    made_for_you(app, ui);
    recently_played(app, ui);
}

struct Tile {
    image: Option<String>,
    name: String,
    page: Page,
    uri: Option<String>,
    liked: bool,
}

fn quick_access(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let mut tiles: Vec<Tile> = vec![Tile {
        image: None,
        name: "Liked Songs".to_string(),
        page: Page::LikedSongs,
        uri: app
            .user
            .as_ref()
            .map(|user| format!("spotify:user:{}:collection", user.id)),
        liked: true,
    }];
    for name in ["Discover Weekly", "Release Radar"] {
        if let Some(playlist) = discovered_playlists(app, name)
            .and_then(|playlists| {
                playlists
                    .iter()
                    .find(|playlist| playlist.name.eq_ignore_ascii_case(name))
            })
            .cloned()
        {
            tiles.push(Tile {
                image: pick_image(&playlist.images, 64).map(str::to_string),
                name: playlist.name.clone(),
                page: Page::Playlist(playlist.id.clone()),
                uri: Some(playlist.uri.clone()),
                liked: false,
            });
        }
    }
    if let Some(playlists) = app.library.playlists.get() {
        for uri in &app.settings.pinned_contexts {
            if tiles
                .iter()
                .any(|tile| tile.uri.as_deref() == Some(uri.as_str()))
            {
                continue;
            }
            if let Some(playlist) = playlists.iter().find(|playlist| playlist.uri == *uri) {
                tiles.push(Tile {
                    image: pick_image(&playlist.images, 64).map(str::to_string),
                    name: playlist.name.clone(),
                    page: Page::Playlist(playlist.id.clone()),
                    uri: Some(playlist.uri.clone()),
                    liked: false,
                });
            }
        }
    }
    let available = ui.available_width();
    let columns = ((available / 250.0).floor() as usize).clamp(2, 5);
    let gap = 8.0;
    let tile_width = (available - gap * (columns as f32 - 1.0)) / columns as f32;
    let rows = tiles.len().div_ceil(columns);
    for row in 0..rows {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for column in 0..columns {
                let Some(Tile {
                    image,
                    name,
                    page,
                    uri,
                    liked,
                }) = tiles.get(row * columns + column)
                else {
                    break;
                };
                let (rect, response) =
                    ui.allocate_exact_size(vec2(tile_width, 52.0), Sense::click());
                if ui.is_rect_visible(rect) {
                    let hovered = ui.rect_contains_pointer(rect);
                    let fill = if hovered {
                        palette.surface_hover
                    } else {
                        palette.surface
                    };
                    ui.painter().rect_filled(rect, CornerRadius::same(6), fill);
                    let cover = Rect::from_min_size(rect.min, Vec2::splat(52.0));
                    if *liked {
                        super::sidebar::liked_cover(ui, cover, 6.0);
                    } else {
                        widgets::paint_cover(
                            ui,
                            &palette,
                            image.as_deref(),
                            cover,
                            6.0,
                            Icon::Music,
                        );
                    }
                    let play_room = if hovered && uri.is_some() { 46.0 } else { 10.0 };
                    let text_rect = Rect::from_min_max(
                        pos2(cover.right() + 10.0, rect.top()),
                        pos2(rect.right() - play_room, rect.bottom()),
                    );
                    ui.painter().with_clip_rect(text_rect).text(
                        pos2(text_rect.left(), rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        name,
                        theme::semibold(14.0),
                        palette.text,
                    );
                    if hovered && let Some(uri) = uri {
                        let button = Rect::from_center_size(
                            pos2(rect.right() - 24.0, rect.center().y),
                            Vec2::splat(36.0),
                        );
                        let mut child =
                            ui.new_child(egui::UiBuilder::new().max_rect(button).layout(
                                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            ));
                        if theme::circle_button(
                            &mut child,
                            Icon::PlayFilled,
                            36.0,
                            palette.accent,
                            palette.accent_hover,
                            palette.on_accent,
                            "Play",
                        )
                        .clicked()
                        {
                            app.actions.push(Action::PlayContext {
                                uri: uri.clone(),
                                offset_uri: None,
                                offset_index: None,
                            });
                        }
                    }
                }
                if response
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    app.actions.push(Action::Open(page.clone()));
                }
            }
        });
    }
}

fn discovered_playlists<'a>(app: &'a App, term: &str) -> Option<&'a [Playlist]> {
    app.home
        .discover
        .get(term)
        .and_then(Loadable::get)
        .map(Vec::as_slice)
}

fn daily_mix_number(name: &str) -> Option<u8> {
    let number = name.strip_prefix("Daily Mix ")?.parse().ok()?;
    (1..=6).contains(&number).then_some(number)
}

fn made_for_you(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let loading = ["Daily Mix", "daylist"]
        .iter()
        .any(|term| matches!(app.home.discover.get(*term), Some(Loadable::Loading)));
    let mut mixes: Vec<(u8, Playlist)> = discovered_playlists(app, "Daily Mix")
        .into_iter()
        .flatten()
        .filter_map(|playlist| Some((daily_mix_number(&playlist.name)?, playlist.clone())))
        .collect();
    mixes.sort_by_key(|(number, _)| *number);
    let mut playlists: Vec<Playlist> = mixes
        .into_iter()
        .take(7)
        .map(|(_, playlist)| playlist)
        .collect();
    if let Some(daylist) = discovered_playlists(app, "daylist")
        .and_then(|playlists| {
            playlists
                .iter()
                .find(|playlist| playlist.name.to_lowercase().contains("daylist"))
        })
        .cloned()
    {
        playlists.push(daylist);
    }
    if playlists.is_empty() && !loading {
        return;
    }
    widgets::shelf(ui, &palette, "made-for-you", "Made for you", |ui| {
        if playlists.is_empty() {
            widgets::loading_row(ui, &palette);
        }
        for playlist in &playlists {
            let subtitle = playlist
                .description
                .as_deref()
                .map(crate::util::strip_html)
                .filter(|d| !d.is_empty())
                .unwrap_or_else(|| format!("By {}", playlist.owner_name()));
            let card = widgets::card(
                ui,
                app,
                pick_image(&playlist.images, 300),
                &playlist.name,
                &subtitle,
                false,
                true,
            );
            if card.play {
                app.actions.push(Action::PlayContext {
                    uri: playlist.uri.clone(),
                    offset_uri: None,
                    offset_index: None,
                });
            }
            if card.clicked {
                app.actions
                    .push(Action::Open(Page::Playlist(playlist.id.clone())));
            }
        }
    });
}

fn recently_played(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let history = match &app.home.recently_played {
        Loadable::Loaded(history) => history.clone(),
        Loadable::Loading | Loadable::NotLoaded => {
            widgets::shelf(ui, &palette, "recent", "Recently played", |ui| {
                widgets::loading_row(ui, &palette)
            });
            return;
        }
        Loadable::Failed(_) => return,
    };
    let mut seen = std::collections::HashSet::new();
    let tracks: Vec<_> = history
        .into_iter()
        .filter(|entry| {
            entry
                .track
                .id
                .as_ref()
                .is_some_and(|id| seen.insert(id.clone()))
        })
        .take(6)
        .collect();
    if tracks.is_empty() {
        return;
    }
    widgets::shelf(ui, &palette, "recent", "Recently played", |ui| {
        for entry in &tracks {
            let track = &entry.track;
            let card = widgets::card(
                ui,
                app,
                track.image(300),
                &track.name,
                &track.artist_names(),
                false,
                true,
            );
            if card.play {
                app.actions.push(Action::PlayUris {
                    uris: vec![track.uri.clone()],
                    index: 0,
                });
            }
            if card.clicked
                && let Some(album) = &track.album
                && !album.id.is_empty()
            {
                app.actions
                    .push(Action::Open(Page::Album(album.id.clone())));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::daily_mix_number;

    #[test]
    fn only_numbered_daily_mixes_one_through_six_are_homeworthy() {
        assert_eq!(daily_mix_number("Daily Mix 1"), Some(1));
        assert_eq!(daily_mix_number("Daily Mix 6"), Some(6));
        assert_eq!(daily_mix_number("Daily Mix 7"), None);
        assert_eq!(daily_mix_number("Daily Mix"), None);
    }
}
