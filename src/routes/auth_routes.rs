use actix_web::{
    cookie::{Cookie, SameSite, time::Duration as CookieDuration},
    http::{StatusCode, header},
    web, HttpRequest, HttpResponse,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};

use crate::structs::AppState;

const SESSION_COOKIE: &str = "clottis_session";
const SESSION_DAYS: i64 = 7;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct User {
    id: String,
    name: String,
    email: String,
    role: String,
    is_active: bool,
}
#[derive(Deserialize)]
struct LoginBody { email: String, password: String }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PasswordBody { current_password: String, new_password: String }
#[derive(Deserialize)]
struct CreateUserBody { name: String, email: String, role: String }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StatusBody { is_active: bool }

fn failure(code: StatusCode, message: &str) -> HttpResponse {
    HttpResponse::build(code).json(serde_json::json!({"success":false,"error":{"message":message}}))
}
fn db_failure(_: sqlx::Error) -> HttpResponse {
    failure(StatusCode::SERVICE_UNAVAILABLE, "Database service unavailable")
}
fn success(value: impl Serialize) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"success":true,"data":value}))
}
fn from_row(row: &sqlx::postgres::PgRow) -> User {
    User {
        id: row.get("id"),
        name: row.get("name"),
        email: row.get("email"),
        role: row.get("role"),
        is_active: row.get("is_active"),
    }
}
fn valid_email(value: &str) -> bool {
    let mut parts = value.split('@');
    matches!((parts.next(), parts.next(), parts.next()),
        (Some(local), Some(domain), None) if !local.is_empty() && domain.contains('.') && !value.contains(char::is_whitespace))
}
fn valid_password(value: &str) -> bool {
    value.len() >= 12 && value.len() <= 72
}
fn ensure_write(req: &HttpRequest, state: &AppState) -> Result<(), HttpResponse> {
    if req.headers().get("X-Requested-With").and_then(|h| h.to_str().ok()) != Some("XMLHttpRequest") {
        return Err(failure(StatusCode::FORBIDDEN, "Missing request verification header"));
    }
    if let Some(origin) = req.headers().get(header::ORIGIN) {
        if origin.to_str().ok() != Some(state.config.cors_allowed_origin.as_str()) {
            return Err(failure(StatusCode::FORBIDDEN, "Invalid request origin"));
        }
    }
    Ok(())
}
async fn authenticated(req: &HttpRequest, db: &PgPool) -> Result<User, HttpResponse> {
    let token = req.cookie(SESSION_COOKIE).map(|c| c.value().to_owned())
        .ok_or_else(|| failure(StatusCode::UNAUTHORIZED, "Authentication required"))?;
    let row = sqlx::query(
        "SELECT u.id::text AS id,u.name,u.email,u.role,u.is_active
         FROM auth_sessions s JOIN auth_users u ON u.id=s.user_id
         WHERE s.token_hash=encode(digest($1,'sha256'),'hex')
           AND s.expires_at>now() AND u.is_active=true"
    ).bind(token).fetch_optional(db).await.map_err(db_failure)?;
    row.as_ref().map(from_row)
        .ok_or_else(|| failure(StatusCode::UNAUTHORIZED, "Authentication required"))
}
fn require_admin(user: &User) -> Result<(), HttpResponse> {
    if user.role == "owner" || user.role == "admin" { Ok(()) }
    else { Err(failure(StatusCode::FORBIDDEN, "Insufficient permissions")) }
}
fn can_manage(actor: &User, target: &User) -> bool {
    actor.id != target.id && (actor.role == "owner" || target.role != "owner")
}
fn cookie(token: String, secure: bool) -> Cookie<'static> {
    Cookie::build(SESSION_COOKIE, token).path("/")
        .http_only(true).secure(secure).same_site(SameSite::Lax)
        .max_age(CookieDuration::days(SESSION_DAYS)).finish()
}
fn is_production(state: &AppState) -> bool {
    matches!(state.config.environment, crate::configs::Environment::Production)
}
async fn login(req: HttpRequest, state: web::Data<AppState>, body: web::Json<LoginBody>) -> HttpResponse {
    if let Err(e) = ensure_write(&req, &state) { return e; }
    let email = body.email.trim().to_lowercase();
    let row = match sqlx::query(
        "SELECT id::text AS id,name,email,role,is_active FROM auth_users
         WHERE email=$1 AND is_active=true AND password_hash=crypt($2,password_hash)"
    ).bind(email).bind(&body.password).fetch_optional(&state.db).await {
        Ok(r) => r, Err(e) => return db_failure(e),
    };
    let Some(row) = row else { return failure(StatusCode::UNAUTHORIZED, "Invalid email or password"); };
    let user = from_row(&row);
    let token: String = match sqlx::query_scalar("SELECT encode(gen_random_bytes(32),'hex')")
        .fetch_one(&state.db).await { Ok(v) => v, Err(e) => return db_failure(e) };
    if let Err(e) = sqlx::query(
        "INSERT INTO auth_sessions(user_id, token_hash, expires_at)
         VALUES ($1::uuid,encode(digest($2,'sha256'),'hex'),now()+interval '7 days')"
    ).bind(&user.id).bind(&token).execute(&state.db).await { return db_failure(e); }
    HttpResponse::Ok().cookie(cookie(token, is_production(&state)))
        .json(serde_json::json!({"success":true,"data":{"user":user}}))
}
async fn logout(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    if let Err(e) = ensure_write(&req, &state) { return e; }
    if let Some(c) = req.cookie(SESSION_COOKIE) {
        if let Err(e) = sqlx::query(
            "DELETE FROM auth_sessions WHERE token_hash=encode(digest($1,'sha256'),'hex')"
        ).bind(c.value()).execute(&state.db).await { return db_failure(e); }
    }
    let mut expired = cookie(String::new(), is_production(&state));
    expired.make_removal();
    HttpResponse::Ok().cookie(expired).json(serde_json::json!({"success":true,"data":{}}))
}
async fn me(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    match authenticated(&req, &state.db).await {
        Ok(user) => success(serde_json::json!({"user":user})),
        Err(e) => e,
    }
}
async fn change_password(req: HttpRequest, state: web::Data<AppState>, body: web::Json<PasswordBody>) -> HttpResponse {
    if let Err(e) = ensure_write(&req, &state) { return e; }
    let user = match authenticated(&req, &state.db).await { Ok(u) => u, Err(e) => return e };
    if !valid_password(&body.new_password) {
        return failure(StatusCode::BAD_REQUEST, "Password must contain 12–72 bytes");
    }
    let changed = match sqlx::query(
        "UPDATE auth_users SET password_hash=crypt($1,gen_salt('bf',12))
         WHERE id=$2::uuid AND password_hash=crypt($3,password_hash)"
    ).bind(&body.new_password).bind(&user.id).bind(&body.current_password)
        .execute(&state.db).await { Ok(r) => r.rows_affected(), Err(e) => return db_failure(e) };
    if changed == 0 { return failure(StatusCode::BAD_REQUEST, "Current password is incorrect"); }
    if let Some(c) = req.cookie(SESSION_COOKIE) {
        if let Err(e) = sqlx::query(
            "DELETE FROM auth_sessions WHERE user_id=$1::uuid
             AND token_hash<>encode(digest($2,'sha256'),'hex')"
        ).bind(&user.id).bind(c.value()).execute(&state.db).await { return db_failure(e); }
    }
    success(serde_json::json!({}))
}
async fn list_users(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let user = match authenticated(&req,&state.db).await { Ok(u)=>u,Err(e)=>return e };
    if let Err(e)=require_admin(&user){ return e; }
    let rows=match sqlx::query("SELECT id::text AS id,name,email,role,is_active FROM auth_users ORDER BY created_at DESC LIMIT 200")
        .fetch_all(&state.db).await { Ok(r)=>r,Err(e)=>return db_failure(e) };
    success(serde_json::json!({"users":rows.iter().map(from_row).collect::<Vec<_>>()}))
}
async fn create_user(req: HttpRequest, state: web::Data<AppState>, body: web::Json<CreateUserBody>) -> HttpResponse {
    if let Err(e)=ensure_write(&req,&state){return e;}
    let actor=match authenticated(&req,&state.db).await{Ok(u)=>u,Err(e)=>return e};
    if let Err(e)=require_admin(&actor){return e;}
    let email=body.email.trim().to_lowercase();
    if !valid_email(&email) || body.name.trim().chars().count()<2 || body.name.trim().chars().count()>200
        || !matches!(body.role.as_str(),"owner"|"admin"|"member")
        || (body.role=="owner" && actor.role!="owner") {
        return failure(StatusCode::BAD_REQUEST,"Invalid name, email or role");
    }
    let password:String=match sqlx::query_scalar("SELECT encode(gen_random_bytes(20),'hex')")
        .fetch_one(&state.db).await{Ok(x)=>x,Err(e)=>return db_failure(e)};
    let result=sqlx::query(
        "INSERT INTO auth_users(name,email,password_hash,role) VALUES
         ($1,$2,crypt($3,gen_salt('bf',12)),$4)
         RETURNING id::text AS id,name,email,role,is_active"
    ).bind(body.name.trim()).bind(&email).bind(&password).bind(&body.role)
        .fetch_one(&state.db).await;
    match result {
        Ok(row)=>success(serde_json::json!({"user":from_row(&row),"initialPassword":password})),
        Err(sqlx::Error::Database(ref e)) if e.is_unique_violation()=>
            failure(StatusCode::CONFLICT,"A user with this email already exists"),
        Err(e)=>db_failure(e)
    }
}
async fn get_target(db:&PgPool,id:&str)->Result<Option<User>,HttpResponse>{
    let row=sqlx::query(
        "SELECT id::text AS id,name,email,role,is_active FROM auth_users WHERE id::text=$1"
    ).bind(id).fetch_optional(db).await.map_err(db_failure)?;
    Ok(row.as_ref().map(from_row))
}
async fn reset_password(req:HttpRequest,state:web::Data<AppState>,path:web::Path<String>)->HttpResponse{
    if let Err(e)=ensure_write(&req,&state){return e;}
    let actor=match authenticated(&req,&state.db).await{Ok(u)=>u,Err(e)=>return e};
    if let Err(e)=require_admin(&actor){return e;}
    let target=match get_target(&state.db,&path).await{Ok(Some(u))=>u,Ok(None)=>return failure(StatusCode::NOT_FOUND,"User not found"),Err(e)=>return e};
    if !can_manage(&actor,&target){return failure(StatusCode::FORBIDDEN,"Cannot manage this user");}
    let password:String=match sqlx::query_scalar("SELECT encode(gen_random_bytes(20),'hex')").fetch_one(&state.db).await{
        Ok(v)=>v,Err(e)=>return db_failure(e)
    };
    let mut tx=match state.db.begin().await{Ok(t)=>t,Err(e)=>return db_failure(e)};
    if let Err(e)=sqlx::query("UPDATE auth_users SET password_hash=crypt($1,gen_salt('bf',12)) WHERE id=$2::uuid")
        .bind(&password).bind(&target.id).execute(&mut *tx).await{return db_failure(e);}
    if let Err(e)=sqlx::query("DELETE FROM auth_sessions WHERE user_id=$1::uuid")
        .bind(&target.id).execute(&mut *tx).await{return db_failure(e);}
    if let Err(e)=tx.commit().await{return db_failure(e);}
    success(serde_json::json!({"user":target,"newPassword":password}))
}
async fn set_status(req:HttpRequest,state:web::Data<AppState>,path:web::Path<String>,body:web::Json<StatusBody>)->HttpResponse{
    if let Err(e)=ensure_write(&req,&state){return e;}
    let actor=match authenticated(&req,&state.db).await{Ok(u)=>u,Err(e)=>return e};
    if let Err(e)=require_admin(&actor){return e;}
    let target=match get_target(&state.db,&path).await{Ok(Some(u))=>u,Ok(None)=>return failure(StatusCode::NOT_FOUND,"User not found"),Err(e)=>return e};
    if !can_manage(&actor,&target){return failure(StatusCode::FORBIDDEN,"Cannot manage this user");}
    let mut tx=match state.db.begin().await{Ok(t)=>t,Err(e)=>return db_failure(e)};
    if let Err(e)=sqlx::query("UPDATE auth_users SET is_active=$1 WHERE id=$2::uuid")
        .bind(body.is_active).bind(&target.id).execute(&mut *tx).await{return db_failure(e);}
    if !body.is_active {
        if let Err(e)=sqlx::query("DELETE FROM auth_sessions WHERE user_id=$1::uuid")
            .bind(&target.id).execute(&mut *tx).await{return db_failure(e);}
    }
    if let Err(e)=tx.commit().await{return db_failure(e);}
    success(serde_json::json!({"isActive":body.is_active}))
}
pub fn configure(cfg:&mut web::ServiceConfig){
    cfg.service(web::scope("/auth")
        .route("/login",web::post().to(login))
        .route("/logout",web::post().to(logout))
        .route("/me",web::get().to(me))
        .route("/change-password",web::post().to(change_password)));
    cfg.service(web::scope("/users")
        .route("",web::get().to(list_users))
        .route("",web::post().to(create_user))
        .route("/{id}/reset-password",web::post().to(reset_password))
        .route("/{id}/status",web::patch().to(set_status)));
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_password_length() {
        assert!(!valid_password("short"));
        assert!(valid_password("a-long-password"));
        assert!(!valid_password(&"x".repeat(73)));
    }
    #[test]
    fn enforces_owner_role_limits() {
        let user=|id:&str,role:&str|User{id:id.into(),name:"Person".into(),email:"a@b.com".into(),role:role.into(),is_active:true};
        assert!(!can_manage(&user("1","admin"),&user("2","owner")));
        assert!(!can_manage(&user("1","owner"),&user("1","member")));
        assert!(can_manage(&user("1","owner"),&user("2","admin")));
    }
}
