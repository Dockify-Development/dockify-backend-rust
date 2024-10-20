/*
    This source file is a part of Dockify
    Dockify is licensed under the Server Side Public License (SSPL), Version 1.
    Find the LICENSE file in the root of this repository for more details.
*/

use crate::utils::{
    db,
    validation::{self, validate_email},
};
use axum::http::StatusCode;
use axum::Json;
use axum::{response::IntoResponse, routing::post, Router};
use base64::{engine::general_purpose, Engine as _};
use chrono::Duration;
use chrono::Utc;
use dotenvy::var;
use lettre::message::{header, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Transport};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct AuthPayload {
    email: String,
    username: String,
    password: String,
}
static SMTP_USER: Lazy<String> =
    Lazy::new(|| var("SMTP_USER").expect("Failed to retrieve SMTP_USER"));
static SMTP_PASS: Lazy<String> =
    Lazy::new(|| var("SMTP_PASS").expect("Failed to retrieve SMTP_PASS"));
static CREDS: Lazy<Credentials> =
    Lazy::new(|| Credentials::new(SMTP_USER.clone(), SMTP_PASS.clone()));

static MAILER: Lazy<SmtpTransport> = Lazy::new(|| {
    SmtpTransport::relay("mail.smtp2go.com")
        .expect("Failed to connect to SMTP relay")
        .credentials(CREDS.clone())
        .build()
});

pub async fn handler(Json(payload): Json<AuthPayload>) -> impl IntoResponse {
    let username = payload.username;
    let email = payload.email;
    if !&username.is_ascii() || !validate_email(&email) {
        return StatusCode::BAD_REQUEST;
    }
    let checks = vec![
        (
            db::check_exists(&username.to_lowercase(), "username", "users"),
            "username",
        ),
        (
            db::check_exists(&email.to_lowercase(), "email", "users"),
            "email",
        ),
    ];

    for (result, field) in checks {
        match result {
            Ok(exists) if exists => return StatusCode::CONFLICT,
            Err(e) => {
                eprintln!("An error occurred while checking {}: {}", field, e);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            _ => {}
        }
    }
    let hash = match validation::hash_password(payload.password) {
        Ok(hash) => hash,
        Err(err) => {
            eprintln!("An error occurred while hashing: {}", err);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    match db::insert_user(&email, &username, &hash, false) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("An error occurred while inserting user: {}", err);
        }
    }

    let verification_code =
        match validation::generate_jwt(username.clone(), Utc::now() + Duration::hours(1)) {
            Ok(jwt) => jwt,
            Err(err) => {
                eprintln!(
                    "An error occurred while generating verification jwt: {}",
                    err
                );
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        };
    match db::insert_code(&username, &verification_code) {
        Ok(_) => (),
        Err(err) => {
            eprintln!(
                "An error occurred while inserting verification jwt into db: {}",
                err
            );
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }
    let email = match Message::builder()
        .from(match "account@dockify.xyz".parse() {
            Ok(addr) => addr,
            Err(e) => {
                eprintln!("Error parsing 'from' address: {:?}", e);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        })
        .to(match email.parse() {
            Ok(addr) => addr,
            Err(e) => {
                eprintln!("Error parsing 'to' address: {:?}", e);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        })
        .subject("Your Dockify Verification Email")
        .header(header::ContentType::TEXT_HTML)
        .body(String::from(format!(
            "<html><head><meta http-equiv=\"x-ua-compatible\" content=\"ie=edge\"><meta name=\"x-apple-disable-message-reformatting\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta name=\"format-detection\" content=\"telephone=no, date=no, address=no, email=no\"><meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\"><style type=\"text/css\">body,table,td{{font-family:Helvetica,Arial,sans-serif!important}}.ExternalClass{{width:100%}}.ExternalClass,.ExternalClass div,.ExternalClass font,.ExternalClass p,.ExternalClass span,.ExternalClass td{{line-height:150%}}a{{text-decoration:none}}*{{color:inherit}}#MessageViewBody a,a[x-apple-data-detectors],u+#body a{{color:inherit;text-decoration:none;font-size:inherit;font-family:inherit;font-weight:inherit;line-height:inherit}}img{{-ms-interpolation-mode:bicubic}}table:not([class^=s-]){{font-family:Helvetica,Arial,sans-serif;mso-table-lspace:0;mso-table-rspace:0;border-spacing:0;border-collapse:collapse}}table:not([class^=s-]) td{{border-spacing:0;border-collapse:collapse}}@media screen and (max-width:600px){{.w-full,.w-full>tbody>tr>td{{width:100%!important}}[class*=s-lg-]>tbody>tr>td{{font-size:0!important;line-height:0!important;height:0!important}}.s-2>tbody>tr>td{{font-size:8px!important;line-height:8px!important;height:8px!important}}.s-5>tbody>tr>td{{font-size:20px!important;line-height:20px!important;height:20px!important}}.s-10>tbody>tr>td{{font-size:40px!important;line-height:40px!important;height:40px!important}}}}</style></head><body class=\"bg-light\" style=\"outline:0;width:100%;min-width:100%;height:100%;-webkit-text-size-adjust:100%;-ms-text-size-adjust:100%;font-family:Helvetica,Arial,sans-serif;line-height:24px;font-weight:400;font-size:16px;-moz-box-sizing:border-box;-webkit-box-sizing:border-box;box-sizing:border-box;color:#000;margin:0;padding:0;border-width:0\" bgcolor=\"#f7fafc\"><table class=\"bg-light body\" valign=\"top\" role=\"presentation\" border=\"0\" cellpadding=\"0\" cellspacing=\"0\" style=\"outline:0;width:100%;min-width:100%;height:100%;-webkit-text-size-adjust:100%;-ms-text-size-adjust:100%;font-family:Helvetica,Arial,sans-serif;line-height:24px;font-weight:400;font-size:16px;-moz-box-sizing:border-box;-webkit-box-sizing:border-box;box-sizing:border-box;color:#000;margin:0;padding:0;border-width:0\" bgcolor=\"#f7fafc\"><tbody><tr><td valign=\"top\" style=\"line-height:24px;font-size:16px;margin:0\" align=\"left\" bgcolor=\"#f7fafc\"><table class=\"container\" role=\"presentation\" border=\"0\" cellpadding=\"0\" cellspacing=\"0\" style=\"width:100%\"><tbody><tr><td align=\"center\" style=\"line-height:24px;font-size:16px;margin:0;padding:0 16px\"><!--[if (gte mso 9)|(IE)]><table align=\"center\" role=\"presentation\"><tbody><tr><td width=\"600\"><![endif]--><table align=\"center\" role=\"presentation\" border=\"0\" cellpadding=\"0\" cellspacing=\"0\" style=\"width:100%;max-width:600px;margin:0 auto\"><tbody><tr><td style=\"line-height:24px;font-size:16px;margin:0\" align=\"left\"><table class=\"s-10 w-full\" role=\"presentation\" border=\"0\" cellpadding=\"0\" cellspacing=\"0\" style=\"width:100%\" width=\"100%\"><tbody><tr><td style=\"line-height:40px;font-size:40px;width:100%;height:40px;margin:0\" align=\"left\" width=\"100%\" height=\"40\">&nbsp;</td></tr></tbody></table><table class=\"card\" role=\"presentation\" border=\"0\" cellpadding=\"0\" cellspacing=\"0\" style=\"border-radius:6px;border-collapse:separate!important;width:100%;overflow:hidden;border:1px solid #e2e8f0\" bgcolor=\"#ffffff\"><tbody><tr><td style=\"line-height:24px;font-size:16px;width:100%;margin:0\" align=\"left\" bgcolor=\"#ffffff\"><table class=\"card-body\" role=\"presentation\" border=\"0\" cellpadding=\"0\" cellspacing=\"0\" style=\"width:100%\"><tbody><tr><td style=\"line-height:24px;font-size:16px;width:100%;margin:0;padding:20px\" align=\"left\"><h1 class=\"h2\" style=\"padding-top:0;padding-bottom:0;font-weight:500;vertical-align:baseline;font-size:32px;line-height:38.4px;margin:0\" align=\"left\">Dockify Verify Email</h1><table class=\"s-2 w-full\" role=\"presentation\" border=\"0\" cellpadding=\"0\" cellspacing=\"0\" style=\"width:100%\" width=\"100%\"><tbody><tr><td style=\"line-height:8px;font-size:8px;width:100%;height:8px;margin:0\" align=\"left\" width=\"100%\" height=\"8\">&nbsp;</td></tr></tbody></table><h5 class=\"text-grey-700\" style=\"padding-top:0;padding-bottom:0;font-weight:500;vertical-align:baseline;font-size:20px;line-height:24px;margin:0\" align=\"left\">Click the verify button to continue.</h5><table class=\"s-5 w-full\" role=\"presentation\" border=\"0\" cellpadding=\"0\" cellspacing=\"0\" style=\"width:100%\" width=\"100%\"><tbody><tr><td style=\"line-height:20px;font-size:20px;width:100%;height:20px;margin:0\" align=\"left\" width=\"100%\" height=\"20\">&nbsp;</td></tr></tbody></table><table class=\"hr\" role=\"presentation\" border=\"0\" cellpadding=\"0\" cellspacing=\"0\" style=\"width:100%\"><tbody><tr><td style=\"line-height:24px;font-size:16px;border-top-width:1px;border-top-color:#e2e8f0;border-top-style:solid;height:1px;width:100%;margin:0\" align=\"left\"></td></tr></tbody></table><table class=\"s-5 w-full\" role=\"presentation\" border=\"0\" cellpadding=\"0\" cellspacing=\"0\" style=\"width:100%\" width=\"100%\"><tbody><tr><td style=\"line-height:20px;font-size:20px;width:100%;height:20px;margin:0\" align=\"left\" width=\"100%\" height=\"20\">&nbsp;</td></tr></tbody></table><div class=\"space-y-3\"><p class=\"text-gray-700\" style=\"line-height:24px;font-size:16px;color:#4a5568;width:100%;margin:0\" align=\"left\">By verifying you agree to Dockify's Terms and Conditions and Privacy Policy</p></div><table class=\"s-5 w-full\" role=\"presentation\" border=\"0\" cellpadding=\"0\" cellspacing=\"0\" style=\"width:100%\" width=\"100%\"><tbody><tr><td style=\"line-height:20px;font-size:20px;width:100%;height:20px;margin:0\" align=\"left\" width=\"100%\" height=\"20\">&nbsp;</td></tr></tbody></table><table class=\"hr\" role=\"presentation\" border=\"0\" cellpadding=\"0\" cellspacing=\"0\" style=\"width:100%\"><tbody><tr><td style=\"line-height:24px;font-size:16px;border-top-width:1px;border-top-color:#e2e8f0;border-top-style:solid;height:1px;width:100%;margin:0\" align=\"left\"></td></tr></tbody></table><table class=\"s-5 w-full\" role=\"presentation\" border=\"0\" cellpadding=\"0\" cellspacing=\"0\" style=\"width:100%\" width=\"100%\"><tbody><tr><td style=\"line-height:20px;font-size:20px;width:100%;height:20px;margin:0\" align=\"left\" width=\"100%\" height=\"20\">&nbsp;</td></tr></tbody></table><table class=\"btn btn-primary\" role=\"presentation\" border=\"0\" cellpadding=\"0\" cellspacing=\"0\" style=\"border-radius:6px;border-collapse:separate!important\"><tbody><tr><td style=\"line-height:24px;font-size:16px;border-radius:6px;margin:0\" align=\"center\" bgcolor=\"#0d6efd\"><a href=\"https://dockify.xyz/verify?code={}\" target=\"_blank\" style=\"color:#fff;font-size:16px;font-family:Helvetica,Arial,sans-serif;text-decoration:none;border-radius:6px;line-height:20px;display:block;font-weight:400;white-space:nowrap;background-color:#0d6efd;padding:8px 12px;border:1px solid #0d6efd\">Click to Verify</a></td></tr></tbody></table></td></tr></tbody></table></td></tr></tbody></table><table class=\"s-10 w-full\" role=\"presentation\" border=\"0\" cellpadding=\"0\" cellspacing=\"0\" style=\"width:100%\" width=\"100%\"><tbody><tr><td style=\"line-height:40px;font-size:40px;width:100%;height:40px;margin:0\" align=\"left\" width=\"100%\" height=\"40\">&nbsp;</td></tr></tbody></table></td></tr></tbody></table><!--[if (gte mso 9)|(IE)]><![endif]--></td></tr></tbody></table></td></tr></tbody></table></body></html>",
            general_purpose::STANDARD.encode(verification_code)
        ))) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error building email: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    match MAILER.send(&email) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error sending email: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }
    return StatusCode::ACCEPTED;
}

pub fn get_routes() -> Router {
    Router::new().route("/api/signup", post(handler))
}
