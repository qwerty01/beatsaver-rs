#![cfg(feature = "async")]
use crate::{BeatSaverApiError, BeatSaverUser, Map, MapId, Page, BEATSAVER_URL};
use async_trait::async_trait;
use bytes::Bytes;
use futures::{stream, Future, Stream, StreamExt};
use serde::Serialize;
use std::error::Error;
use std::pin::Pin;
use url::Url;
use urlencoding::encode;

fn iterate_page<
    'a,
    T: Serialize,
    E: Error,
    F: Fn(usize) -> Pin<Box<dyn Future<Output = Result<Page<T>, BeatSaverApiError<E>>> + 'a>> + 'a,
>(
    f: F,
    initial: usize,
) -> Pin<Box<dyn Stream<Item = Result<T, BeatSaverApiError<E>>> + 'a>>
where
    F: Copy,
{
    Box::pin(
        stream::unfold(Some(initial), move |num| async move {
            match num {
                Some(n) => {
                    let page = f(n).await;
                    match page {
                        Ok(p) => {
                            let v: Vec<Result<T, BeatSaverApiError<E>>> =
                                p.docs.into_iter().map(Ok).collect();
                            Some((stream::iter(v), p.next_page))
                        }
                        Err(e) => {
                            let v = vec![Err(e.into())];
                            Some((stream::iter(v), Some(n)))
                        }
                    }
                }
                None => None,
            }
        })
        .flatten(),
    )
}

/// API trait for asynchronous clients
#[async_trait]
pub trait BeatSaverApiAsync<'a, T: 'a + Error>
where
    BeatSaverApiError<T>: From<T>,
{
    /// Executes a raw request to the provided [Url][url::Url]
    ///
    /// Make sure to handle 429 (pass the data to [rate_limit][crate::rate_limit])
    async fn request_raw(&'a self, url: Url) -> Result<Bytes, BeatSaverApiError<T>>;
    /// Executes a request and converts the result into a [String][std::string::String]
    async fn request(&'a self, url: Url) -> Result<String, BeatSaverApiError<T>> {
        let data = self.request_raw(url).await?;
        Ok(String::from_utf8(data.as_ref().to_vec())?)
    }
    /// Gets a map from a given [MapId][crate::MapId]
    async fn map(&'a self, id: &'a MapId) -> Result<Map, BeatSaverApiError<T>> {
        let data = match id {
            MapId::Key(k) => {
                let url = BEATSAVER_URL
                    .join(format!("api/maps/detail/{:x}", k).as_str())
                    .unwrap();
                self.request(url.clone()).await?
            }
            MapId::Hash(h) => {
                let url = BEATSAVER_URL
                    .join(format!("api/maps/by-hash/{}", h).as_str())
                    .unwrap();
                self.request(url.clone()).await?
            }
        };

        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves maps created by a specified beatsaver user
    fn maps_by(
        &'a self,
        user: &'a BeatSaverUser,
    ) -> Pin<Box<dyn Stream<Item = Result<Map, BeatSaverApiError<T>>> + 'a>>
    where
        Self: Send + Sync,
    {
        self.maps_by_page_iter(user, 0)
    }
    /// Retrieves maps created by a specified beatsaver user, specifying a page number
    async fn maps_by_page(
        &'a self,
        user: &'a BeatSaverUser,
        page: usize,
    ) -> Result<Page<Map>, BeatSaverApiError<T>> {
        let url = BEATSAVER_URL
            .join(format!("api/maps/uploader/{}/", user.id).as_str())
            .unwrap();
        let data = self
            .request(url.join(page.to_string().as_str()).unwrap())
            .await?;
        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves maps created by a specified beatsaver user, specifying a page number, iterable
    fn maps_by_page_iter(
        &'a self,
        user: &'a BeatSaverUser,
        page: usize,
    ) -> Pin<Box<dyn Stream<Item = Result<Map, BeatSaverApiError<T>>> + 'a>>
    where
        Self: Send + Sync,
    {
        iterate_page(move |p| self.maps_by_page(user, p), page)
    }
    /// Retrieves the current hot maps on beatsaver
    fn maps_hot(&'a self) -> Pin<Box<dyn Stream<Item = Result<Map, BeatSaverApiError<T>>> + 'a>>
    where
        Self: Send + Sync,
    {
        self.maps_hot_page_iter(0)
    }
    /// Retrieves the current hot maps on beatsaver, specifying a page number
    async fn maps_hot_page(&'a self, page: usize) -> Result<Page<Map>, BeatSaverApiError<T>> {
        let url = BEATSAVER_URL.join("api/maps/hot/").unwrap();
        let data = self
            .request(url.join(page.to_string().as_str()).unwrap())
            .await?;
        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves the current hot maps on beatsaver, specifying a page number, iterable
    fn maps_hot_page_iter(
        &'a self,
        page: usize,
    ) -> Pin<Box<dyn Stream<Item = Result<Map, BeatSaverApiError<T>>> + 'a>>
    where
        Self: Send + Sync,
    {
        iterate_page(move |p| self.maps_hot_page(p), page)
    }
    /// Retrieves all maps sorted by rating
    fn maps_rating(&'a self) -> Pin<Box<dyn Stream<Item = Result<Map, BeatSaverApiError<T>>> + 'a>>
    where
        Self: Send + Sync,
    {
        self.maps_rating_page_iter(0)
    }
    /// Retrieves all maps sorted by rating, specifying a page number
    async fn maps_rating_page(&'a self, page: usize) -> Result<Page<Map>, BeatSaverApiError<T>> {
        let url = BEATSAVER_URL.join("api/maps/rating/").unwrap();
        let data = self
            .request(url.join(page.to_string().as_str()).unwrap())
            .await?;
        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves all maps sorted by rating, specifying a page number, iterable
    fn maps_rating_page_iter(
        &'a self,
        page: usize,
    ) -> Pin<Box<dyn Stream<Item = Result<Map, BeatSaverApiError<T>>> + 'a>>
    where
        Self: Send + Sync,
    {
        iterate_page(move |p| self.maps_rating_page(p), page)
    }
    /// Retrieves all maps sorted by upload time
    fn maps_latest(&'a self) -> Pin<Box<dyn Stream<Item = Result<Map, BeatSaverApiError<T>>> + 'a>>
    where
        Self: Send + Sync,
    {
        self.maps_latest_page_iter(0)
    }
    /// Retrieves all maps sorted by upload time, specifying a page number
    async fn maps_latest_page(&'a self, page: usize) -> Result<Page<Map>, BeatSaverApiError<T>> {
        let url = BEATSAVER_URL.join("api/maps/latest/").unwrap();
        let data = self
            .request(url.join(page.to_string().as_str()).unwrap())
            .await?;
        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves all maps sorted by upload time, specifying a page number, iterable
    fn maps_latest_page_iter(
        &'a self,
        page: usize,
    ) -> Pin<Box<dyn Stream<Item = Result<Map, BeatSaverApiError<T>>> + 'a>>
    where
        Self: Send + Sync,
    {
        iterate_page(move |p| self.maps_latest_page(p), page)
    }
    /// Retrieves all maps sorted by total downloads
    fn maps_downloads(
        &'a self,
    ) -> Pin<Box<dyn Stream<Item = Result<Map, BeatSaverApiError<T>>> + 'a>>
    where
        Self: Send + Sync,
    {
        self.maps_downloads_page_iter(0)
    }
    /// Retrieves all maps sorted by total downloads, specifying a page number
    async fn maps_downloads_page(&'a self, page: usize) -> Result<Page<Map>, BeatSaverApiError<T>> {
        let url = BEATSAVER_URL.join("api/maps/downloads/").unwrap();
        let data = self
            .request(url.join(page.to_string().as_str()).unwrap())
            .await?;
        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves all maps sorted by total downloads, specifying a page number, iterable
    fn maps_downloads_page_iter(
        &'a self,
        page: usize,
    ) -> Pin<Box<dyn Stream<Item = Result<Map, BeatSaverApiError<T>>> + 'a>>
    where
        Self: Send + Sync,
    {
        iterate_page(move |p| self.maps_downloads_page(p), page)
    }
    /// Retrieves all maps sorted by number of plays, specifying a page number
    fn maps_plays(&'a self) -> Pin<Box<dyn Stream<Item = Result<Map, BeatSaverApiError<T>>> + 'a>>
    where
        Self: Send + Sync,
    {
        self.maps_plays_page_iter(0)
    }
    /// Retrieves all maps sorted by number of plays
    async fn maps_plays_page(&'a self, page: usize) -> Result<Page<Map>, BeatSaverApiError<T>> {
        let url = BEATSAVER_URL.join("api/maps/plays/").unwrap();
        let data = self
            .request(url.join(page.to_string().as_str()).unwrap())
            .await?;
        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves all maps sorted by number of plays, iterable
    fn maps_plays_page_iter(
        &'a self,
        page: usize,
    ) -> Pin<Box<dyn Stream<Item = Result<Map, BeatSaverApiError<T>>> + 'a>>
    where
        Self: Send + Sync,
    {
        iterate_page(move |p| self.maps_plays_page(p), page)
    }
    /// Retrieves info on a specified beatsaber user
    async fn user(&'a self, id: String) -> Result<BeatSaverUser, BeatSaverApiError<T>> {
        if id.len() != 24 || hex::decode(&id).is_err() {
            return Err(BeatSaverApiError::ArgumentError("id"));
        }
        let url = BEATSAVER_URL
            .join(format!("api/users/find/{}", id).as_str())
            .unwrap();
        let data = self.request(url.clone()).await?;

        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves maps based on a specified search query
    ///
    /// Note: urlencodes the query
    fn search(
        &'a self,
        query: &'a String,
    ) -> Pin<Box<dyn Stream<Item = Result<Map, BeatSaverApiError<T>>> + 'a>>
    where
        Self: Send + Sync,
    {
        self.search_page_iter(query, 0)
    }
    /// Retrieves maps based on a specified search query, specifying a page number
    ///
    /// Note: urlencodes the query
    async fn search_page(
        &'a self,
        query: &'a String,
        page: usize,
    ) -> Result<Page<Map>, BeatSaverApiError<T>> {
        let query = encode(query.as_str());
        let url = BEATSAVER_URL
            .join(format!("api/search/text/{}?q={}", page, query).as_str())
            .unwrap();
        let data = self.request(url).await?;

        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves maps based on a specified search query, specifying a page number, iterable
    ///
    /// Note: urlencodes the query
    fn search_page_iter(
        &'a self,
        query: &'a String,
        page: usize,
    ) -> Pin<Box<dyn Stream<Item = Result<Map, BeatSaverApiError<T>>> + 'a>>
    where
        Self: Send + Sync,
    {
        iterate_page(move |p| self.search_page(query, p), page)
    }
    /// Retrieves maps based on an advanced search query
    ///
    /// Note: urlencodes the query
    ///
    /// Advanced queries use [Apache Lucene](https://lucene.apache.org/core/2_9_4/queryparsersyntax.html) syntax
    fn search_advanced(
        &'a self,
        query: &'a String,
    ) -> Pin<Box<dyn Stream<Item = Result<Map, BeatSaverApiError<T>>> + 'a>>
    where
        Self: Send + Sync,
    {
        self.search_advanced_page_iter(query, 0)
    }
    /// Retrieves maps based on an advanced search query, specifying a page number
    ///
    /// Note: urlencodes the query
    ///
    /// Advanced queries use [Apache Lucene](https://lucene.apache.org/core/2_9_4/queryparsersyntax.html) syntax
    async fn search_advanced_page(
        &'a self,
        query: &'a String,
        page: usize,
    ) -> Result<Page<Map>, BeatSaverApiError<T>> {
        // TODO: Validate Lucene syntax
        let query = encode(query.as_str());
        let url = BEATSAVER_URL
            .join(format!("api/search/advanced/{}?q={}", page, query).as_str())
            .unwrap();
        let data = self.request(url).await?;

        Ok(serde_json::from_str(data.as_str())?)
    }
    /// Retrieves maps based on an advanced search query, specifying a page number, iterable
    ///
    /// Note: urlencodes the query
    ///
    /// Advanced queries use [Apache Lucene](https://lucene.apache.org/core/2_9_4/queryparsersyntax.html) syntax
    fn search_advanced_page_iter(
        &'a self,
        query: &'a String,
        page: usize,
    ) -> Pin<Box<dyn Stream<Item = Result<Map, BeatSaverApiError<T>>> + 'a>>
    where
        Self: Send + Sync,
    {
        iterate_page(move |p| self.search_advanced_page(query, p), page)
    }
    /// Downloads a provided map
    ///
    /// [Maps][crate::map::Map] can be converted to [MapIds][crate::MapId] using the [Into][std::convert::Into] trait.
    async fn download(&'a self, id: MapId) -> Result<Bytes, BeatSaverApiError<T>> {
        let url = BEATSAVER_URL
            .join(
                match id {
                    MapId::Key(k) => format!("api/download/key/{:x}", k),
                    MapId::Hash(h) => format!("api/download/hash/{}", h),
                }
                .as_str(),
            )
            .unwrap();
        Ok(self.request_raw(url.clone()).await?)
    }
}

#[cfg(test)]
mod tests;
