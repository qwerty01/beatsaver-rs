#![cfg(feature = "sync")]
use crate::map::Map;
use crate::{BeatSaverApiError, BeatSaverUser, MapId, Page, BEATSAVER_URL};
use bytes::Bytes;
use hex;
use serde::Serialize;
use serde_json;
use std::collections::VecDeque;
use std::convert::From;
use std::error::Error;
use url::Url;
use urlencoding::encode;

/// Structure used for iterating over a page
pub struct PageIterator<T: Serialize, E: Error, F>
where
    BeatSaverApiError<E>: From<E>,
    F: Fn(usize) -> Result<Page<T>, BeatSaverApiError<E>> + ?Sized,
{
    curr: Page<T>,
    next_page: Box<F>,
}

impl<T: Serialize, E: Error, F> Iterator for PageIterator<T, E, F>
where
    BeatSaverApiError<E>: From<E>,
    F: Fn(usize) -> Result<Page<T>, BeatSaverApiError<E>> + ?Sized,
{
    type Item = Result<T, BeatSaverApiError<E>>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.curr.docs.is_empty() {
            // We're at the end of the current page
            self.curr = match self.curr.next_page {
                Some(n) => {
                    let next = self.next_page.as_ref();
                    match next(n) {
                        Ok(s) => s,
                        Err(e) => return Some(Err(e.into())),
                    }
                }
                None => return None,
            };
        }
        let item = self.curr.docs.pop_front().unwrap();
        Some(Ok(item))
    }
}

/// API trait for synchronous clients
pub trait BeatSaverApiSync<'a, T: 'a + Error>
where
    BeatSaverApiError<T>: From<T>,
{
    /// Executes a raw request to the provided [Url][url::Url]
    ///
    /// Make sure to handle 429 (pass the data to [rate_limit][crate::rate_limit])
    fn request_raw(&'a self, url: Url) -> Result<Bytes, BeatSaverApiError<T>>;
    /// Executes a request and converts the result into a [String][std::string::String]
    fn request(&'a self, url: Url) -> Result<String, BeatSaverApiError<T>> {
        let data = self.request_raw(url)?;
        Ok(String::from_utf8(data.as_ref().to_vec())?)
    }
    /// Gets a map from a given [MapId][crate::MapId]
    fn map(&'a self, id: &'a MapId) -> Result<Map, BeatSaverApiError<T>> {
        let data = match id {
            MapId::Key(k) => self.request(
                BEATSAVER_URL
                    .join(format!("api/maps/detail/{:x}", k).as_str())
                    .unwrap(),
            )?,
            MapId::Hash(h) => self.request(
                BEATSAVER_URL
                    .join(format!("api/maps/by-hash/{}", h).as_str())
                    .unwrap(),
            )?,
        };

        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves maps created by a specified beatsaver user
    fn maps_by(
        &'a self,
        user: &'a BeatSaverUser,
    ) -> PageIterator<Map, T, dyn Fn(usize) -> Result<Page<Map>, BeatSaverApiError<T>> + 'a> {
        self.maps_by_page_iter(user, 0)
    }
    /// Retrieves maps created by a specified beatsaver user, specifying a page number
    fn maps_by_page(
        &'a self,
        user: &BeatSaverUser,
        page: usize,
    ) -> Result<Page<Map>, BeatSaverApiError<T>> {
        let url = BEATSAVER_URL
            .join(format!("api/maps/uploader/{}/", user.id).as_str())
            .unwrap();
        let data = self.request(url.join(page.to_string().as_str()).unwrap())?;
        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves maps created by a specified beatsaver user, specifying a page number, iterable
    fn maps_by_page_iter(
        &'a self,
        user: &'a BeatSaverUser,
        page: usize,
    ) -> PageIterator<Map, T, dyn Fn(usize) -> Result<Page<Map>, BeatSaverApiError<T>> + 'a> {
        let page = Page {
            docs: VecDeque::<Map>::new(),
            total_docs: 0,
            last_page: 0,
            prev_page: None,
            next_page: Some(page),
        };

        let next = move |p| self.maps_by_page(user, p);

        PageIterator {
            curr: page,
            next_page: Box::new(next),
        }
    }
    /// Retrieves the current hot maps on beatsaver
    fn maps_hot(
        &'a self,
    ) -> PageIterator<Map, T, dyn Fn(usize) -> Result<Page<Map>, BeatSaverApiError<T>> + 'a> {
        self.maps_hot_page_iter(0)
    }
    /// Retrieves the current hot maps on beatsaver, specifying a page number
    fn maps_hot_page(&'a self, page: usize) -> Result<Page<Map>, BeatSaverApiError<T>> {
        let url = BEATSAVER_URL.join("api/maps/hot/").unwrap();
        let data = self.request(url.join(page.to_string().as_str()).unwrap())?;
        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves the current hot maps on beatsaver, specifying a page number, iterable
    fn maps_hot_page_iter(
        &'a self,
        page: usize,
    ) -> PageIterator<Map, T, dyn Fn(usize) -> Result<Page<Map>, BeatSaverApiError<T>> + 'a> {
        let page = Page {
            docs: VecDeque::<Map>::new(),
            total_docs: 0,
            last_page: 0,
            prev_page: None,
            next_page: Some(page),
        };

        let next = move |p| self.maps_hot_page(p);

        PageIterator {
            curr: page,
            next_page: Box::new(next),
        }
    }
    /// Retrieves all maps sorted by rating
    fn maps_rating(
        &'a self,
    ) -> PageIterator<Map, T, dyn Fn(usize) -> Result<Page<Map>, BeatSaverApiError<T>> + 'a> {
        self.maps_rating_page_iter(0)
    }
    /// Retrieves all maps sorted by rating, specifying a page number
    fn maps_rating_page(&'a self, page: usize) -> Result<Page<Map>, BeatSaverApiError<T>> {
        let url = BEATSAVER_URL.join("api/maps/rating/").unwrap();
        let data = self.request(url.join(page.to_string().as_str()).unwrap())?;
        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves all maps sorted by rating, specifying a page number, iterable
    fn maps_rating_page_iter(
        &'a self,
        page: usize,
    ) -> PageIterator<Map, T, dyn Fn(usize) -> Result<Page<Map>, BeatSaverApiError<T>> + 'a> {
        let page = Page {
            docs: VecDeque::<Map>::new(),
            total_docs: 0,
            last_page: 0,
            prev_page: None,
            next_page: Some(page),
        };

        let next = move |p| self.maps_rating_page(p);

        PageIterator {
            curr: page,
            next_page: Box::new(next),
        }
    }
    /// Retrieves all maps sorted by upload time
    fn maps_latest(
        &'a self,
    ) -> PageIterator<Map, T, dyn Fn(usize) -> Result<Page<Map>, BeatSaverApiError<T>> + 'a> {
        self.maps_latest_page_iter(0)
    }
    /// Retrieves all maps sorted by upload time, specifying a page number
    fn maps_latest_page(&'a self, page: usize) -> Result<Page<Map>, BeatSaverApiError<T>> {
        let url = BEATSAVER_URL.join("api/maps/latest/").unwrap();
        let data = self.request(url.join(page.to_string().as_str()).unwrap())?;
        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves all maps sorted by upload time, specifying a page number
    fn maps_latest_page_iter(
        &'a self,
        page: usize,
    ) -> PageIterator<Map, T, dyn Fn(usize) -> Result<Page<Map>, BeatSaverApiError<T>> + 'a> {
        let page = Page {
            docs: VecDeque::<Map>::new(),
            total_docs: 0,
            last_page: 0,
            prev_page: None,
            next_page: Some(page),
        };

        let next = move |p| self.maps_latest_page(p);

        PageIterator {
            curr: page,
            next_page: Box::new(next),
        }
    }
    /// Retrieves all maps sorted by total downloads
    fn maps_downloads(
        &'a self,
    ) -> PageIterator<Map, T, dyn Fn(usize) -> Result<Page<Map>, BeatSaverApiError<T>> + 'a> {
        self.maps_downloads_page_iter(0)
    }
    /// Retrieves all maps sorted by total downloads, specifying a page number
    fn maps_downloads_page(&'a self, page: usize) -> Result<Page<Map>, BeatSaverApiError<T>> {
        let url = BEATSAVER_URL.join("api/maps/downloads/").unwrap();
        let data = self.request(url.join(page.to_string().as_str()).unwrap())?;
        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves all maps sorted by total downloads, specifying a page number, iterable
    fn maps_downloads_page_iter(
        &'a self,
        page: usize,
    ) -> PageIterator<Map, T, dyn Fn(usize) -> Result<Page<Map>, BeatSaverApiError<T>> + 'a> {
        let page = Page {
            docs: VecDeque::<Map>::new(),
            total_docs: 0,
            last_page: 0,
            prev_page: None,
            next_page: Some(page),
        };

        let next = move |p| self.maps_downloads_page(p);

        PageIterator {
            curr: page,
            next_page: Box::new(next),
        }
    }
    /// Retrieves all maps sorted by number of plays
    fn maps_plays(
        &'a self,
    ) -> PageIterator<Map, T, dyn Fn(usize) -> Result<Page<Map>, BeatSaverApiError<T>> + 'a> {
        self.maps_plays_page_iter(0)
    }
    /// Retrieves all maps sorted by number of plays, specifying a page number
    fn maps_plays_page(&'a self, page: usize) -> Result<Page<Map>, BeatSaverApiError<T>> {
        let url = BEATSAVER_URL.join("api/maps/plays/").unwrap();
        let data = self.request(url.join(page.to_string().as_str()).unwrap())?;
        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves all maps sorted by number of plays, specifying a page number
    fn maps_plays_page_iter(
        &'a self,
        page: usize,
    ) -> PageIterator<Map, T, dyn Fn(usize) -> Result<Page<Map>, BeatSaverApiError<T>> + 'a> {
        let page = Page {
            docs: VecDeque::<Map>::new(),
            total_docs: 0,
            last_page: 0,
            prev_page: None,
            next_page: Some(page),
        };

        let next = move |p| self.maps_plays_page(p);

        PageIterator {
            curr: page,
            next_page: Box::new(next),
        }
    }
    /// Retrieves info on a specified beatsaber user
    fn user(&'a self, id: String) -> Result<BeatSaverUser, BeatSaverApiError<T>> {
        if id.len() != 24 || hex::decode(&id).is_err() {
            return Err(BeatSaverApiError::ArgumentError("id"));
        }
        let data = self.request(
            BEATSAVER_URL
                .join(format!("api/users/find/{}", id).as_str())
                .unwrap(),
        )?;

        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves maps based on a specified search query
    ///
    /// Note: urlencodes the query
    fn search(
        &'a self,
        query: &'a String,
    ) -> PageIterator<Map, T, dyn Fn(usize) -> Result<Page<Map>, BeatSaverApiError<T>> + 'a> {
        self.search_page_iter(query, 0)
    }
    /// Retrieves maps based on a specified search query, specifying a page number
    ///
    /// Note: urlencodes the query
    fn search_page(
        &'a self,
        query: &'a String,
        page: usize,
    ) -> Result<Page<Map>, BeatSaverApiError<T>> {
        let query = encode(query.as_str());
        let url = BEATSAVER_URL
            .join(format!("api/search/text/{}?q={}", page, query).as_str())
            .unwrap();
        let data = self.request(url)?;
        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves maps based on a specified search query, starting at the specified page
    ///
    /// Note: urlencodes the query
    fn search_page_iter(
        &'a self,
        query: &'a String,
        page: usize,
    ) -> PageIterator<Map, T, dyn Fn(usize) -> Result<Page<Map>, BeatSaverApiError<T>> + 'a> {
        // TODO: Don't make a request! Should return PageIterator every time!
        let page = Page {
            docs: VecDeque::<Map>::new(),
            total_docs: 0,
            last_page: 0,
            prev_page: None,
            next_page: Some(page),
        };

        let next = move |p| self.search_page(query, p);

        PageIterator {
            curr: page,
            next_page: Box::new(next),
        }
    }
    /// Retrieves maps based on an advanced search query
    ///
    /// Note: urlencodes the query
    ///
    /// Advanced queries use [Apache Lucene](https://lucene.apache.org/core/2_9_4/queryparsersyntax.html) syntax
    fn search_advanced(
        &'a self,
        query: &'a String,
    ) -> PageIterator<Map, T, dyn Fn(usize) -> Result<Page<Map>, BeatSaverApiError<T>> + 'a> {
        self.search_advanced_page_iter(query, 0)
    }
    /// Retrieves maps based on an advanced search query, specifying a page
    ///
    /// Note: urlencodes the query
    ///
    /// Advanced queries use [Apache Lucene](https://lucene.apache.org/core/2_9_4/queryparsersyntax.html) syntax
    fn search_advanced_page(
        &'a self,
        query: &'a String,
        page: usize,
    ) -> Result<Page<Map>, BeatSaverApiError<T>> {
        // TODO: Validate Lucene syntax
        let query = encode(query.as_str());
        let url = BEATSAVER_URL
            .join(format!("api/search/advanced/{}?q={}", page, query).as_str())
            .unwrap();
        let data = self.request(url)?;
        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves maps based on an advanced search query, specifying a page, iterable
    ///
    /// Note: urlencodes the query
    ///
    /// Advanced queries use [Apache Lucene](https://lucene.apache.org/core/2_9_4/queryparsersyntax.html) syntax
    fn search_advanced_page_iter(
        &'a self,
        query: &'a String,
        page: usize,
    ) -> PageIterator<Map, T, dyn Fn(usize) -> Result<Page<Map>, BeatSaverApiError<T>> + 'a> {
        let page = Page {
            docs: VecDeque::<Map>::new(),
            total_docs: 0,
            last_page: 0,
            prev_page: None,
            next_page: Some(page),
        };

        let next = move |p| self.search_advanced_page(query, p);

        PageIterator {
            curr: page,
            next_page: Box::new(next),
        }
    }
    /// Downloads a provided map
    ///
    /// [Maps][crate::map::Map] can be converted to [MapIds][crate::MapId] using the [Into][std::convert::Into] trait.
    fn download(&'a self, id: MapId) -> Result<Bytes, BeatSaverApiError<T>> {
        Ok(self.request_raw(
            BEATSAVER_URL
                .join(
                    match id {
                        MapId::Key(k) => format!("api/download/key/{:x}", k),
                        MapId::Hash(h) => format!("api/download/hash/{}", h),
                    }
                    .as_str(),
                )
                .unwrap(),
        )?)
    }
}

#[cfg(test)]
mod tests;