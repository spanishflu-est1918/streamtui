//! Detail view for movies and TV shows
//!
//! Shows full info, seasons/episodes for TV, and stream sources.
//! Uses cyberpunk neon aesthetic with keyboard navigation.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, BorderType, List, ListItem, Paragraph, Wrap},
};

use crate::models::{
    Episode, MediaType, MovieDetail, Quality, SearchResult, SeasonSummary, StreamSource, TvDetail,
};
use crate::ui::Theme;

/// Detail view state
#[derive(Debug, Default)]
pub struct DetailView {
    /// The media being displayed (search result)
    pub media: Option<SearchResult>,
    /// Movie details (if movie)
    pub movie_detail: Option<MovieDetail>,
    /// TV details (if TV show)
    pub tv_detail: Option<TvDetail>,
    /// Available stream sources
    pub sources: Vec<StreamSource>,
    /// Selected source index
    pub selected_source: usize,
    /// For TV: available seasons
    pub seasons: Vec<SeasonSummary>,
    /// For TV: selected season
    pub selected_season: usize,
    /// For TV: episodes in selected season
    pub episodes: Vec<Episode>,
    /// For TV: selected episode
    pub selected_episode: usize,
    /// Current focus area
    pub focus: DetailFocus,
    /// Scroll offset for overview text
    pub overview_scroll: u16,
}

/// Focus areas in detail view
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DetailFocus {
    #[default]
    Info,
    Seasons,
    Episodes,
    Sources,
}

impl DetailFocus {
    /// Cycle to next focus area (for TV shows)
    pub fn next_tv(self) -> Self {
        match self {
            DetailFocus::Info => DetailFocus::Seasons,
            DetailFocus::Seasons => DetailFocus::Episodes,
            DetailFocus::Episodes => DetailFocus::Sources,
            DetailFocus::Sources => DetailFocus::Info,
        }
    }

    /// Cycle to previous focus area (for TV shows)
    pub fn prev_tv(self) -> Self {
        match self {
            DetailFocus::Info => DetailFocus::Sources,
            DetailFocus::Seasons => DetailFocus::Info,
            DetailFocus::Episodes => DetailFocus::Seasons,
            DetailFocus::Sources => DetailFocus::Episodes,
        }
    }

    /// Cycle to next focus area (for movies - skip seasons/episodes)
    pub fn next_movie(self) -> Self {
        match self {
            DetailFocus::Info => DetailFocus::Sources,
            DetailFocus::Sources => DetailFocus::Info,
            _ => DetailFocus::Info,
        }
    }

    /// Cycle to previous focus area (for movies)
    pub fn prev_movie(self) -> Self {
        match self {
            DetailFocus::Info => DetailFocus::Sources,
            DetailFocus::Sources => DetailFocus::Info,
            _ => DetailFocus::Info,
        }
    }
}

impl DetailView {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set media to display (from search result)
    pub fn set_media(&mut self, media: SearchResult) {
        self.media = Some(media);
        self.movie_detail = None;
        self.tv_detail = None;
        self.sources.clear();
        self.selected_source = 0;
        self.seasons.clear();
        self.selected_season = 0;
        self.episodes.clear();
        self.selected_episode = 0;
        self.focus = DetailFocus::Info;
        self.overview_scroll = 0;
    }

    /// Set movie details
    pub fn set_movie_detail(&mut self, detail: MovieDetail) {
        self.movie_detail = Some(detail);
    }

    /// Set TV details
    pub fn set_tv_detail(&mut self, detail: TvDetail) {
        self.seasons = detail.seasons.clone();
        self.tv_detail = Some(detail);
        self.selected_season = 0;
    }

    /// Set episodes for current season
    pub fn set_episodes(&mut self, episodes: Vec<Episode>) {
        self.episodes = episodes;
        self.selected_episode = 0;
    }

    /// Set available sources
    pub fn set_sources(&mut self, sources: Vec<StreamSource>) {
        self.sources = sources;
        self.selected_source = 0;
    }

    /// Check if displaying a TV show
    pub fn is_tv(&self) -> bool {
        self.media.as_ref().map(|m| m.media_type == MediaType::Tv).unwrap_or(false)
    }

    /// Get title for display
    pub fn title(&self) -> &str {
        if let Some(movie) = &self.movie_detail {
            &movie.title
        } else if let Some(tv) = &self.tv_detail {
            &tv.name
        } else if let Some(media) = &self.media {
            &media.title
        } else {
            "Unknown"
        }
    }

    /// Get year for display
    pub fn year(&self) -> Option<u16> {
        if let Some(movie) = &self.movie_detail {
            Some(movie.year)
        } else if let Some(tv) = &self.tv_detail {
            Some(tv.year)
        } else if let Some(media) = &self.media {
            media.year
        } else {
            None
        }
    }

    /// Get rating for display
    pub fn rating(&self) -> f32 {
        if let Some(movie) = &self.movie_detail {
            movie.vote_average
        } else if let Some(tv) = &self.tv_detail {
            tv.vote_average
        } else if let Some(media) = &self.media {
            media.vote_average
        } else {
            0.0
        }
    }

    /// Get overview text
    pub fn overview(&self) -> &str {
        if let Some(movie) = &self.movie_detail {
            &movie.overview
        } else if let Some(tv) = &self.tv_detail {
            &tv.overview
        } else if let Some(media) = &self.media {
            &media.overview
        } else {
            ""
        }
    }

    /// Get genres as string
    pub fn genres_str(&self) -> String {
        if let Some(movie) = &self.movie_detail {
            movie.genres.join(", ")
        } else if let Some(tv) = &self.tv_detail {
            tv.genres.join(", ")
        } else {
            String::new()
        }
    }

    /// Get runtime string (movies only)
    pub fn runtime_str(&self) -> Option<String> {
        self.movie_detail.as_ref().map(|m| {
            let hours = m.runtime / 60;
            let mins = m.runtime % 60;
            if hours > 0 {
                format!("{}h {}m", hours, mins)
            } else {
                format!("{}m", mins)
            }
        })
    }

    /// Get selected source
    pub fn current_source(&self) -> Option<&StreamSource> {
        self.sources.get(self.selected_source)
    }

    /// Get selected season
    pub fn current_season(&self) -> Option<&SeasonSummary> {
        self.seasons.get(self.selected_season)
    }

    /// Get selected episode
    pub fn current_episode(&self) -> Option<&Episode> {
        self.episodes.get(self.selected_episode)
    }

    /// Cycle focus to next panel
    pub fn focus_next(&mut self) {
        self.focus = if self.is_tv() {
            self.focus.next_tv()
        } else {
            self.focus.next_movie()
        };
    }

    /// Cycle focus to previous panel
    pub fn focus_prev(&mut self) {
        self.focus = if self.is_tv() {
            self.focus.prev_tv()
        } else {
            self.focus.prev_movie()
        };
    }

    /// Navigate up in current focus area
    pub fn up(&mut self) {
        match self.focus {
            DetailFocus::Info => {
                self.overview_scroll = self.overview_scroll.saturating_sub(1);
            }
            DetailFocus::Seasons => {
                if self.selected_season > 0 {
                    self.selected_season -= 1;
                }
            }
            DetailFocus::Episodes => {
                if self.selected_episode > 0 {
                    self.selected_episode -= 1;
                }
            }
            DetailFocus::Sources => {
                if self.selected_source > 0 {
                    self.selected_source -= 1;
                }
            }
        }
    }

    /// Navigate down in current focus area
    pub fn down(&mut self) {
        match self.focus {
            DetailFocus::Info => {
                self.overview_scroll = self.overview_scroll.saturating_add(1);
            }
            DetailFocus::Seasons => {
                if self.selected_season < self.seasons.len().saturating_sub(1) {
                    self.selected_season += 1;
                }
            }
            DetailFocus::Episodes => {
                if self.selected_episode < self.episodes.len().saturating_sub(1) {
                    self.selected_episode += 1;
                }
            }
            DetailFocus::Sources => {
                if self.selected_source < self.sources.len().saturating_sub(1) {
                    self.selected_source += 1;
                }
            }
        }
    }

    /// Select source by hotkey (1-9)
    pub fn select_source_by_hotkey(&mut self, num: usize) {
        if num > 0 && num <= self.sources.len() {
            self.selected_source = num - 1;
            self.focus = DetailFocus::Sources;
        }
    }

    /// Render the detail view
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if self.media.is_none() {
            self.render_empty(frame, area);
            return;
        }

        if self.is_tv() {
            self.render_tv_layout(frame, area);
        } else {
            self.render_movie_layout(frame, area);
        }
    }

    /// Render empty state
    fn render_empty(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Theme::border())
            .title(Span::styled(" DETAIL ", Theme::title()));

        let empty = Paragraph::new("No media selected")
            .style(Theme::dimmed())
            .alignment(Alignment::Center)
            .block(block);

        frame.render_widget(empty, area);
    }

    /// Render movie layout (info panel + sources)
    fn render_movie_layout(&self, frame: &mut Frame, area: Rect) {
        // Split into info (left 60%) and sources (right 40%)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        self.render_info_panel(frame, chunks[0]);
        self.render_sources_panel(frame, chunks[1]);
    }

    /// Render TV layout (info + seasons/episodes + sources)
    fn render_tv_layout(&self, frame: &mut Frame, area: Rect) {
        // Split into left (info) and right (seasons/episodes/sources)
        let h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Left: info panel
        self.render_info_panel(frame, h_chunks[0]);

        // Right: vertical stack of seasons, episodes, sources
        let v_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(35),
                Constraint::Percentage(40),
            ])
            .split(h_chunks[1]);

        self.render_seasons_panel(frame, v_chunks[0]);
        self.render_episodes_panel(frame, v_chunks[1]);
        self.render_sources_panel(frame, v_chunks[2]);
    }

    /// Render the info panel (title, rating, overview, etc.)
    fn render_info_panel(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus == DetailFocus::Info;
        let border_style = if is_focused {
            Theme::border_focused()
        } else {
            Theme::border()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(Span::styled(" INFO ", Theme::title()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Build content lines
        let mut lines = Vec::new();

        // Title line with neon glow effect
        let year_str = self.year().map(|y| format!(" ({})", y)).unwrap_or_default();
        lines.push(Line::from(vec![
            Span::styled("▶ ", Theme::accent()),
            Span::styled(self.title().to_string(), Theme::title()),
            Span::styled(year_str, Theme::secondary()),
        ]));

        // Rating and runtime
        let rating = self.rating();
        let rating_style = if rating >= 7.5 {
            Theme::success()
        } else if rating >= 6.0 {
            Theme::warning()
        } else {
            Theme::error()
        };

        let mut meta_spans = vec![
            Span::styled(format!("★ {:.1}", rating), rating_style),
        ];

        if let Some(runtime) = self.runtime_str() {
            meta_spans.push(Span::styled(" │ ", Theme::dimmed()));
            meta_spans.push(Span::styled(runtime, Theme::secondary()));
        }

        if self.is_tv() {
            if let Some(tv) = &self.tv_detail {
                meta_spans.push(Span::styled(" │ ", Theme::dimmed()));
                meta_spans.push(Span::styled(
                    format!("{} seasons", tv.seasons.len()),
                    Theme::secondary(),
                ));
            }
        }

        lines.push(Line::from(meta_spans));

        // Genres
        let genres = self.genres_str();
        if !genres.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Genre: ", Theme::dimmed()),
                Span::styled(genres, Theme::text()),
            ]));
        }

        // Separator
        lines.push(Line::from(Span::styled(
            "─".repeat(inner.width as usize),
            Theme::dimmed(),
        )));

        // Overview
        let overview = self.overview();
        if !overview.is_empty() {
            lines.push(Line::from(Span::styled("OVERVIEW", Theme::accent())));
            lines.push(Line::from(""));

            // Word wrap overview text
            for line in overview.lines() {
                lines.push(Line::from(Span::styled(line.to_string(), Theme::text())));
            }
        }

        let text = Text::from(lines);
        let paragraph = Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .scroll((self.overview_scroll, 0));

        frame.render_widget(paragraph, inner);
    }

    /// Render seasons panel (TV only)
    fn render_seasons_panel(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus == DetailFocus::Seasons;
        let border_style = if is_focused {
            Theme::border_focused()
        } else {
            Theme::border()
        };

        let title = format!(" SEASONS ({}) ", self.seasons.len());
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(Span::styled(title, Theme::title()));

        if self.seasons.is_empty() {
            let empty = Paragraph::new("No seasons")
                .style(Theme::dimmed())
                .alignment(Alignment::Center)
                .block(block);
            frame.render_widget(empty, area);
            return;
        }

        let items: Vec<ListItem> = self
            .seasons
            .iter()
            .enumerate()
            .map(|(i, season)| {
                let is_selected = i == self.selected_season;
                let marker = if is_selected { "▸ " } else { "  " };
                let name = season.name.as_deref().unwrap_or("Season");

                let line = Line::from(vec![
                    Span::styled(
                        marker.to_string(),
                        if is_selected { Theme::accent() } else { Theme::dimmed() },
                    ),
                    Span::styled(
                        format!("{} {}", name, season.season_number),
                        if is_selected {
                            Theme::list_item_selected()
                        } else {
                            Theme::text()
                        },
                    ),
                    Span::styled(
                        format!(" ({} eps)", season.episode_count),
                        if is_selected { Theme::accent() } else { Theme::dimmed() },
                    ),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    /// Render episodes panel (TV only)
    fn render_episodes_panel(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus == DetailFocus::Episodes;
        let border_style = if is_focused {
            Theme::border_focused()
        } else {
            Theme::border()
        };

        let title = format!(" EPISODES ({}) ", self.episodes.len());
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(Span::styled(title, Theme::title()));

        if self.episodes.is_empty() {
            let empty = Paragraph::new("Select a season")
                .style(Theme::dimmed())
                .alignment(Alignment::Center)
                .block(block);
            frame.render_widget(empty, area);
            return;
        }

        let inner = block.inner(area);
        let visible_height = inner.height as usize;

        // Calculate offset for scrolling
        let offset = if self.selected_episode >= visible_height {
            self.selected_episode - visible_height + 1
        } else {
            0
        };

        let items: Vec<ListItem> = self
            .episodes
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_height)
            .map(|(i, ep)| {
                let is_selected = i == self.selected_episode;
                let marker = if is_selected { "▸ " } else { "  " };

                // Format: ▸ S01E05 - Episode Name
                let line = Line::from(vec![
                    Span::styled(
                        marker.to_string(),
                        if is_selected { Theme::accent() } else { Theme::dimmed() },
                    ),
                    Span::styled(
                        format!("S{:02}E{:02}", ep.season, ep.episode),
                        if is_selected { Theme::secondary() } else { Theme::secondary() },
                    ),
                    Span::styled(" - ", Theme::dimmed()),
                    Span::styled(
                        ep.name.chars().take(30).collect::<String>(),
                        if is_selected {
                            Theme::list_item_selected()
                        } else {
                            Theme::text()
                        },
                    ),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    /// Render sources panel
    fn render_sources_panel(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus == DetailFocus::Sources;
        let border_style = if is_focused {
            Theme::border_focused()
        } else {
            Theme::border()
        };

        let title = format!(" SOURCES ({}) ", self.sources.len());
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .title(Span::styled(title, Theme::title()));

        if self.sources.is_empty() {
            let empty = Paragraph::new("Loading sources...")
                .style(Theme::loading())
                .alignment(Alignment::Center)
                .block(block);
            frame.render_widget(empty, area);
            return;
        }

        let inner = block.inner(area);
        let visible_height = inner.height as usize;

        // Calculate offset for scrolling
        let offset = if self.selected_source >= visible_height {
            self.selected_source - visible_height + 1
        } else {
            0
        };

        let items: Vec<ListItem> = self
            .sources
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible_height)
            .map(|(i, source)| self.render_source_item(i, source))
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    /// Render a single source item
    /// Format: [1] 1080p  4.2 GB  👤142  Title...
    fn render_source_item(&self, index: usize, source: &StreamSource) -> ListItem<'static> {
        let is_selected = index == self.selected_source;

        // Hotkey [1]-[9]
        let hotkey = if index < 9 {
            format!("[{}] ", index + 1)
        } else {
            "    ".to_string()
        };

        // Quality style
        let quality_style = Self::quality_style(&source.quality, is_selected);
        let seeds_style = Self::seeds_style(source.seeds, is_selected);

        // Truncate title
        let title_display: String = source
            .title
            .lines()
            .next()
            .unwrap_or(&source.title)
            .chars()
            .take(25)
            .collect();

        let line = Line::from(vec![
            Span::styled(
                hotkey,
                if is_selected { Theme::accent() } else { Theme::keybind() },
            ),
            Span::styled(format!("{:<6}", source.quality.to_string()), quality_style),
            Span::styled(
                format!("{:>7}", source.format_size()),
                if is_selected { Theme::accent() } else { Theme::file_size() },
            ),
            Span::raw(" "),
            Span::styled(format!("👤{:<4}", source.seeds), seeds_style),
            Span::raw(" "),
            Span::styled(
                title_display,
                if is_selected {
                    Theme::list_item_selected()
                } else {
                    Theme::dimmed()
                },
            ),
        ]);

        ListItem::new(line)
    }

    /// Style for quality badge
    fn quality_style(quality: &Quality, is_selected: bool) -> Style {
        if is_selected {
            return Theme::accent();
        }
        match quality {
            Quality::UHD4K => Theme::quality_4k(),
            Quality::FHD1080p => Theme::quality_1080p(),
            Quality::HD720p => Theme::quality_720p(),
            Quality::SD480p | Quality::Unknown => Theme::quality_sd(),
        }
    }

    /// Style for seed count
    fn seeds_style(seeds: u32, is_selected: bool) -> Style {
        if is_selected {
            return Theme::accent();
        }
        if seeds >= 100 {
            Theme::seeds_high()
        } else if seeds >= 20 {
            Theme::seeds_medium()
        } else {
            Theme::seeds_low()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_search_result() -> SearchResult {
        SearchResult {
            id: 1,
            media_type: MediaType::Movie,
            title: "The Batman".to_string(),
            year: Some(2022),
            overview: "A dark and gritty Batman film.".to_string(),
            poster_path: None,
            vote_average: 7.8,
        }
    }

    fn sample_tv_search_result() -> SearchResult {
        SearchResult {
            id: 2,
            media_type: MediaType::Tv,
            title: "Breaking Bad".to_string(),
            year: Some(2008),
            overview: "A chemistry teacher turns to crime.".to_string(),
            poster_path: None,
            vote_average: 9.5,
        }
    }

    fn sample_movie_detail() -> MovieDetail {
        MovieDetail {
            id: 1,
            imdb_id: "tt1877830".to_string(),
            title: "The Batman".to_string(),
            year: 2022,
            runtime: 176,
            genres: vec!["Action".to_string(), "Crime".to_string()],
            overview: "A dark and gritty Batman film.".to_string(),
            vote_average: 7.8,
            poster_path: None,
            backdrop_path: None,
        }
    }

    fn sample_tv_detail() -> TvDetail {
        TvDetail {
            id: 2,
            imdb_id: "tt0903747".to_string(),
            name: "Breaking Bad".to_string(),
            year: 2008,
            seasons: vec![
                SeasonSummary {
                    season_number: 1,
                    episode_count: 7,
                    name: Some("Season".to_string()),
                    air_date: None,
                },
                SeasonSummary {
                    season_number: 2,
                    episode_count: 13,
                    name: Some("Season".to_string()),
                    air_date: None,
                },
            ],
            genres: vec!["Drama".to_string(), "Crime".to_string()],
            overview: "A chemistry teacher turns to crime.".to_string(),
            vote_average: 9.5,
            poster_path: None,
            backdrop_path: None,
        }
    }

    fn sample_episodes() -> Vec<Episode> {
        vec![
            Episode {
                season: 1,
                episode: 1,
                name: "Pilot".to_string(),
                overview: "The beginning.".to_string(),
                runtime: Some(58),
                imdb_id: None,
            },
            Episode {
                season: 1,
                episode: 2,
                name: "Cat's in the Bag...".to_string(),
                overview: "Disposal problems.".to_string(),
                runtime: Some(48),
                imdb_id: None,
            },
        ]
    }

    fn sample_sources() -> Vec<StreamSource> {
        vec![
            StreamSource {
                name: "Torrentio".to_string(),
                title: "The.Batman.2022.1080p.BluRay.x264".to_string(),
                info_hash: "abc123".to_string(),
                file_idx: Some(0),
                seeds: 142,
                quality: Quality::FHD1080p,
                size_bytes: Some(4_500_000_000),
            },
            StreamSource {
                name: "Torrentio".to_string(),
                title: "The.Batman.2022.2160p.WEB-DL.HDR".to_string(),
                info_hash: "def456".to_string(),
                file_idx: Some(0),
                seeds: 89,
                quality: Quality::UHD4K,
                size_bytes: Some(12_000_000_000),
            },
        ]
    }

    // =========================================================================
    // Basic State Tests
    // =========================================================================

    #[test]
    fn test_detail_view_new() {
        let view = DetailView::new();
        assert!(view.media.is_none());
        assert!(view.movie_detail.is_none());
        assert!(view.tv_detail.is_none());
        assert!(view.sources.is_empty());
        assert_eq!(view.focus, DetailFocus::Info);
    }

    #[test]
    fn test_set_media_resets_state() {
        let mut view = DetailView::new();
        view.selected_source = 5;
        view.focus = DetailFocus::Sources;

        view.set_media(sample_search_result());

        assert!(view.media.is_some());
        assert_eq!(view.selected_source, 0);
        assert_eq!(view.focus, DetailFocus::Info);
    }

    #[test]
    fn test_is_tv() {
        let mut view = DetailView::new();

        view.set_media(sample_search_result());
        assert!(!view.is_tv());

        view.set_media(sample_tv_search_result());
        assert!(view.is_tv());
    }

    // =========================================================================
    // Title/Year/Rating Accessors
    // =========================================================================

    #[test]
    fn test_title_from_search_result() {
        let mut view = DetailView::new();
        view.set_media(sample_search_result());
        assert_eq!(view.title(), "The Batman");
    }

    #[test]
    fn test_title_from_movie_detail() {
        let mut view = DetailView::new();
        view.set_media(sample_search_result());
        view.set_movie_detail(sample_movie_detail());
        assert_eq!(view.title(), "The Batman");
    }

    #[test]
    fn test_title_from_tv_detail() {
        let mut view = DetailView::new();
        view.set_media(sample_tv_search_result());
        view.set_tv_detail(sample_tv_detail());
        assert_eq!(view.title(), "Breaking Bad");
    }

    #[test]
    fn test_year() {
        let mut view = DetailView::new();
        view.set_media(sample_search_result());
        assert_eq!(view.year(), Some(2022));

        view.set_movie_detail(sample_movie_detail());
        assert_eq!(view.year(), Some(2022));
    }

    #[test]
    fn test_rating() {
        let mut view = DetailView::new();
        view.set_media(sample_search_result());
        assert!((view.rating() - 7.8).abs() < 0.01);
    }

    #[test]
    fn test_genres_str() {
        let mut view = DetailView::new();
        view.set_media(sample_search_result());
        assert_eq!(view.genres_str(), ""); // No detail yet

        view.set_movie_detail(sample_movie_detail());
        assert_eq!(view.genres_str(), "Action, Crime");
    }

    #[test]
    fn test_runtime_str() {
        let mut view = DetailView::new();
        view.set_media(sample_search_result());
        assert!(view.runtime_str().is_none());

        view.set_movie_detail(sample_movie_detail());
        assert_eq!(view.runtime_str(), Some("2h 56m".to_string()));
    }

    // =========================================================================
    // Focus Navigation Tests
    // =========================================================================

    #[test]
    fn test_focus_cycle_movie() {
        let mut view = DetailView::new();
        view.set_media(sample_search_result());

        assert_eq!(view.focus, DetailFocus::Info);

        view.focus_next();
        assert_eq!(view.focus, DetailFocus::Sources);

        view.focus_next();
        assert_eq!(view.focus, DetailFocus::Info);

        view.focus_prev();
        assert_eq!(view.focus, DetailFocus::Sources);
    }

    #[test]
    fn test_focus_cycle_tv() {
        let mut view = DetailView::new();
        view.set_media(sample_tv_search_result());

        assert_eq!(view.focus, DetailFocus::Info);

        view.focus_next();
        assert_eq!(view.focus, DetailFocus::Seasons);

        view.focus_next();
        assert_eq!(view.focus, DetailFocus::Episodes);

        view.focus_next();
        assert_eq!(view.focus, DetailFocus::Sources);

        view.focus_next();
        assert_eq!(view.focus, DetailFocus::Info);
    }

    // =========================================================================
    // Navigation Tests
    // =========================================================================

    #[test]
    fn test_source_navigation() {
        let mut view = DetailView::new();
        view.set_media(sample_search_result());
        view.set_sources(sample_sources());
        view.focus = DetailFocus::Sources;

        assert_eq!(view.selected_source, 0);

        view.down();
        assert_eq!(view.selected_source, 1);

        view.down(); // At end
        assert_eq!(view.selected_source, 1);

        view.up();
        assert_eq!(view.selected_source, 0);

        view.up(); // At start
        assert_eq!(view.selected_source, 0);
    }

    #[test]
    fn test_season_navigation() {
        let mut view = DetailView::new();
        view.set_media(sample_tv_search_result());
        view.set_tv_detail(sample_tv_detail());
        view.focus = DetailFocus::Seasons;

        assert_eq!(view.selected_season, 0);

        view.down();
        assert_eq!(view.selected_season, 1);

        view.up();
        assert_eq!(view.selected_season, 0);
    }

    #[test]
    fn test_episode_navigation() {
        let mut view = DetailView::new();
        view.set_media(sample_tv_search_result());
        view.set_episodes(sample_episodes());
        view.focus = DetailFocus::Episodes;

        assert_eq!(view.selected_episode, 0);

        view.down();
        assert_eq!(view.selected_episode, 1);

        view.up();
        assert_eq!(view.selected_episode, 0);
    }

    #[test]
    fn test_overview_scroll() {
        let mut view = DetailView::new();
        view.set_media(sample_search_result());
        view.focus = DetailFocus::Info;

        assert_eq!(view.overview_scroll, 0);

        view.down();
        assert_eq!(view.overview_scroll, 1);

        view.down();
        assert_eq!(view.overview_scroll, 2);

        view.up();
        assert_eq!(view.overview_scroll, 1);
    }

    // =========================================================================
    // Hotkey Selection Tests
    // =========================================================================

    #[test]
    fn test_select_source_by_hotkey() {
        let mut view = DetailView::new();
        view.set_media(sample_search_result());
        view.set_sources(sample_sources());

        view.select_source_by_hotkey(2);
        assert_eq!(view.selected_source, 1);
        assert_eq!(view.focus, DetailFocus::Sources);

        view.select_source_by_hotkey(1);
        assert_eq!(view.selected_source, 0);
    }

    #[test]
    fn test_select_source_by_hotkey_out_of_range() {
        let mut view = DetailView::new();
        view.set_media(sample_search_result());
        view.set_sources(sample_sources());

        view.select_source_by_hotkey(10); // Out of range
        assert_eq!(view.selected_source, 0); // Unchanged

        view.select_source_by_hotkey(0); // Invalid
        assert_eq!(view.selected_source, 0);
    }

    // =========================================================================
    // Current Selection Accessors
    // =========================================================================

    #[test]
    fn test_current_source() {
        let mut view = DetailView::new();
        view.set_sources(sample_sources());

        let source = view.current_source().unwrap();
        assert_eq!(source.quality, Quality::FHD1080p);

        view.selected_source = 1;
        let source = view.current_source().unwrap();
        assert_eq!(source.quality, Quality::UHD4K);
    }

    #[test]
    fn test_current_source_empty() {
        let view = DetailView::new();
        assert!(view.current_source().is_none());
    }

    #[test]
    fn test_current_season() {
        let mut view = DetailView::new();
        view.set_media(sample_tv_search_result());
        view.set_tv_detail(sample_tv_detail());

        let season = view.current_season().unwrap();
        assert_eq!(season.season_number, 1);

        view.selected_season = 1;
        let season = view.current_season().unwrap();
        assert_eq!(season.season_number, 2);
    }

    #[test]
    fn test_current_episode() {
        let mut view = DetailView::new();
        view.set_episodes(sample_episodes());

        let ep = view.current_episode().unwrap();
        assert_eq!(ep.name, "Pilot");

        view.selected_episode = 1;
        let ep = view.current_episode().unwrap();
        assert_eq!(ep.name, "Cat's in the Bag...");
    }

    // =========================================================================
    // Style Tests
    // =========================================================================

    #[test]
    fn test_quality_style_4k() {
        let style = DetailView::quality_style(&Quality::UHD4K, false);
        assert_eq!(style.fg, Some(Theme::SECONDARY)); // Magenta for 4K
    }

    #[test]
    fn test_quality_style_selected() {
        let style = DetailView::quality_style(&Quality::UHD4K, true);
        assert_eq!(style.fg, Some(Theme::ACCENT)); // Selected = accent
    }

    #[test]
    fn test_seeds_style_high() {
        let style = DetailView::seeds_style(150, false);
        assert_eq!(style.fg, Some(Theme::SUCCESS));
    }

    #[test]
    fn test_seeds_style_medium() {
        let style = DetailView::seeds_style(50, false);
        assert_eq!(style.fg, Some(Theme::WARNING));
    }

    #[test]
    fn test_seeds_style_low() {
        let style = DetailView::seeds_style(10, false);
        assert_eq!(style.fg, Some(Theme::ERROR));
    }

    // =========================================================================
    // DetailFocus Tests
    // =========================================================================

    #[test]
    fn test_detail_focus_default() {
        let focus = DetailFocus::default();
        assert_eq!(focus, DetailFocus::Info);
    }

    #[test]
    fn test_detail_focus_next_tv_cycle() {
        assert_eq!(DetailFocus::Info.next_tv(), DetailFocus::Seasons);
        assert_eq!(DetailFocus::Seasons.next_tv(), DetailFocus::Episodes);
        assert_eq!(DetailFocus::Episodes.next_tv(), DetailFocus::Sources);
        assert_eq!(DetailFocus::Sources.next_tv(), DetailFocus::Info);
    }

    #[test]
    fn test_detail_focus_prev_tv_cycle() {
        assert_eq!(DetailFocus::Info.prev_tv(), DetailFocus::Sources);
        assert_eq!(DetailFocus::Sources.prev_tv(), DetailFocus::Episodes);
        assert_eq!(DetailFocus::Episodes.prev_tv(), DetailFocus::Seasons);
        assert_eq!(DetailFocus::Seasons.prev_tv(), DetailFocus::Info);
    }

    #[test]
    fn test_detail_focus_next_movie_cycle() {
        assert_eq!(DetailFocus::Info.next_movie(), DetailFocus::Sources);
        assert_eq!(DetailFocus::Sources.next_movie(), DetailFocus::Info);
    }

    #[test]
    fn test_detail_focus_prev_movie_cycle() {
        assert_eq!(DetailFocus::Info.prev_movie(), DetailFocus::Sources);
        assert_eq!(DetailFocus::Sources.prev_movie(), DetailFocus::Info);
    }
}
