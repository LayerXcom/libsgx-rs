use std::{
    prelude::v1::*,
    net::TcpStream,
    str,
    time::SystemTime,
    untrusted::time::SystemTimeEx,
    io::{BufReader, Write},
    collections::HashMap,
};
use http_req::{request::{Request, Method}, uri::Uri, response::{Headers, Response}};
use anyhow::{Result, anyhow};

pub const IAS_URL: &str = "https://api.trustedservices.intel.com/sgx/dev/attestation/v3/report";
// pub const DEV_HOSTNAME : &str = "api.trustedservices.intel.com";
// pub const REPORT_PATH : &str = "/sgx/dev/attestation/v3/report";
pub const TEST_SUB_KEY: &str = "77e2533de0624df28dc3be3a5b9e50d9";
pub const TEST_SPID: &str = "2C149BFC94A61D306A96211AED155BE9";

pub const IAS_REPORT_CA: &[u8] = include_bytes!("../AttestationReportSigningCACert.pem");
type SignatureAlgorithms = &'static [&'static webpki::SignatureAlgorithm];
static SUPPORTED_SIG_ALGS: SignatureAlgorithms = &[
    &webpki::ECDSA_P256_SHA256,
    &webpki::ECDSA_P256_SHA384,
    &webpki::ECDSA_P384_SHA256,
    &webpki::ECDSA_P384_SHA384,
    &webpki::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
    &webpki::RSA_PSS_2048_8192_SHA384_LEGACY_KEY,
    &webpki::RSA_PSS_2048_8192_SHA512_LEGACY_KEY,
    &webpki::RSA_PKCS1_2048_8192_SHA256,
    &webpki::RSA_PKCS1_2048_8192_SHA384,
    &webpki::RSA_PKCS1_2048_8192_SHA512,
    &webpki::RSA_PKCS1_3072_8192_SHA384,
];

pub struct RAService;

impl RAService {
    pub fn remote_attestation(
        uri: &str,
        ias_api_key: &str,
        quote: &str,
    ) -> Result<(Report, ReportSig)> {
        let uri: Uri = uri.parse().expect("Invalid uri");
        let body = format!("{{\"isvEnclaveQuote\":\"{}\"}}\r\n", quote);
        let mut writer = Vec::new();

        let response = RAClient::new(&uri)
            .ias_apikey_header_mut(ias_api_key)
            .quote_body_mut(&body.as_bytes())
            .send(&mut writer)?;

        let ra_resp = RAResponse::from_response(writer, response)?;
        Ok((ra_resp.body, ra_resp.sig))
    }
}

pub struct RAClient<'a> {
    request: Request<'a>,
    host: String,
}

impl<'a> RAClient<'a> {
    pub fn new(uri: &'a Uri) -> Self {
        let host = uri.host_header().expect("Not found host in the uri");

        RAClient{
            request: Request::new(&uri),
            host,
        }
    }

    pub fn ias_apikey_header_mut(&mut self, ias_api_key: &str) -> &mut Self {
        let mut headers = Headers::new();
        headers.insert("HOST", &self.host);
        headers.insert("Ocp-Apim-Subscription-Key", ias_api_key);
        headers.insert("Connection", "close");
        self.request.headers(headers);
        self.request.method(Method::POST);

        self
    }

    /// Sets the body to the JSON serialization of the passed value, and
    /// also sets the `Content-Type: application/json` header.
    pub fn quote_body_mut(&'a mut self, body: &'a [u8]) -> &mut Self {
        let len = body.len().to_string();
        self.request.header("Content-Type", "application/json");
        self.request.header("Content-Length", &len);
        self.request.body(&body);

        self
    }

    pub fn send<T: Write>(&self, writer: &mut T) -> Result<Response> {
        self.request.send(writer)
            .map_err(|e| anyhow!("{:?}", e))
            .map_err(Into::into)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Report(Vec<u8>);

impl Report {
    pub fn new(report: Vec<u8>) -> Self {
        Report(report)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReportSig(Vec<u8>);

impl ReportSig {
    pub fn base64_decode(v: &[u8]) -> Result<Self> {
        let v = base64::decode(v)?;
        Ok(ReportSig(v))
    }

    pub fn new(report_sig: Vec<u8>) -> Self {
        ReportSig(report_sig)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

#[derive(Debug, Clone)]
pub struct RAResponse {
    body: Report,
    sig: ReportSig,
    cert: Vec<u8>,
}

impl RAResponse {
    pub fn from_response(body: Vec<u8>, resp: Response) -> Result<Self> {
        // TODO: ADD status_code verifications

        let headers = resp.headers();
        let sig = headers.get("X-IASReport-Signature")
            .ok_or(anyhow!("Not found X-IASReport-Signature header"))?;
        let sig = ReportSig::base64_decode(sig.as_bytes())?;

        let cert = headers.get("X-IASReport-Signing-Certificate")
            .ok_or(anyhow!("Not found X-IASReport-Signing-Certificate"))?
            .replace("%0A", "");
        let cert = percent_decode(cert)?;

        Ok(RAResponse {
            body: Report::new(body),
            sig,
            cert,
        })
    }

    // pub fn parse(resp : &[u8]) -> Result<Self> {
    //     let mut headers = [httparse::EMPTY_HEADER; 16];
    //     let mut respp   = httparse::Response::new(&mut headers);
    //     let result = respp.parse(resp);

    //     let msg : &'static str;

    //     match respp.code {
    //         Some(200) => msg = "OK Operation Successful",
    //         Some(401) => msg = "Unauthorized Failed to authenticate or authorize request.",
    //         Some(404) => msg = "Not Found GID does not refer to a valid EPID group ID.",
    //         Some(500) => msg = "Internal error occurred",
    //         Some(503) => msg = "Service is currently not able to process the request (due to
    //             a temporary overloading or maintenance). This is a
    //             temporary state – the same request can be repeated after
    //             some time. ",
    //         _ => {println!("DBG:{}", respp.code.unwrap()); msg = "Unknown error occurred"},
    //     }

    //     println!("    [Enclave] msg = {}", msg);
    //     let mut len_num : u32 = 0;

    //     let mut sig = ReportSig::default();
    //     let mut cert_str = String::new();
    //     let mut body = Report::default();

    //     for i in 0..respp.headers.len() {
    //         let h = respp.headers[i];
    //         match h.name{
    //             "Content-Length" => {
    //                 let len_str = String::from_utf8(h.value.to_vec()).unwrap();
    //                 len_num = len_str.parse::<u32>().unwrap();
    //             }
    //             "X-IASReport-Signature" => sig = ReportSig::base64_decode(h.value)?,
    //             "X-IASReport-Signing-Certificate" => cert_str = String::from_utf8(h.value.to_vec()).unwrap(),
    //             _ => (),
    //         }
    //     }

    //     // Remove %0A from cert, and only obtain the signing cert
    //     cert_str = cert_str.replace("%0A", "");
    //     cert_str = percent_decode(cert_str);
    //     let v: Vec<&str> = cert_str.split("-----").collect();
    //     let cert = base64::decode(v[2])?;

    //     // This root_cert is equal to AttestationReportSigningCACert.pem
    //     // let root_cert = v[6].to_string();

    //     if len_num != 0 {
    //         let header_len = result.unwrap().unwrap();
    //         body = Report::new(resp[header_len..].to_vec());
    //     }

    //     Ok(Response {
    //         body,
    //         sig,
    //         cert,
    //     })
    // }

    fn verify_sig_cert(&self) -> Result<()> {
        let now_func = webpki::Time::try_from(SystemTime::now())?;

        let mut ca_reader = BufReader::new(&IAS_REPORT_CA[..]);
        let mut root_store = rustls::RootCertStore::empty();
        root_store.add_pem_file(&mut ca_reader).expect("Failed to add CA");

        let trust_anchors: Vec<webpki::TrustAnchor> = root_store
            .roots
            .iter()
            .map(|cert| cert.to_trust_anchor())
            .collect();

        let ias_cert_dec = Self::decode_ias_report_ca()?;
        let mut chain:Vec<&[u8]> = Vec::new();
        chain.push(&ias_cert_dec);

        let sig_cert = webpki::EndEntityCert::from(&self.cert)?;

        sig_cert.verify_is_valid_tls_server_cert(
            SUPPORTED_SIG_ALGS,
            &webpki::TLSServerTrustAnchors(&trust_anchors),
            &chain,
            now_func,
        )?;

        sig_cert.verify_signature(
            &webpki::RSA_PKCS1_2048_8192_SHA256,
            &self.body.0,
            &self.sig.0,
        )?;

        Ok(())
    }

    // fn verify_report(&self) -> Result<()> {
    //     // timestamp is within 24H (90day is recommended by Intel)
    //     let attn_report: Value = serde_json::from_slice(attn_report_raw).unwrap();
    //     if let Value::String(time) = &attn_report["timestamp"] {
    //         let time_fixed = time.clone() + "+0000";
    //         let ts = DateTime::parse_from_str(&time_fixed, "%Y-%m-%dT%H:%M:%S%.f%z").unwrap().timestamp();
    //         let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    //     } else {
    //         println!("Failed to fetch timestamp from attestation report");
    //         return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    //     }
    // }

    fn decode_ias_report_ca() -> Result<Vec<u8>> {
        let mut ias_ca_stripped = IAS_REPORT_CA.to_vec();
        ias_ca_stripped.retain(|&x| x != 0x0d && x != 0x0a);
        let head_len = "-----BEGIN CERTIFICATE-----".len();
        let tail_len = "-----END CERTIFICATE-----".len();

        let full_len = ias_ca_stripped.len();
        let ias_ca_core : &[u8] = &ias_ca_stripped[head_len..full_len - tail_len];
        let ias_cert_dec = base64::decode(ias_ca_core)?;
        Ok(ias_cert_dec)
    }
}

fn percent_decode(orig: String) -> Result<Vec<u8>> {
    let v:Vec<&str> = orig.split('%').collect();
    let mut ret = String::new();
    ret.push_str(v[0]);
    if v.len() > 1 {
        for s in v[1..].iter() {
            ret.push(u8::from_str_radix(&s[0..2], 16).unwrap() as char);
            ret.push_str(&s[2..]);
        }
    }
    let v: Vec<&str> = ret.split("-----").collect();
    base64::decode(v[2]).map_err(Into::into)
}
