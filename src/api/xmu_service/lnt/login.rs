use anyhow::{Result, anyhow};
use base64::Engine;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginApiBody {
    #[serde(rename = "lt")]
    token: &'static str, //登录令牌，固定为空
    #[serde(rename = "uuid", skip_serializing_if = "Option::is_none")]
    pub qrcode_id: Option<String>, //二维码UUID
    #[serde(rename = "cllt")]
    client_type: &'static str, //登录类型
    #[serde(rename = "dllt")]
    login_type: &'static str, //登录方式
    #[serde(rename = "execution")]
    execution: String, //执行标识
    #[serde(rename = "_eventId")]
    event_id: &'static str, //事件ID，固定为submit
    #[serde(rename = "rmShown")]
    remember_me: Option<&'static str>, //是否显示记住我，固定为1
    #[serde(rename = "username", skip_serializing_if = "Option::is_none")]
    username: Option<String>, //用户名
    #[serde(rename = "password", skip_serializing_if = "Option::is_none")]
    password: Option<String>, //密码
    #[serde(rename = "captcha", skip_serializing_if = "Option::is_none")]
    captcha: Option<&'static str>, //验证码，默认为Some("")
}

#[derive(Serialize, Debug)]
pub struct LoginRequest {
    pub url: String,
    pub body: LoginApiBody,
}

impl LoginRequest {
    pub fn qrcode(url: String, qrcode_id: String, execution: String) -> Self {
        LoginRequest {
            url,
            body: LoginApiBody {
                token: "",
                qrcode_id: Some(qrcode_id),
                client_type: "qrLogin",
                login_type: "generalLogin",
                execution,
                event_id: "submit",
                remember_me: Some("1"),
                username: None,
                password: None,
                captcha: None,
            },
        }
    }

    pub fn password(
        url: String,
        execution: String,
        salt: &str,
        username: String,
        password: &str,
    ) -> Result<Self> {
        let mut random_password = Vec::with_capacity(64 + password.len());
        fill_random_bytes_vec(&mut random_password, 64);
        random_password.extend_from_slice(password.as_bytes());

        let mut iv = [0u8; 16];
        fill_random_bytes(&mut iv);

        let encrypted_password_u8 =
            soft_aes::aes::aes_enc_cbc(&random_password, salt.as_bytes(), &iv, Some("PKCS7"))
                .map_err(|e| anyhow!("加密错误，可能是传入的salt不正确: {}", e))?;

        let encrypted_password =
            base64::engine::general_purpose::STANDARD.encode(encrypted_password_u8);

        Ok(Self {
            url,
            body: LoginApiBody {
                token: "",
                qrcode_id: None,
                client_type: "userNameLogin",
                login_type: "generalLogin",
                execution,
                event_id: "submit",
                remember_me: Some("1"),
                username: Some(username),
                password: Some(encrypted_password),
                captcha: Some(""),
            },
        })
    }
}

const AES_CHARS: &[u8] = b"ABCDEFGHJKMNPQRSTWXYZabcdefhijkmnprstwxyz2345678";

fn fill_random_bytes_vec(buf: &mut Vec<u8>, len: usize) {
    let mut rng = rand::rng();
    for _ in 0..len {
        let idx = rng.random_range(0..AES_CHARS.len());
        buf.push(AES_CHARS[idx]);
    }
}

fn fill_random_bytes(dest: &mut [u8]) {
    let mut rng = rand::rng();
    for byte in dest.iter_mut() {
        let idx = rng.random_range(0..AES_CHARS.len());
        *byte = AES_CHARS[idx];
    }
}
