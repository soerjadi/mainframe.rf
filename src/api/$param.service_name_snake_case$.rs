//! Koleksi query yang digunakan untuk operasi pada rest API.
#![allow(missing_docs)]

use actix_web::{HttpRequest, HttpResponse};
use chrono::NaiveDateTime;
// <% if param.with_protobuf %>
use protobuf;
// <% endif %>
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::crypto::{self, PublicKey, SecretKey, Signature};

use crate::{
    api,
    api::types::*,
    api::{ApiResult, Error as ApiError, HttpRequest as ApiHttpRequest},
    auth,
    error::{Error, ErrorCode},
    prelude::*,
    $param.service_name_snake_case$_dao,
    ID
};

/// Definisi query untuk mendaftarkan akun baru via rest API.
#[derive(Debug, Serialize, Deserialize)]
pub struct Register$param.service_name_pascal_case$ {
    pub full_name: String,
    pub email: String,
    pub phone_num: String,
    // comment out: mungkin tidak untuk sekarang
    // pub nik: String,
}

/// Definisi query untuk mengaktifkan akun yang telah didaftarkan.
#[derive(Debug, Serialize, Deserialize)]
pub struct Activate$param.service_name_pascal_case$ {
    pub token: String,
    pub password: String,
}

// <% if param.with_protobuf %>
/// Setiap query transaksi harus menggunakan wrapper ini,
/// masuk pada `body` dan signature dari `body` disimpan pada field `signature`.
///
/// TxQuery implement `sign` method yang bisa digunakan untuk melakukan signing
/// pada data di `body`.
#[derive(Debug, Serialize, Deserialize)]
pub struct TxQuery<T>
where
    T: Serialize,
{
    pub body: T,
    pub signature: String,
}

impl<T> TxQuery<T>
where
    T: protobuf::Message + Serialize + Clone,
{
    /// Create new tx query instance.
    pub fn new(body: T) -> Self {
        Self {
            body,
            signature: Default::default(),
        }
    }

    /// Lakukan signing pada data di `body`.
    /// Operasi signing dilakukan dengan cara men-serialize data pada `body` ke dalam
    /// bentuk protobuf bytes lalu di-sign menggunakan `secret_key`.
    pub fn sign(&self, secret_key: &SecretKey) -> Self {
        assert!(self.signature.is_empty(), "already signed.");

        // convert ke bytes format protobuf
        let bytes = self.body.write_to_bytes().expect("Cannot write to bytes");
        let signature = crypto::sign(&bytes, &secret_key);
        Self {
            body: self.body.clone(),
            signature: signature.to_hex(),
        }
    }

    /// Untuk memverifikasi signature pada body ini.
    pub fn verify(&self, public_key: &PublicKey, secret_key: &SecretKey) -> Result<()> {
        if self.signature.is_empty() {
            Err(Error::BadRequest(
                ErrorCode::MessageHasNoSign as i32,
                "message has no signature.".to_string(),
            ))?
        }
        let bytes = self.body.write_to_bytes().expect("Cannot write to bytes");
        let signature: Signature = self.signature.parse::<Signature>()?;
        debug!("verify sig `{}` using pubkey: `{}`", &signature, public_key);
        if !crypto::is_verified(bytes.as_slice(), &signature, public_key) {
            let real_signature = crypto::sign(&bytes, &secret_key);
            debug!("  - expected signature: `{}`", real_signature);
            debug!("  - data: `{}`", hex::encode(bytes));
            Err(Error::Unauthorized)?
        }
        Ok(())
    }
}
// <% endif %>

/// Model untuk keperluan tukar menukar data API
/// bukan yang di database (crate::models).
pub mod types {

    use chrono::NaiveDateTime;

    use crate::{api::ApiResult, models};

    use std::convert::From;

    /// Bentuk model akun di dalam database.
    #[derive(Clone, Serialize, Deserialize, PartialEq)]
    pub struct $param.service_name_pascal_case$ {
        /// ID dari akun.
        pub id: i64,

        /// Nama lengkap akun.
        pub full_name: String,

        /// Alamat email kun.
        pub email: String,

        /// Nomor telpon akun.
        pub phone_num: String,

        /// Waktu kapan akun ini didaftarkan.
        pub register_time: NaiveDateTime,
    }

    impl From<models::$param.service_name_pascal_case$> for $param.service_name_pascal_case$ {
        fn from(a: models::$param.service_name_pascal_case$) -> Self {
            $param.service_name_pascal_case$ {
                id: a.id,
                full_name: a.full_name,
                email: a.email,
                phone_num: a.phone_num,
                register_time: a.register_time,
            }
        }
    }

    impl From<models::$param.service_name_pascal_case$> for ApiResult<$param.service_name_pascal_case$> {
        fn from(a: models::$param.service_name_pascal_case$) -> Self {
            ApiResult::success($param.service_name_pascal_case$ {
                id: a.id,
                full_name: a.full_name,
                email: a.email,
                phone_num: a.phone_num,
                register_time: a.register_time,
            })
        }
    }

}


#[derive(Deserialize)]
pub struct UpdatePassword {
    pub old_password: String,
    pub new_password: String,
    pub verif_new_password: String,
}

use crate::models::AccessToken;

/// Holder untuk implementasi API endpoint publik untuk $param.service_name_snake_case$.
pub struct PublicApi;

#[api_group("$param.service_name_pascal_case$", "public", base="/$param.service_name_snake_case$/v1")]
impl PublicApi {


    // <% if param.with_protobuf %>
    #[inline]
    fn verify_tx<T>(query: &TxQuery<T>, schema: &$param.service_name_pascal_case$Dao, current_$param.service_name_snake_case$: &db::$param.service_name_pascal_case$) -> api::Result<()>
    where
        T: Serialize + protobuf::Message + Clone,
    {
        // verifikasi digital signature
        let acc_key = schema.get_$param.service_name_snake_case$_key(current_$param.service_name_snake_case$.id)?;
        let secret_key = acc_key.secret_key.parse::<SecretKey>()?;
        let public_key = acc_key.pub_key.parse::<PublicKey>()?;

        query.verify(&public_key, &secret_key)?;

        Ok(())
    }
    // <% endif %>


    /// Rest API endpoint untuk mendaftarkan akun baru.
    /// Setelah register akun tidak langsung aktif, perlu melakukan
    /// aktifasi menggunakan endpoint `/$param.service_name_snake_case$/activate`.
    #[api_endpoint(path = "/$param.service_name_snake_case$/register", mutable, auth = "none")]
    pub fn register_$param.service_name_snake_case$(query: Register$param.service_name_pascal_case$) -> ApiResult<String> {
        let conn = state.db();
        let schema = $param.service_name_pascal_case$Dao::new(&conn);

        schema
            .register_$param.service_name_snake_case$(&query.full_name, &query.email, &query.phone_num)
            .map_err(From::from)
            .map(ApiResult::success)
    }

    /// Mengaktifkan user yang telah teregister.
    /// Ini nantinya dijadikan link yang akan dikirimkan ke email pendaftar.
    #[api_endpoint(path = "/$param.service_name_snake_case$/activate", auth = "none", mutable)]
    pub fn activate_$param.service_name_snake_case$(query: Activate$param.service_name_pascal_case$) -> ApiResult<types::$param.service_name_pascal_case$> {
        let conn = state.db();
        let schema = $param.service_name_pascal_case$Dao::new(&conn);
        let $param.service_name_snake_case$ = schema.activate_registered_$param.service_name_snake_case$(query.token)?;
        schema.set_password($param.service_name_snake_case$.id, &query.password)?;
        Ok($param.service_name_snake_case$.into())
    }

    /// Mendapatkan informasi current $param.service_name_snake_case$.
    #[api_endpoint(path = "/me/info", auth = "required")]
    pub fn me_info(state: &AppState, query: (), req: &ApiHttpRequest) -> ApiResult<types::$param.service_name_pascal_case$> {
        Ok(ApiResult::success(current_$param.service_name_snake_case$.into()))
    }

    /// Update password.
    #[api_endpoint(path = "/update_password", auth = "required", mutable)]
    pub fn update_password(query: UpdatePassword) -> ApiResult<()> {
        let conn = state.db();
        let dao = $param.service_name_pascal_case$Dao::new(&conn);

        if query.new_password.len() < 6 {
            param_error("New password too short, please use min 6 characters long")?;
        }

        if query.new_password != query.verif_new_password {
            param_error("Password verification didn't match")?;
        }

        let auth_dao = auth::Schema::new(&conn);

        let $param.service_name_snake_case$_passhash = auth_dao.get_passhash(current_$param.service_name_snake_case$.id)?;
        if !crypto::password_match(&query.old_password, &$param.service_name_snake_case$_passhash) {
            warn!(
                "$param.service_name_snake_case$ `{}` try to update password using wrong password",
                &current_$param.service_name_snake_case$.id
            );
            Err(ApiError::Unauthorized)?
        }

        dao.set_password(current_$param.service_name_snake_case$.id, &query.new_password)?;

        Ok(ApiResult::success(()))
    }
}

use crate::models as db;

/// Holder untuk implementasi API endpoint privat.
pub struct PrivateApi;

#[api_group("$param.service_name_pascal_case$", "private", base = "/$param.service_name_snake_case$/v1")]
impl PrivateApi {

    /// Listing $param.service_name_snake_case$
    #[api_endpoint(path = "/$param.service_name_snake_case$s", auth = "none")]
    pub fn list_$param.service_name_snake_case$(query: QueryEntries) -> ApiResult<EntriesResult<db::$param.service_name_pascal_case$>> {
        let conn = state.db();
        let dao = $param.service_name_pascal_case$Dao::new(&conn);

        let entries = dao.get_$param.service_name_snake_case$s(query.offset, query.limit)?;

        let count = dao.count()?;
        Ok(ApiResult::success(EntriesResult { count, entries }))
    }

    /// Mencari akun berdasarkan kata kunci.
    #[api_endpoint(path = "/$param.service_name_snake_case$/search", auth = "none")]
    pub fn search_$param.service_name_snake_case$s(query: QueryEntries) -> ApiResult<EntriesResult<db::$param.service_name_pascal_case$>> {
        let conn = state.db();
        let dao = $param.service_name_pascal_case$Dao::new(&conn);

        if query.query.is_none() {
            return Self::list_$param.service_name_snake_case$(&state, query, req);
        }

        let keyword = query.query.unwrap();

        let (entries, count) = dao.search(&keyword, query.offset, query.limit)?;

        Ok(ApiResult::success(EntriesResult { count, entries }))
    }

    /// Mendapatkan jumlah akun secara keseluruhan.
    #[api_endpoint(path = "/$param.service_name_snake_case$/count")]
    pub fn $param.service_name_snake_case$_count(state: &AppState, query: ()) -> ApiResult<i64> {
        let conn = state.db();
        let dao = $param.service_name_pascal_case$Dao::new(&conn);

        dao
            .count()
            .map(ApiResult::success)
            .map_err(From::from)
    }

    /// Mendapatkan data akun.
    #[api_endpoint(path = "/$param.service_name_snake_case$/info", auth = "required")]
    pub fn $param.service_name_snake_case$_info(query: IdQuery) -> ApiResult<db::$param.service_name_pascal_case$> {
        let conn = state.db();
        let dao = $param.service_name_pascal_case$Dao::new(&conn);

        dao
            .get_by_id(query.id)
            .map(ApiResult::success)
            .map_err(From::from)
    }

}
