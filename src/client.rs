use crate::endpoints::access_management::{
    ApiVersion, ApiVersionResponse, GetServices, GetServicesResponse, SessionKey,
    SessionKeyResponse, VerifyAccessKey, VerifyAccessKeyResponse,
};
use crate::endpoints::adding_files::{
    AddFile, AddFileRequest, AddFileResponse, ArchiveFiles, ArchiveFilesRequest, DeleteFiles,
    DeleteFilesRequest, UnarchiveFiles, UnarchiveFilesRequest, UndeleteFiles, UndeleteFilesRequest,
};
use crate::endpoints::adding_tags::{AddTags, AddTagsRequest, CleanTags, CleanTagsResponse};
use crate::endpoints::Endpoint;
use crate::error::{Error, Result};
use crate::utils::string_list_to_json_array;
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::Serialize;

static ACCESS_KEY_HEADER: &str = "Hydrus-Client-API-Access-Key";

pub struct Client {
    inner: reqwest::Client,
    base_url: String,
    access_key: String,
}

impl Client {
    /// Creates a new client to start requests against the hydrus api.
    pub fn new<S: AsRef<str>>(url: S, access_key: S) -> Result<Self> {
        Ok(Self {
            inner: reqwest::Client::new(),
            access_key: access_key.as_ref().to_string(),
            base_url: url.as_ref().to_string(),
        })
    }

    /// Starts a get request to the path associated with the return type
    async fn get_and_parse<E: Endpoint, Q: Serialize + ?Sized>(
        &mut self,
        query: &Q,
    ) -> Result<E::Response> {
        let response = self
            .inner
            .get(format!("{}/{}", self.base_url, E::get_path()))
            .header(ACCESS_KEY_HEADER, &self.access_key)
            .query(query)
            .send()
            .await?;
        let response = Self::extract_error(response).await?;

        Self::extract_content(response).await
    }

    /// Stats a post request to the path associated with the return type
    async fn post<E: Endpoint>(&mut self, body: E::Request) -> Result<Response> {
        let response = self
            .inner
            .post(format!("{}/{}", self.base_url, E::get_path()))
            .json(&body)
            .header(ACCESS_KEY_HEADER, &self.access_key)
            .send()
            .await?;
        let response = Self::extract_error(response).await?;
        Ok(response)
    }

    /// Stats a post request and parses the body as json
    async fn post_and_parse<E: Endpoint>(&mut self, body: E::Request) -> Result<E::Response> {
        let response = self.post::<E>(body).await?;

        Self::extract_content(response).await
    }

    /// Stats a post request to the path associated with the return type
    async fn post_binary<E: Endpoint>(&mut self, data: Vec<u8>) -> Result<E::Response> {
        let response = self
            .inner
            .post(format!("{}/{}", self.base_url, E::get_path()))
            .body(data)
            .header(ACCESS_KEY_HEADER, &self.access_key)
            .header("Content-Type", "application/octet-stream")
            .send()
            .await?;
        let response = Self::extract_error(response).await?;

        Self::extract_content(response).await
    }

    /// Returns an error with the response text content if the status doesn't indicate success
    async fn extract_error(response: Response) -> Result<Response> {
        if !response.status().is_success() {
            let msg = response.text().await?;
            Err(Error::Hydrus(msg))
        } else {
            Ok(response)
        }
    }

    /// Parses the response as JSOn
    async fn extract_content<T: DeserializeOwned>(response: Response) -> Result<T> {
        response.json::<T>().await.map_err(Error::from)
    }

    /// Returns the current API version. It's being incremented every time the API changes.
    pub async fn api_version(&mut self) -> Result<ApiVersionResponse> {
        self.get_and_parse::<ApiVersion, ()>(&()).await
    }

    /// Creates a new session key
    pub async fn session_key(&mut self) -> Result<SessionKeyResponse> {
        self.get_and_parse::<SessionKey, ()>(&()).await
    }

    /// Verifies if the access key is valid and returns some information about its permissions
    pub async fn verify_access_key(&mut self) -> Result<VerifyAccessKeyResponse> {
        self.get_and_parse::<VerifyAccessKey, ()>(&()).await
    }

    /// Returns the list of tag and file services of the client
    pub async fn get_services(&mut self) -> Result<GetServicesResponse> {
        self.get_and_parse::<GetServices, ()>(&()).await
    }

    /// Adds a file to hydrus
    pub async fn add_file<S: AsRef<str>>(&mut self, path: S) -> Result<AddFileResponse> {
        self.post_and_parse::<AddFile>(AddFileRequest {
            path: path.as_ref().to_string(),
        })
        .await
    }

    /// Adds a file from binary data to hydrus
    pub async fn add_binary_file(&mut self, data: Vec<u8>) -> Result<AddFileResponse> {
        self.post_binary::<AddFile>(data).await
    }

    /// Moves files with matching hashes to the trash
    pub async fn delete_files(&mut self, hashes: Vec<String>) -> Result<()> {
        self.post::<DeleteFiles>(DeleteFilesRequest { hashes })
            .await?;

        Ok(())
    }

    /// Pulls files out of the trash by hash
    pub async fn undelete_files(&mut self, hashes: Vec<String>) -> Result<()> {
        self.post::<UndeleteFiles>(UndeleteFilesRequest { hashes })
            .await?;

        Ok(())
    }

    /// Moves files from the inbox into the archive
    pub async fn archive_files(&mut self, hashes: Vec<String>) -> Result<()> {
        self.post::<ArchiveFiles>(ArchiveFilesRequest { hashes })
            .await?;

        Ok(())
    }

    /// Moves files from the archive into the inbox
    pub async fn unarchive_files(&mut self, hashes: Vec<String>) -> Result<()> {
        self.post::<UnarchiveFiles>(UnarchiveFilesRequest { hashes })
            .await?;

        Ok(())
    }

    /// Returns the list of tags as the client would see them in a human friendly order
    pub async fn clean_tags(&mut self, tags: Vec<String>) -> Result<CleanTagsResponse> {
        self.get_and_parse::<CleanTags, [(&str, String)]>(&[(
            "tags",
            string_list_to_json_array(tags),
        )])
        .await
    }

    /// Adds tags to files with the given hashes
    pub async fn add_tags(&mut self, request: AddTagsRequest) -> Result<()> {
        self.post::<AddTags>(request).await?;

        Ok(())
    }
}
